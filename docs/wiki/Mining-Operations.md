# Zyanya Mining Operations Guide (v0.4.0)

> **Official Release**: v0.4.0 Production Binaries  
> **Consensus Engine**: GhostDAG 1 BPS with AstroBWTv3 Proof-of-Work  
> **Coinbase Reward**: 50 ZYAN per block with zero premine (Genesis: `OP_FALSE`)  
> **Payout Architecture**: Direct on-chain UTXO credit via Coinbase Transaction

---

## 1. Network Parameters & Port Matrix

Zyanya binaries natively support both Mainnet and Testnet-10 environments. Ensure local firewall rules and socket arguments match your target deployment:

```
+--------------------------------------------------------------------------+
| MAINNET (Default Production)                                             |
+--------------------------------------------------------------------------+
| • P2P Protocol:       TCP [::]:18111 (Pure IPv6)                         |
| • gRPC RPC:           127.0.0.1:18110 (or [::1]:18110)                   |
| • wRPC Borsh:         19110                                              |
| • wRPC JSON / MCP:    20110                                              |
| • Address Prefix:     zyanya:                                            |
| • Coinbase Reward:    50 ZYAN                                            |
| • Launch Flag:        Default (no network flag required)                 |
+--------------------------------------------------------------------------+

+--------------------------------------------------------------------------+
| TESTNET-10 (Public Staging Mesh)                                         |
+--------------------------------------------------------------------------+
| • P2P Protocol:       TCP [::]:18211 (Pure IPv6)                         |
| • gRPC RPC:           127.0.0.1:18210 (or [::1]:18210)                   |
| • wRPC Borsh:         19210                                              |
| • wRPC JSON / MCP:    20210                                              |
| • Address Prefix:     zyanyatest:                                        |
| • Coinbase Reward:    50 ZYAN (Testnet Sompi)                            |
| • Launch Flag:        --testnet (or --netsuffix=10)                      |
+--------------------------------------------------------------------------+
```

---

## 2. Standalone Solo CPU Mining Quickstart

Solo mining communicates directly with your local `zyanyad` node daemon via gRPC block template polling.

### 2.1 Package Verification & Extraction

```
+--------------------------------------------------------------------------+
| Official Release Hashes (v0.4.0)                                         |
+--------------------------------------------------------------------------+
| Linux x86_64:                                                            |
| cfa8cbdc75267298613999b907db8ebec7e07148431d181e6c7cfc723f9201b1        |
|                                                                          |
| Windows x64:                                                             |
| 9f62b1bd33d4569b07e7695743f5ec6b0732339332d2355c1a09a2b7b4a3dd19        |
+--------------------------------------------------------------------------+
```

```bash
# --- Linux x86_64 ---
wget https://zyanya.scottcloudhawk.org/releases/zyanya-v0.4.0-linux-x86_64.tar.gz
echo "cfa8cbdc75267298613999b907db8ebec7e07148431d181e6c7cfc723f9201b1  zyanya-v0.4.0-linux-x86_64.tar.gz" | sha256sum -c -
tar -xzvf zyanya-v0.4.0-linux-x86_64.tar.gz
cd zyanya-v0.4.0-linux-x86_64
chmod +x zyanyad zyanya-miner zyanya-wallet zyanya-query zyanya-explorer
```

```powershell
# --- Windows x64 (PowerShell) ---
Invoke-WebRequest -Uri "https://zyanya.scottcloudhawk.org/releases/zyanya-v0.4.0-windows-x64.zip" -OutFile "zyanya-v0.4.0-windows-x64.zip"
Get-FileHash -Algorithm SHA256 .\zyanya-v0.4.0-windows-x64.zip
Expand-Archive .\zyanya-v0.4.0-windows-x64.zip -DestinationPath .\zyanya-v0.4.0
cd .\zyanya-v0.4.0
```

### 2.2 Address Generation

Generate your mining payout address using the native CLI wallet:

```bash
# Mainnet Payout Address (zyanya:...)
./zyanya-wallet new-address

# Testnet-10 Payout Address (zyanyatest:...)
./zyanya-wallet --testnet new-address
```

### 2.3 Starting Local Node Daemon

```bash
# Mainnet Daemon Execution
./zyanyad --utxoindex

# Testnet-10 Daemon Execution
./zyanyad --testnet --utxoindex
```

### 2.4 Launching the Miner

Open a dedicated terminal window and run `zyanya-miner` pointed to the local node:

