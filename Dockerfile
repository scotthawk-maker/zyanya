# ==============================================================================
# Zyanya High-Performance Node & Sovereign Suite Production Image
# Multi-Stage Build: Rust 1.83 (Debian Bookworm) -> Minimal Runtime
# ==============================================================================

FROM rust:1.83-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    libclang-dev \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

COPY . .

ENV CARGO_INCREMENTAL=0
ENV RUSTFLAGS="-C target-cpu=native"

RUN cargo build --release -p zyanyad -p zyanya-wallet -p zyanya-query -p zyanya-explorer

# ------------------------------------------------------------------------------
# Runtime Image
# ------------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    jq \
    libssl3 \
    netcat-openbsd \
    procps \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled binaries from builder stage
COPY --from=builder /build/target/release/zyanyad /usr/local/bin/zyanyad
COPY --from=builder /build/target/release/zyanya-wallet /usr/local/bin/zyanya-wallet
COPY --from=builder /build/target/release/zyanya-query /usr/local/bin/zyanya-query
COPY --from=builder /build/target/release/zyanya-explorer /usr/local/bin/zyanya-explorer

# Fetch standalone AstroBWTv3 miner binary
RUN curl -fsSL https://github.com/scotthawk-maker/zyanya-miner/releases/download/v0.4.0/zyanya-miner-v0.4.0-linux-x86_64.tar.gz | tar -xz -C /usr/local/bin zyanya-miner && \
    chmod +x /usr/local/bin/zyanya-miner || true

# Copy and prepare entrypoint
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Create non-root user and persistent directories
RUN useradd -m -u 1000 -U -s /bin/bash zyanya && \
    mkdir -p /home/zyanya/.zyanyad && \
    chown -R zyanya:zyanya /home/zyanya

USER zyanya
VOLUME ["/home/zyanya/.zyanyad"]

# Ports:
# 18111: Mainnet P2P (18211 Testnet)
# 18110: Mainnet gRPC (18210 Testnet)
# 19110: Mainnet Borsh wRPC (19210 Testnet)
# 20110: Mainnet JSON wRPC (20210 Testnet)
# 8099:  Local Block Explorer
EXPOSE 18111 18110 19110 20110 8099

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["zyanyad"]
