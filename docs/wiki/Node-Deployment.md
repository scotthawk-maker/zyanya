# Sovereign Node Deployment & Pure-IPv6 Architecture

> **Protocol Version**: Zyanya Daemon (`zyanyad`) v0.4.0  
> **Consensus Engine**: GhostDAG (1 BPS) with Subnetwork 3 ZCL Smart Contract VM  
> **Transport Protocol**: Pure IPv6 Global Unicast (TCP)  
> **IPv4 Policy**: Strictly Disabled at Socket and Discovery Layer (`IPV6_V6ONLY`)

---

## 1. System Requirements & Sizing Profiles

Node resource allocation directly dictates block ingestion rate, GhostDAG virtual block resolution latency, and RocksDB cache hit ratios.

```
+--------------------------------------------------------------------------+
| LIGHT / EDGE NODE PROFILE (Validation & Wallet RPC)                     |
+--------------------------------------------------------------------------+
| • Target System:      Edge Compute, SBC (Raspberry Pi 5), Low-End VPS    |
| • CPU:                4 Cores (x86_64 or ARM64)                          |
| • Memory:             4 GB - 8 GB RAM                                    |
| • Storage:            100 GB NVMe / High-Speed SSD                       |
| • Recommended Flags:  --ram-scale=0.3 --nologfiles                       |
+--------------------------------------------------------------------------+

+--------------------------------------------------------------------------+
| STANDARD PRODUCTION NODE (Default Full Node)                             |
+--------------------------------------------------------------------------+
| • Target System:      Dedicated Server / Mining Host / Node Validator    |
| • CPU:                8 - 16 Cores (AMD Zen 3/4/5 or Intel 12th+ Gen)    |
| • Memory:             16 GB - 32 GB RAM                                  |
| • Storage:            500 GB NVMe SSD (PCIe Gen 4)                       |
| • Recommended Flags:  --utxoindex --ram-scale=1.0                        |
+--------------------------------------------------------------------------+

+--------------------------------------------------------------------------+
| HIGH-THROUGHPUT ARCHIVAL & RPC CLUSTER NODE                              |
+--------------------------------------------------------------------------+
| • Target System:      Enterprise Infrastructure / Block Explorer / WebMCP|
| • CPU:                32+ Cores (AMD EPYC or Threadripper)               |
| • Memory:             64 GB - 128 GB ECC RAM                             |
| • Storage:            2 TB+ Enterprise NVMe (U.2/U.3)                    |
| • Recommended Flags:  --utxoindex --archival --ram-scale=3.5             |
|                       --rpclisten-borsh --rpclisten-json                 |
+--------------------------------------------------------------------------+
```

---

## 2. Pure-IPv6 Networking Prerequisites

Zyanya enforces pure IPv6 transport across the entire peer-to-peer mesh. Sockets are opened using `socket2` with `IPV6_V6ONLY = true`.

```
+-----------------------------------------------------------------------+
|                       IPV6 FILTERING INVARIANTS                       |
|                                                                       |
| 1. IPv4-Mapped IPv6 Addresses (::ffff:0:0/96) -> Dropped Immediately   |
| 2. Link-Local Unicast (fe80::/10)             -> Blocked from P2P DB  |
| 3. Loopback Addresses (::1/128)               -> Internal RPC Only    |
| 4. Global Unicast (2000::/3)                  -> Accepted for P2P     |
+-----------------------------------------------------------------------+
```

### 2.1 Host IPv6 Verification

Before starting `zyanyad`, verify that your host has a publicly routable global unicast IPv6 address:

```bash
# Check global IPv6 address assignment (scope global)
ip -6 addr show scope global

# Test IPv6 connectivity to official seed infrastructure
ping6 -c 4 zyanya.scottcloudhawk.org
```

### 2.2 Linux Firewall Configuration (UFW / nftables)

Open the required P2P and RPC listener ports for IPv6 traffic:

```bash
# Mainnet P2P Port (18111)
sudo ufw allow in proto tcp to any port 18111 comment "Zyanya Mainnet P2P"

# Testnet-10 P2P Port (18211)
sudo ufw allow in proto tcp to any port 18211 comment "Zyanya Testnet P2P"

# Localhost Only for gRPC (18110 / 18210) - Do NOT expose raw gRPC globally
sudo ufw allow in proto tcp from ::1 to any port 18110 comment "Zyanya Mainnet gRPC Localhost"
sudo ufw allow in proto tcp from ::1 to any port 18210 comment "Zyanya Testnet gRPC Localhost"
```

### 2.3 Canonical Seed Nodes

If your node is isolated or behind strict edge firewalls, explicitly connect to the seed infrastructure:

