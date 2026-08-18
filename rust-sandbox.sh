#!/usr/bin/env bash
# rust-sandbox: run cargo inside an ephemeral, cache-backed container.
# Generic — drop into any Rust project root (next to Cargo.toml) and run.
#
# Usage:  ./rust-sandbox.sh <cargo-subcmd> [args...]
#   e.g.  ./rust-sandbox.sh check
#         ./rust-sandbox.sh clippy -- -D warnings
#         ./rust-sandbox.sh run
#         ./rust-sandbox.sh run -- --some-flag
#
# Design:
#  - Project dir bind-mounted read-write at /work  -> host `target/` cache is shared.
#  - Persistent named volume for the cargo registry  -> dependency fetch is one-time.
#  - Container runs as host uid:gid  -> files it writes (in target/) stay host-owned.
#  - .git bind-mounted read-only  -> buggy/generated code can't corrupt history.
#  - --rm + --init  -> ephemeral container, clean signal propagation, no orphans.
#  - Image rust version pinned to host rustc  -> compatible shared target cache.
#
# Gotcha: the cache volume MUST NOT mount at /usr/local/cargo (that path holds the
# cargo BINARY in the rust image). We use a separate CARGO_HOME=/cargo-cache.
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
CARGO_VOL="rust-sandbox-cargo"
IMAGE_NAME="${RUST_SANDBOX_IMAGE:-rust-sandbox:latest}"
UID_G="$(id -u):$(id -g)"

# Use sudo only if the user can't reach the docker socket directly
# (durable fix: `sudo usermod -aG docker $USER` + re-login).
DOCKER=docker
if ! docker info >/dev/null 2>&1; then
  DOCKER="sudo docker"
fi

# RUST image tag. Default: match host rustc (keeps the shared target/ cache
# compatible) BUT with a floor — if host rustc is older than RUST_SANDBOX_RUST_MIN,
# bump to the floor, because many projects now need edition2024 (rust >=1.85)
# or a Cargo.lock v4 (cargo >=1.78) and an old host toolchain silently builds
# an unusable image. Override per-project with the exact version, e.g.
#   RUST_SANDBOX_RUST_TAG=rust:1.89-slim-bookworm  (solana deps need >=1.89)
# Avoid rust 1.97: its bundled rust-lld hits an "invalid symbol index" bug
# linking the test harness on a fresh build.
RUST_SANDBOX_RUST_MIN="${RUST_SANDBOX_RUST_MIN:-1.85}"
RUST_TAG="${RUST_SANDBOX_RUST_TAG:-}"
if [ -z "$RUST_TAG" ]; then
  RUST_TAG="rust:slim-bookworm"
  if command -v rustc >/dev/null 2>&1; then
    if rustver="$(rustc --version 2>/dev/null | sed -n 's/^rustc \([0-9]*\.[0-9]*\)\..*/\1/p')"; then
      [ -n "$rustver" ] && RUST_TAG="rust:${rustver}-slim-bookworm"
      if [ -n "$rustver" ] && awk "BEGIN{exit !($rustver < $RUST_SANDBOX_RUST_MIN)}"; then
        RUST_TAG="rust:${RUST_SANDBOX_RUST_MIN}-slim-bookworm"
        echo "[rust-sandbox] host rustc $rustver < min $RUST_SANDBOX_RUST_MIN; using $RUST_TAG" >&2
      fi
    fi
  fi
fi

# Build the image once (cache keyed on RUST_TAG via a build arg).
if ! $DOCKER image inspect "$IMAGE_NAME" >/dev/null 2>&1; then
  echo "[rust-sandbox] building image $IMAGE_NAME ($RUST_TAG) ..." >&2
  $DOCKER build --build-arg RUST_IMAGE="$RUST_TAG" \
    -f "$PROJECT_DIR/Dockerfile.sandbox" -t "$IMAGE_NAME" "$PROJECT_DIR" >&2
fi

# Ensure the cargo-registry volume exists and is owned by the host uid
# (one-time, idempotent via a marker file).
$DOCKER volume create "$CARGO_VOL" >/dev/null 2>&1 || true
if ! $DOCKER run --rm -v "$CARGO_VOL":/c alpine test -f /c/.sandbox-init 2>/dev/null; then
  echo "[rust-sandbox] initializing cargo cache volume ..." >&2
  $DOCKER run --rm -v "$CARGO_VOL":/c alpine \
    sh -c "chown -R $UID_G /c && touch /c/.sandbox-init" >/dev/null
fi

# Mount .git read-only only if it exists.
GIT_MOUNT=()
if [ -d "$PROJECT_DIR/.git" ]; then
  GIT_MOUNT=(-v "$PROJECT_DIR/.git":/work/.git:ro)
fi

exec $DOCKER run --rm --init \
  --user "$UID_G" \
  -e CARGO_HOME=/cargo-cache \
  -e CARGO_BUILD_JOBS="${RUST_SANDBOX_JOBS:-}" \
  -e USER="$(id -un)" \
  -v "$CARGO_VOL":/cargo-cache \
  -v "$PROJECT_DIR":/work \
  "${GIT_MOUNT[@]}" \
  -w /work \
  "$IMAGE_NAME" cargo "$@"