```bash
# Mainnet Miner (Allocating 8 worker threads)
./zyanya-miner \
  --mining-address zyanya:qrel0w4d7v9n8s6t5c3m2k1j0h4g7f8d9s0a1b2c3d \
  --threads 8

# Testnet-10 Miner
./zyanya-miner \
  --testnet \
  --mining-address zyanyatest:qrel0w4d7v9n8s6t5c3m2k1j0h4g7f8d9s0a1b2c3d \
  --threads 8
```

---

## 3. Advanced Miner Configuration & Flags

The `zyanya-miner` binary provides granular controls over CPU execution scheduling, pool protocols, and network routing:

```
+--------------------------------------------------------------------------+
| FLAG REFERENCE                                                           |
+--------------------------------------------------------------------------+
| --mining-address <ADDR>   Payout address for mined coinbase rewards      |
| --threads <NUM>           Explicit worker thread count                   |
| --cpu-percent <1-100>     Maximum total CPU utilization cap              |
| --dynamic                 Auto-adjust thread count based on system load  |
| --throttle <MS>           Inter-hash cycle delay in milliseconds         |
| --pool <HOST:PORT>        Stratum pool endpoint for pooled mining        |
| --zyanyad-address <IP>    gRPC host IP for daemon connection             |
| --port <PORT>             gRPC target port (18110 Main, 18210 Testnet)   |
| --mine-when-not-synced    Permit hashing before daemon initial sync completes |
+--------------------------------------------------------------------------+
```

### 3.1 Advanced Flag Usage Examples

```bash
# 1. Thermal-Regulated Mining (Cap at 70% CPU duty cycle, 4 threads)
./zyanya-miner \
  --mining-address zyanya:qrel0w4... \
  --threads 4 \
  --cpu-percent 70

# 2. Dynamic Desktop Background Mode (Yields cores during user activity)
./zyanya-miner \
  --mining-address zyanya:qrel0w4... \
  --dynamic \
  --throttle 5

# 3. Remote Sovereign Node Mining (Daemon hosted on remote IPv6 node)
./zyanya-miner \
  --zyanyad-address 2606:8ac0:2615:79aa::10 \
  --port 18110 \
  --mining-address zyanya:qrel0w4... \
  --threads 16

# 4. Stratum Pool Mining
./zyanya-miner \
  --pool stratum+tcp://pool.zyanya.org:4433 \
  --mining-address zyanya:qrel0w4... \
  --threads 32

# 5. Staging/Local Cluster Testing (Permit mining on unsynced node)
./zyanya-miner \
  --testnet \
  --mine-when-not-synced \
  --mining-address zyanyatest:qrel0w4... \
  --threads 2
```

---

## 4. Enterprise Rig Integration (HiveOS & mmpOS)

Automate large-scale CPU farm deployments across headless Linux mining operating systems.

### 4.1 HiveOS Custom Miner Integration

Create a custom miner integration package (`zyanya-miner.tar.gz`) containing the official v0.4.0 binary and HiveOS wrapper scripts.

#### File: `h-manifest.conf`
```bash
NAME="zyanya-miner"
VERSION="0.4.0"
CUSTOM_NAME="zyanya-miner"
CUSTOM_VERSION="0.4.0"
CUSTOM_BUILD="0"
CUSTOM_LOG_BASENAME="/var/log/miner/custom/zyanya-miner"
CUSTOM_CONFIG_FILENAME="/hive/miners/custom/zyanya-miner/zyanya.conf"
```

#### File: `h-run.sh`
```bash
#!/usr/bin/env bash
[[ ! -s /hive/miners/custom/zyanya-miner/zyanya.conf ]] && echo "Config file missing!" && exit 1
source /hive/miners/custom/zyanya-miner/zyanya.conf

# Enforce 2MB hugepages on startup
sysctl -w vm.nr_hugepages=$(nproc)

# Execute miner binary with flight sheet arguments
cd /hive/miners/custom/zyanya-miner
./zyanya-miner $CUSTOM_USER_ARGS 2>&1 | tee ${CUSTOM_LOG_BASENAME}.log
```

#### File: `h-stats.sh`
```bash
#!/usr/bin/env bash
# Parse hashrate from miner stdout/RPC log
LOG_FILE="/var/log/miner/custom/zyanya-miner.log"
HASHRATE=$(tail -n 20 $LOG_FILE | grep -oP 'Hashrate:\s+\K[0-9.]+' | tail -n 1)
[[ -z $HASHRATE ]] && HASHRATE=0

khs=$(echo "$HASHRATE / 1000" | bc -l)
stats=$(jq -n --arg khs "$khs" --arg hs "$HASHRATE" \
  '{"hs": [$hs|tonumber], "hs_units": "hs", "khs": ($khs|tonumber), "temp": [], "fan": [], "uptime": 100, "ar": [1, 0]}')
```