- **Canonical Testnet-10 Seed**: `[2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18211`
- **Manual Peering Flag**: `--connect=[2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18211`

---

## 3. Configuration Flags & Daemon Architecture

The `zyanyad` binary exposes deep configuration directives controlling memory consumption, indexers, storage persistence, and multi-protocol RPC interfaces.

```
+--------------------------------------------------------------------------+
| CORE FLAG REFERENCE                                                      |
+--------------------------------------------------------------------------+
| --appdir <PATH>         Custom data directory (RocksDB + logs)           |
| --utxoindex             Enable full in-memory & RocksDB UTXO index       |
| --ram-scale <FLOAT>     Memory scale factor (0.3 for 4GB, 4.0 for 64GB)  |
| --archival              Disable block pruning (retains all DAG history)  |
| --reset-db              Wipe database on boot (required between testnets)|
| --nologfiles            Stream logs to stdout only (disables file I/O)   |
| --perf-metrics          Collect real-time CPU, RAM, and disk metrics     |
+--------------------------------------------------------------------------+

+--------------------------------------------------------------------------+
| NETWORK & RPC LISTENER FLAGS                                             |
+--------------------------------------------------------------------------+
| --listen <[IP]:PORT>        P2P listening socket (default [::]:18111)    |
| --rpclisten <[IP]:PORT>     gRPC RPC server (default 127.0.0.1:18110)    |
| --rpclisten-borsh <ADDR>    wRPC Borsh binary WebSocket (default 19110)  |
| --rpclisten-json <ADDR>     wRPC JSON-RPC / WebMCP WebSocket (def 20110) |
| --unsaferpc                 Enable mutating administrative RPC commands  |
| --outpeers <N>              Target outbound peer connections (def: 8)    |
| --maxinpeers <N>            Max inbound peer connections (default: 128)  |
+--------------------------------------------------------------------------+
```

### 3.1 Memory Tuning via `--ram-scale`

The `--ram-scale` parameter adjusts consensus cache sizes, DAG reachability tables, and RocksDB block cache footprints:

- **Low-Memory Nodes (4 GB - 8 GB RAM)**:
  `--ram-scale=0.3` to `--ram-scale=0.5`  
  Limits memory footprint to $\approx 2.5\text{ GB}$, preventing Linux OOM killer invocation during intensive sync phases.
- **Default Production Nodes (16 GB - 32 GB RAM)**:
  `--ram-scale=1.0` (Default)  
  Standard cache sizing balancing block processing latency and operating system page cache.
- **Enterprise High-Memory Nodes (64 GB - 128 GB+ RAM)**:
  `--ram-scale=3.0` to `--ram-scale=4.0`  
  Retains the entire active DAG frontier and recent transaction index in L1/L2 consensus memory, increasing sync speed for connecting peers by up to $400\%$.

---

## 4. Production Deployment on Linux (systemd)

Run `zyanyad` as an isolated, unprivileged system service with automated recovery, resource limits, and journald log integration.

### 4.1 Create Dedicated Service User & Directory

```bash
# Create system user without interactive shell
sudo useradd -r -s /usr/sbin/nologin -d /var/lib/zyanya zyanya

# Create data directory and assign ownership
sudo mkdir -p /var/lib/zyanya/data
sudo chown -R zyanya:zyanya /var/lib/zyanya
```

### 4.2 Deploy Binary

```bash
# Copy binary to system path
sudo cp zyanyad /usr/local/bin/
sudo chmod 755 /usr/local/bin/zyanyad
```

### 4.3 Configure systemd Service Unit

Create `/etc/systemd/system/zyanyad.service`:

```ini
[Unit]
Description=Zyanya Sovereign GhostDAG Node Daemon
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=zyanya
Group=zyanya
WorkingDirectory=/var/lib/zyanya

ExecStart=/usr/local/bin/zyanyad \
  --testnet \
  --appdir=/var/lib/zyanya/data \
  --utxoindex \
  --ram-scale=1.0 \
  --listen=[::]:18211 \
  --rpclisten=127.0.0.1:18210 \
  --rpclisten-borsh=[::1]:19210 \
  --rpclisten-json=[::1]:20210 \
  --perf-metrics

Restart=always
RestartSec=10
TimeoutStopSec=60
KillMode=process

# Security Hardening
LimitNOFILE=65536
LimitNPROC=32768
ProtectSystem=full
ProtectHome=true
NoNewPrivileges=true
PrivateTmp=true
ProtectKernelTunables=true
ProtectControlGroups=true

[Install]
WantedBy=multi-user.target
```

### 4.4 Enable and Manage Service

