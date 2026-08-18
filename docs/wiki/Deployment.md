# Deployment

## Build Requirements

- **Rust** stable (1.97+)
- **MSVC toolchain** for Windows (GNU toolchain caused crashes — see WO #18)
- **RocksDB** development libraries
- **CMake** (for native build scripts)

## Building on Windows (from Linux host via SSH)

```bash
# SSH to Windows node
ssh windows

# Build (always specify -p <package> due to ambiguous binary targets)
cd C:\Users\Shawn\zyanya-build\rusty-spectre-git
cargo build -p zyanya-vm --release
cargo build -p zyanya-consensus --release
cargo build -p zyanyad --release
```

### ⚠️ Ambiguous Binary Targets
`zyanya-wallet` exists in two packages. Always pass `-p <package>` when building specific binaries:
```bash
cargo build -p zyanyad    # NOT cargo build zyanyad
```

### ⚠️ Git Credentials Over SSH
Windows GCM defaults to `wincredman` which **does not work over SSH sessions**. Fix:
```bash
git config --global credential.credentialStore dpapi
```
DPAPI-protected files work in SSH sessions. Credentials stored at `%USERPROFILE%\.gcm\dpapi_store`.

## Cross-Compilation (Linux → Windows)

```bash
# Add MSVC target
rustup target add x86_64-pc-windows-msvc

# Build
cargo build -p zyanyad --target x86_64-pc-windows-msvc --release
```

**Do NOT use `x86_64-pc-windows-gnu`** — causes NULL deref crash on Windows 11 build 26200 (see Issue #1).

## Running a Node

```bash
# Testnet
zyanyad --testnet --netsuffix=10 --appdir=/path/to/node-data --nologfiles

# Devnet
zyanyad --devnet --appdir=/path/to/node-data --nologfiles
```

## 3-Node Testnet

| Node | OS | Tailscale IP | Command |
|------|-----|-------------|---------|
| cachyos | Linux | 100.124.134.6 | `zyanyad --devnet --appdir=~/node-data` |
| minisforum | Windows | 100.83.211.115 | `zyanyad.exe --devnet --appdir=C:\node-data` |
| scotthawk | Linux | 100.106.22.123 | `zyanyad --devnet --appdir=~/node-data` |

Nodes peer over IPv6 via Tailscale.

## Explorer

```bash
zyanya-explorer --listen [::]:8098 --rpcserver 127.0.0.1:18610
```

State-changing endpoints disabled by default. Enable with:
```bash
ZYANYA_EXPLORER_ENABLE_WRITE=1 zyanya-explorer --listen [::]:8098
```

## Wallet

```bash
# Generate key (address only, no secret in output)
zyanya-wallet --generate-key

# Generate with secret visible
zyanya-wallet --generate-key --show-secret

# Launch TUI
zyanya-wallet --devnet --rpcserver 127.0.0.1:18610
```

## Testing

```bash
# VM tests
cargo test -p zyanya-vm

# Consensus tests
cargo test -p zyanya-consensus --lib

# All tests
cargo test --workspace
```

Expected: 19 VM unit + 8 VM integration + 50 consensus + 24 consensus-core = 101 tests pass.