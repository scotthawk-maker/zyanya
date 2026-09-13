# Zyanya Mining Quickstart (60 Seconds to CPU Mining)

Solo mining on Zyanya requires zero AI agents, zero specialized ASICs, and zero cloud accounts. Anyone with a modern 64-bit CPU can run a sovereign node and mine native ZYAN coins using the AstroBWTv3 proof-of-work algorithm.

---

### 1. Download Official Release Binaries (v1.0.0 / v0.4.0)

Choose the release package for your operating system:

- 🐧 **Linux x86_64 Bundle**:
  • Download: `https://zyanya.scottcloudhawk.org/releases/zyanya-v1.0.0-linux-x86_64.tar.gz`
  • Includes: `zyanyad`, `zyanya-miner`, `zyanya-wallet`, `zyanya-query`, `zyanya-explorer`

- 🪟 **Windows x64 Bundle**:
  • Download: `https://zyanya.scottcloudhawk.org/releases/zyanya-v1.0.0-windows-x64.zip`
  • Includes: `zyanyad.exe`, `zyanya-miner.exe`, `zyanya-wallet.exe`, `zyanya-query.exe`, `zyanya-explorer.exe`

---

### 2. Extract Archive

- **Linux**:
  ```bash
  tar -xzvf zyanya-v1.0.0-linux-x86_64.tar.gz
  cd zyanya-v1.0.0-linux-x86_64
  chmod +x zyanyad zyanya-miner zyanya-wallet zyanya-query zyanya-explorer
  ```

- **Windows**:
  Right-click `zyanya-v1.0.0-windows-x64.zip` and select **Extract All**, then open PowerShell in the extracted folder:
  ```powershell
  cd zyanya-v1.0.0-windows-x64
  ```

---

### 3. Networks: Mainnet vs Testnet

The release binaries are unified and support both networks:

- 🦅 **Mainnet (Launch: October 1, 2026)**:
  • Daemon: `./zyanyad --utxoindex` (RPC: `18110`, P2P: `18111`)
  • Miner: `./zyanya-miner --mining-address zyanya:...`
  • Address prefix: `zyanya:`
  • Payout: 50 ZYAN / block (25 Liquid + 25 Vested for 12 months)

- 🧪 **Testnet (Live Today - Testnet-10)**:
  • Daemon: `./zyanyad --testnet --utxoindex` (RPC: `18210`, P2P: `18211`)
  • Miner: `./zyanya-miner --testnet --mining-address zyanyatest:...`
  • Address prefix: `zyanyatest:`

---

### 4. Generate Your Sovereign Mining Address

Run the native CLI wallet to create a new address:

- **Linux / macOS**:
  ```bash
  # Mainnet
  ./zyanya-wallet new-address

  # Testnet
  ./zyanya-wallet
  $ network testnet-10
  $ new-address
  ```

- **Windows (PowerShell)**:
  ```powershell
  # Mainnet
  .\zyanya-wallet.exe new-address

  # Testnet
  .\zyanya-wallet.exe
  $ network testnet-10
  $ new-address
  ```

Save your generated payout address (`zyanya:...` for mainnet or `zyanyatest:...` for testnet).

---

### 5. Start Your Sovereign Node Daemon

Launch the node with the UTXO index enabled:

- **Linux**:
  ```bash
  # Mainnet
  ./zyanyad --utxoindex

  # Testnet
  ./zyanyad --testnet --utxoindex
  ```

- **Windows (PowerShell)**:
  ```powershell
  # Mainnet
  .\zyanyad.exe --utxoindex

  # Testnet
  .\zyanyad.exe --testnet --utxoindex
  ```

Wait 10 to 30 seconds for your node to connect to the peer network and sync the latest GHOSTDAG blocks.

---

### 6. Launch AstroBWTv3 CPU Miner

Open a second terminal window and point the standalone miner to your local node:

- **Linux (Interactive)**:
  ```bash
  # Mainnet (Allocates 8 threads)
  ./zyanya-miner --threads 8 --mining-address <YOUR_ZYANYA_ADDRESS>

  # Or allocate by percentage of CPU cores (e.g. 50%)
  ./zyanya-miner --cpu-percent 50 --mining-address <YOUR_ZYANYA_ADDRESS>

  # Testnet
  ./zyanya-miner --testnet --threads 8 --mining-address <YOUR_ZYANYATEST_ADDRESS>
  ```

- **Windows (PowerShell)**:
  ```powershell
  # Mainnet (Default: 25% CPU cores)
  .\zyanya-miner.exe --mining-address <YOUR_ZYANYA_ADDRESS>

  # Mainnet (Allocate 8 threads or 50% CPU)
  .\zyanya-miner.exe --threads 8 --mining-address <YOUR_ZYANYA_ADDRESS>
  .\zyanya-miner.exe --cpu-percent 50 --mining-address <YOUR_ZYANYA_ADDRESS>

  # Testnet
  .\zyanya-miner.exe --testnet --threads 8 --mining-address <YOUR_ZYANYATEST_ADDRESS>
  ```

---

### 7. Run Miner as a 24/7 Background Service (Linux systemd)

To keep your miner running continuously in the background on Ubuntu, Debian, Arch, or CachyOS:

1. Create a systemd service file:
   ```bash
   sudo nano /etc/systemd/system/zyanya-miner.service
   ```

2. Paste the following configuration (replace `/path/to` and `<YOUR_ZYANYA_ADDRESS>`):
   ```ini
   [Unit]
   Description=Zyanya AstroBWTv3 CPU Miner
   After=network.target zyanyad.service

   [Service]
   Type=simple
   User=shawn
   WorkingDirectory=/opt/zyanya
   ExecStart=/opt/zyanya/zyanya-miner --mining-address <YOUR_ZYANYA_ADDRESS> --cpu-percent 50 --dynamic
   Restart=always
   RestartSec=10
   LimitNOFILE=65535

   [Install]
   WantedBy=multi-user.target
   ```

3. Enable and start the service:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable --now zyanya-miner
   sudo journalctl -u zyanya-miner -f
   ```

---

### 8. HiveOS & Dedicated Mining Rigs

For headless mining rigs or HiveOS custom miner integration:

- **Miner Command (Mainnet)**:
  ```bash
  ./zyanya-miner --zyanyad-address <NODE_IP> --port 18110 --mining-address <YOUR_ZYANYA_ADDRESS> --threads $(nproc)
  ```

- **Miner Command (Testnet)**:
  ```bash
  ./zyanya-miner --testnet --zyanyad-address <NODE_IP> --port 18210 --mining-address <YOUR_ZYANYATEST_ADDRESS> --threads $(nproc)
  ```

- **Key Parameters**:
  • `--zyanyad-address`: IP of your Zyanya node daemon (default `127.0.0.1`, brackets supported for IPv6: `[2606:...]`).
  • `--port`: gRPC RPC port (`18110` for mainnet, `18210` for testnet).
  • `--mining-address`: Payout address receiving 50 ZYAN block rewards directly on-chain.
  • `--threads`: CPU worker thread count.
  • `--cpu-percent`: Target CPU load percentage (1-100).
  • `--dynamic`: Automatically scale threads based on host system load.
  • `--mine-when-not-synced`: Mine even when the node is synchronizing initial blocks.

---

### 9. Verification & Live Explorer

Track your mined blocks, DAG height, and network difficulty on the live block explorer:

- 🌐 **Web Explorer**: `https://zyanya.scottcloudhawk.org`
- 📊 **Network Info**: `https://zyanya.scottcloudhawk.org/api/info`
- 🤖 **WebMCP Gateway**: `https://zyanya.scottcloudhawk.org/mcp/rpc`