```bash
# Reload systemd manager configuration
sudo systemctl daemon-reload

# Enable service across system reboots
sudo systemctl enable zyanyad.service

# Start the node
sudo systemctl start zyanyad.service

# Check real-time status
sudo systemctl status zyanyad.service

# Stream live node logs
sudo journalctl -u zyanyad.service -f -o cat
```

---

## 5. Production Deployment on Windows (PowerShell Daemon)

For Windows Server or Windows 11 host deployments, utilize the automated background PowerShell supervisor script with watchdog restarts.

### 5.1 Directory Preparation

Open PowerShell as Administrator:

```powershell
New-Item -ItemType Directory -Force -Path "C:\Zyanya\Data"
New-Item -ItemType Directory -Force -Path "C:\Zyanya\Logs"
Copy-Item ".\zyanyad.exe" "C:\Zyanya\zyanyad.exe"
```

### 5.2 Create Daemon Watchdog Script (`C:\Zyanya\Start-ZyanyaNode.ps1`)

```powershell
# Zyanya Node Supervisor Script
param (
    [switch]$Mainnet,
    [switch]$Testnet = $true,
    [double]$RamScale = 1.0
)

$DataDir = "C:\Zyanya\Data"
$LogFile = "C:\Zyanya\Logs\zyanyad-stdout.log"
$ExePath = "C:\Zyanya\zyanyad.exe"

$NetArg = if ($Mainnet) { "" } else { "--testnet" }

$Arguments = @(
    $NetArg,
    "--appdir=$DataDir",
    "--utxoindex",
    "--ram-scale=$RamScale",
    "--listen=[::]:18211",
    "--rpclisten=127.0.0.1:18210",
    "--rpclisten-borsh=127.0.0.1:19210",
    "--rpclisten-json=127.0.0.1:20210",
    "--perf-metrics"
) -filter { $_ -ne "" }

Write-Host "[INFO] Starting Zyanya Daemon with arguments: $Arguments" -ForegroundColor Green

while ($true) {
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Add-Content -Path $LogFile -Value "[$Timestamp] [SUPERVISOR] Starting zyanyad process..."
    
    $Process = Start-Process -FilePath $ExePath `
                             -ArgumentList $Arguments `
                             -NoNewWindow `
                             -PassThru `
                             -RedirectStandardOutput $LogFile `
                             -RedirectStandardError $LogFile
    
    $Process.WaitForExit()
    
    $ExitCode = $Process.ExitCode
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Add-Content -Path $LogFile -Value "[$Timestamp] [SUPERVISOR] Process exited with code $ExitCode. Restarting in 5s..."
    Start-Sleep -Seconds 5
}
```

---

## 6. Health Checking & Peering Diagnostics

Verify node synchronization, peer health, and RPC responsiveness using `zyanya-query` and raw HTTP/WebMCP requests.

### 6.1 Diagnostic Inspection via `zyanya-query`

```bash
# 1. General Node & Consensus State Info
./zyanya-query --testnet get-info

# 2. Server Metadata & Sync Status
./zyanya-query --testnet get-server-info

# 3. Block DAG Metrics (Block Count, Tip Count, Virtual DAA Score)
./zyanya-query --testnet get-dag-info

# 4. Connected Peers & IPv6 Addresses
./zyanya-query --testnet get-connected-peer-info

# 5. Check Blue Score Progression
./zyanya-query --testnet get-sink-blue-score
```

### 6.2 Raw JSON-RPC & WebMCP Health Probes

Query the WebMCP discovery endpoint and OpenAPI documentation using `curl -6`:

```bash
# Query WebMCP Discovery Schema
curl -6 https://zyanya.scottcloudhawk.org/mcp.json

# Query OpenAPI 3.1 Specification
curl -6 https://zyanya.scottcloudhawk.org/openapi.json

# Raw JSON-RPC 2.0 Call against WebMCP Gateway
curl -6 -X POST https://zyanya.scottcloudhawk.org/mcp/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "zyanya_get_dag_info",
      "arguments": {}
    }
  }'
```

### 6.3 Diagnostic Metrics & Verification Checklist

```
+--------------------------------------------------------------------------+
| SYNC VERIFICATION CHECKLIST                                              |
+--------------------------------------------------------------------------+
| [✓] is_synced = true (Node has matched network virtual DAA score)       |
| [✓] tip_hashes > 0   (Active GhostDAG frontier received)                 |
| [✓] peer_count >= 8  (Healthy inbound/outbound IPv6 mesh connectivity)   |
| [✓] utxoindex live   (UTXO set synchronized for smart contract calls)   |
| [✓] storage steady   (RocksDB SST compactions operating without latency) |
+--------------------------------------------------------------------------+
```