#### HiveOS Flight Sheet Configuration
```
+--------------------------------------------------------------------------+
| HIVEOS FLIGHT SHEET INPUTS                                               |
+--------------------------------------------------------------------------+
| • Coin:               ZYN (or Custom ZYAN)                               |
| • Wallet:             %WAL% (Configured Zyanya Bech32 Address)           |
| • Pool:               Configure in miner                                 |
| • Miner:              Custom                                             |
| • Package URL:        https://zyanya.scottcloudhawk.org/releases/hiveos/ |
|                       zyanya-miner-v0.4.0.tar.gz                         |
| • Extra Config Args:  --zyanyad-address YOUR_NODE_IP --port 18110        |
|                       --mining-address %WAL% --threads %THREADS%         |
+--------------------------------------------------------------------------+
```

### 4.2 mmpOS Flight Sheet Setup

In mmpOS, create a Custom CPU profile:

```json
{
  "miner": "zyanya-miner",
  "version": "0.4.0",
  "url": "https://zyanya.scottcloudhawk.org/releases/zyanya-v0.4.0-linux-x86_64.tar.gz",
  "exe": "zyanya-miner",
  "args": "--zyanyad-address %HOST% --port %PORT% --mining-address %WAL% --threads $(nproc) --dynamic",
  "pre_start": "sysctl -w vm.nr_hugepages=$(nproc)"
}
```

---

## 5. Mined Block Verification & Diagnostics

Once your miner discovers a valid Proof-of-Work solution meeting the current network difficulty target, verify block acceptance through stdout logs, gRPC inspection, and the block explorer.

### 5.1 Real-Time Miner Output Analysis

```
[2026-09-09 14:02:11.104] [INFO] [Miner-0] Found PoW Solution! Nonce: 0x3f8a9e1d2c4b5000
[2026-09-09 14:02:11.105] [INFO] [RPC] Submitting Block to Daemon (127.0.0.1:18110)...
[2026-09-09 14:02:11.128] [INFO] [RPC] Block Accepted: Hash=8f3d1c9a7e5b2d4f8a0e1c2d3b4a5f6e7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d
[2026-09-09 14:02:11.129] [INFO] [Miner] Blue Score: 489120 | Reward: 50.00000000 ZYAN
```

### 5.2 Command-Line Diagnostics via `zyanya-query`

Inspect node DAG consensus state and verify your newly minted block:

```bash
# 1. Query Current DAG Tips, DAA Score, and Difficulty
./zyanya-query get-dag-info

# 2. Inspect Block by Hash (Verify Coinbase TX and Blue Work)
./zyanya-query get-block --hash 8f3d1c9a7e5b2d4f8a0e1c2d3b4a5f6e7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d --include-transactions

# 3. Check Sink Blue Score
./zyanya-query get-sink-blue-score
```

### 5.3 Explorer Verification

Verify block status, confirmations, and reward distribution via the official web interfaces:

- **Web Explorer**: `https://zyanya.scottcloudhawk.org`
- **Block Inspector**: `https://zyanya.scottcloudhawk.org/block/<BLOCK_HASH>`
- **Address Balance**: `https://zyanya.scottcloudhawk.org/address/<YOUR_MINING_ADDRESS>`
- **Consensus Stats**: `https://zyanya.scottcloudhawk.org/api/info`

---

## 6. Operational Troubleshooting

```
+--------------------------------------------------------------------------+
| SYMPTOM                | CAUSE                  | RESOLUTION             |
+------------------------+------------------------+------------------------+
| "Connection refused    | Node gRPC listener     | Check zyanyad is       |
| 127.0.0.1:18110"       | inactive or port mismatch | running with --rpclisten |
|                        |                        | on 18110 / 18210       |
|                        |                        |                        |
| Hashrate drops 70%     | Thread count exceeds   | Reduce --threads to    |
| after 5 minutes        | L3 cache / 2MB limit   | L3 Cache (MB) / 2      |
|                        |                        |                        |
| "Daemon is not synced" | Node downloading DAG   | Wait for sync or use   |
| error on start         | headers from peers     | --mine-when-not-synced |
|                        |                        | (testing only)         |
|                        |                        |                        |
| Invalid block rejected | Address network        | Ensure address prefix  |
| by consensus           | prefix mismatch        | matches network        |
|                        |                        | (zyanya: vs zyanyatest:)|
+--------------------------------------------------------------------------+
```
