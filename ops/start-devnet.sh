#!/usr/bin/env bash
# Zyanya devnet launcher — crypto server (10.10.1.119)
# Starts: zyanyad (with --utxoindex) + miner (25% CPU) + pool
# Usage: ./start-devnet.sh   (idempotent — kills existing instances first)
set -euo pipefail

NODE_BIN=/home/shawn/projects/blockchain-fork/rusty-spectre/target/release/zyanyad
MINER_BIN=/home/shawn/projects/blockchain-fork/zyanya-miner/target/release/zyanya-miner
POOL_BIN=/home/shawn/projects/blockchain-fork/zyanya-pool/target/release/zyanya-pool
APPDIR=/home/shawn/.local/share/zyanya-devnet
MINING_ADDR=zyanyadev:qrncgmfvvgp63rlhuew6phnzxc9cy2fczt4pgsqpdaepft8592zwg4l7fma02

# Kill any existing instances
pkill -f "zyanya-miner --devnet" 2>/dev/null || true
pkill -f "zyanya-pool" 2>/dev/null || true
pkill -f "zyanyad --devnet" 2>/dev/null || true
sleep 2

# Start node with --utxoindex (required for coin_supply + balance queries in explorer)
nohup "$NODE_BIN" --devnet \
  --appdir "$APPDIR" \
  --nologfiles \
  --listen=[::]:18611 \
  --rpclisten=[::]:18610 \
  --rpclisten-json=[::]:20610 \
  --utxoindex \
  --enable-unsynced-mining \
  > /tmp/zyanyad-devnet.log 2>&1 &
echo "zyanyad started (PID $!) — waiting for RPC..."
sleep 4

# Start miner
nohup "$MINER_BIN" --devnet \
  --mine-when-not-synced \
  --cpu-percent 25 \
  --zyanyad-address=127.0.0.1 \
  --port=18610 \
  --mining-address="$MINING_ADDR" \
  > /tmp/zyanya-miner-devnet.log 2>&1 &
echo "zyanya-miner started (PID $!)"

# Start pool
nohup "$POOL_BIN" \
  --node 127.0.0.1:18610 \
  --listen [::]:3334 \
  --fee 1.0 \
  > /tmp/zyanya-pool-devnet.log 2>&1 &
echo "zyanya-pool started (PID $!)"

echo "Devnet up. Logs: /tmp/zyanya{d,miner,pool}-devnet.log"