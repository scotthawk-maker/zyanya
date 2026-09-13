#!/usr/bin/env python3
"""
Zyanya Production Release Packaging & Checksum Generator
Packages core daemons, miner, wallet, and tooling into official release bundles
with SHA256 checksums for GitHub Releases.
"""

import sys
import os
import shutil
import hashlib
import zipfile
import tarfile
import argparse
from pathlib import Path

# Ensure UTF-8 console output on Windows
if sys.stdout.encoding != 'utf-8':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
        sys.stderr.reconfigure(encoding='utf-8')
    except Exception:
        pass

REPO_ROOT = Path(__file__).resolve().parent.parent
MINER_ROOT = REPO_ROOT.parent / "zyanya-miner"
DIST_DIR = REPO_ROOT.parent / "releases"

BINARIES = [
    "zyanyad",
    "zyanya-miner",
    "zyanya-wallet",
    "zyanya-query",
    "zyanya-explorer"
]

README_TEMPLATE = """================================================================================
  ZYANYA CORE & STANDALONE CPU MINER ({version} - {platform})
  Decentralized GHOSTDAG PoW Layer 1 | AstroBWTv3 | Native WebMCP
================================================================================

BINARIES INCLUDED:
  - zyanyad{ext}          : High-performance GHOSTDAG consensus node daemon
  - zyanya-miner{ext}     : Standalone AstroBWTv3 CPU miner (Zero-AI required)
  - zyanya-wallet{ext}    : Sovereign CLI wallet generator & manager
  - zyanya-query{ext}     : Direct gRPC blockchain query tool
  - zyanya-explorer{ext}  : Local block explorer daemon & UI server

QUICKSTART (MAINNET - OCTOBER 1, 2026):

1. Generate a Mining Address:
   .{sep}zyanya-wallet{ext} new-address

2. Run the Node Daemon:
   .{sep}zyanyad{ext} --utxoindex

3. Start Solo Mining (AstroBWTv3 CPU):
   .{sep}zyanya-miner{ext} --threads 8 --mining-address <YOUR_ZYANYA_ADDRESS>

QUICKSTART (TESTNET):

1. Generate a Testnet Address:
   .{sep}zyanya-wallet{ext}
   $ network testnet-10
   $ new-address

2. Run the Testnet Node Daemon:
   .{sep}zyanyad{ext} --testnet --utxoindex

3. Start Solo Mining (Testnet):
   .{sep}zyanya-miner{ext} --testnet --threads 8 --mining-address <YOUR_ZYANYATEST_ADDRESS>

DOCUMENTATION & NETWORK EXPLORER:
  GitHub:      https://github.com/scotthawk-maker/zyanya
  SourceForge: https://sourceforge.net/projects/zyanya/
  Docs:        https://github.com/scotthawk-maker/zyanya/tree/main/docs
  Web:         https://zyanya.scottcloudhawk.org
  WebMCP:      https://zyanya.scottcloudhawk.org/mcp/rpc
"""

def sha256_file(filepath: Path) -> str:
    hasher = hashlib.sha256()
    with open(filepath, "rb") as f:
        while chunk := f.read(65536):
            hasher.update(chunk)
    return hasher.hexdigest()

def find_binary(name: str, is_windows: bool) -> Path:
    ext = ".exe" if is_windows else ""
    binary_name = f"{name}{ext}"
    
    # 1. Check zyanya-miner repo target
    if name == "zyanya-miner":
        candidate = MINER_ROOT / "target" / "release" / binary_name
        if candidate.exists():
            return candidate

    # 2. Check main repo target
    candidate = REPO_ROOT / "target" / "release" / binary_name
    if candidate.exists():
        return candidate

    # 3. Check parent build folder
    candidate = REPO_ROOT.parent / f"zyanya-v0.4.0-windows-x64" / binary_name
    if candidate.exists():
        return candidate

    raise FileNotFoundError(f"Could not locate compiled binary: {binary_name}")

def package_windows(version: str) -> Path:
    bundle_name = f"zyanya-{version}-windows-x64"
    stage_dir = DIST_DIR / bundle_name
    zip_path = DIST_DIR / f"{bundle_name}.zip"

    print(f"\n📦 Packaging Windows x64 Bundle: {bundle_name}.zip")
    if stage_dir.exists():
        shutil.rmtree(stage_dir)
    stage_dir.mkdir(parents=True, exist_ok=True)

    # Copy binaries
    for b in BINARIES:
        src = find_binary(b, is_windows=True)
        dst = stage_dir / src.name
        print(f"  • Copying {src.name} ({src.stat().st_size / 1024 / 1024:.2f} MB)...")
        shutil.copy2(src, dst)

    # Write README.txt
    readme_content = README_TEMPLATE.format(
        version=version,
        platform="Windows x64",
        ext=".exe",
        sep="\\"
    )
    (stage_dir / "README.txt").write_text(readme_content, encoding="utf-8")

    # Create ZIP archive
    if zip_path.exists():
        zip_path.unlink()

    with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as zipf:
        for root, _, files in os.walk(stage_dir):
            for file in files:
                file_path = Path(root) / file
                arcname = file_path.relative_to(DIST_DIR)
                zipf.write(file_path, arcname)

    # Calculate SHA256
    digest = sha256_file(zip_path)
    sha_path = DIST_DIR / f"{bundle_name}.zip.sha256"
    sha_path.write_text(f"{digest}  {zip_path.name}\n", encoding="utf-8")

    print(f"  ✅ Created: {zip_path.name} ({zip_path.stat().st_size / 1024 / 1024:.2f} MB)")
    print(f"  🔑 SHA256:  {digest}")
    return zip_path

def main():
    parser = argparse.ArgumentParser(description="Zyanya Release Packager")
    parser.add_argument("--version", default="v1.0.0-mainnet", help="Release version tag (default: v1.0.0-mainnet)")
    args = parser.parse_args()

    DIST_DIR.mkdir(parents=True, exist_ok=True)

    print("=" * 75)
    print(f"  ZYANYA OFFICIAL RELEASE PACKAGING SYSTEM ({args.version})")
    print("=" * 75)

    created_archives = []

    # Windows packaging
    if sys.platform == "win32" or os.name == "nt":
        try:
            zip_path = package_windows(args.version)
            created_archives.append(zip_path)
        except Exception as e:
            print(f"  ❌ Windows packaging error: {e}")
            sys.exit(1)

    # Aggregate sha256sums.txt
    sums_file = DIST_DIR / f"sha256sums-{args.version}.txt"
    with open(sums_file, "w", encoding="utf-8") as out:
        for archive in created_archives:
            h = sha256_file(archive)
            out.write(f"{h}  {archive.name}\n")

    print("\n" + "=" * 75)
    print(f"  RELEASE PACKAGING COMPLETE")
    print(f"  Artifacts directory: {DIST_DIR}")
    print(f"  Checksum file:       {sums_file.name}")
    print("=" * 75)

if __name__ == "__main__":
    main()
