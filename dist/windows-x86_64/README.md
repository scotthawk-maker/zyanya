# Zyanya Windows x86_64 build (2026-08-04)

Cross-compiled from the Zyanya source (`x86_64-pc-windows-gnu` target) in the
omp-test container on unRAID, using mingw-w64 (gcc 12-posix) + Rust stable.

## Binaries
| File | Size | Notes |
|------|------|-------|
| `zyanyad.exe` | ~45 MB | Full node daemon (v0.3.17) |
| `zyanya-query.exe` | ~18 MB | gRPC query CLI |
| `rothschild.exe` | ~18 MB | New bin since Jul-29 (replaces the removed `zyanya-miner`) |

## Runtime dependencies (mingw DLLs — ship alongside the .exe)
| File | Size |
|------|------|
| `libgcc_s_seh-1.dll` | ~650 KB |
| `libstdc++-6.dll` | ~23 MB |
| `libwinpthread-1.dll` | ~320 KB |

## Known issue — runtime crash on Windows 11 build 26200
`zyanyad.exe` crashes on startup on Windows 11 build 26200 (Insider/25H2) with
`0xc0000005` (ACCESS_VIOLATION) at offset 0x0 in an unknown module — a NULL
function-pointer call inside `create_core`, after logging setup, before
networking/consensus init. The Jul-30 build of the same v0.3.17 ran and synced
node-data on this box on Jul 30 but crashes the same way now, so a Windows
update broke the mingw-built binary's runtime. Mitigation options: newer
mingw toolchain, MSVC target via xwin, source bisect Jul30->Aug4, static-link
mingw runtime, WinDbg for the exact faulting RVA.

## Reproduce the build
Inside the omp-test container (mingw-w64 gcc 12-posix + Rust stable,
x86_64-pc-windows-gnu target installed; cross-compile env vars preset):
```
cd /zyanya-src
cargo build --release --target x86_64-pc-windows-gnu \
  --bin zyanyad --bin zyanya-query --bin rothschild
```
