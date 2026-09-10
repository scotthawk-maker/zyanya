# Zyanya Mining Quickstart (60 Seconds to CPU Mining)

Solo mining on Zyanya requires zero AI agents, zero specialized ASICs, and zero cloud accounts. Anyone with a modern 64-bit CPU can run a sovereign node and mine native ZYAN coins using the AstroBWTv3 proof-of-work algorithm.

---

### 1. Download Official Release Binaries (v0.4.0)

Choose the release package for your operating system:

- 🐧 **Linux x86_64 Bundle**:
  • Download: `https://zyanya.scottcloudhawk.org/releases/zyanya-v0.4.0-linux-x86_64.tar.gz`
  • SHA256: `cfa8cbdc75267298613999b907db8ebec7e07148431d181e6c7cfc723f9201b1`
  • Includes: `zyanyad`, `zyanya-miner`, `zyanya-wallet`, `zyanya-query`, `zyanya-explorer`

- 🪟 **Windows x64 Bundle**:
  • Download: `https://zyanya.scottcloudhawk.org/releases/zyanya-v0.4.0-windows-x64.zip`
  • SHA256: `9f62b1bd33d4569b07e7695743f5ec6b0732339332d2355c1a09a2b7b4a3dd19`
  • Includes: `zyanyad.exe`, `zyanya-miner.exe`, `zyanya-wallet.exe`, `zyanya-query.exe`, `zyanya-explorer.exe`

---

### 2. Extract Archive

- **Linux**:
  ```bash
  tar -xzvf zyanya-v0.4.0-linux-x86_64.tar.gz
  cd zyanya-v0.4.0-linux-x86_64
  chmod +x zyanyad zyanya-miner zyanya-wallet zyanya-query zyanya-explorer
  ```

- **Windows**:
  Right-click `zyanya-v0.4.0-windows-x64.zip` and select **Extract All**, then open PowerShell in the extracted folder.

---

### 3. Networks: Mainnet vs Testnet

The release binaries are unified and support both networks:

- 🦅 **Mainnet (Default - October 1 Launch)**:
  • Daemon: `./zyanyad --utxoindex` (RPC: `18110`, P2P: `18111`)
  • Miner: `./zyanya-miner --mining-address zyanya:...`
  • Address prefix: `zyanya:`

- 🧪 **Testnet (Live Today - Testnet-10)**:
  • Daemon: `./zyanyad --testnet --utxoindex` (RPC: `18210`, P2P: `18211`)
  • Miner: `./zyanya-miner --testnet --mining-address zyanyatest:...`
  • Address prefix: `zyanyatest:`

---

### 4. Generate Your Sovereign Mining Address

Run the native CLI wallet to create a new address:

- **Mainnet (Default)**:
  ```bash
  ./zyanya-wallet new-address
  ```

- **Testnet**:
  Launch `./zyanya-wallet`, switch network with `network testnet-10`, and run `new-address`:
  ```bash
  $ network testnet-10
  $ new-address
  ```

Save your generated payout address (`zyanya:...` for mainnet or `zyanyatest:...` for testnet).

---

### 5. Start Your Sovereign Node Daemon

Launch the node with the UTXO index enabled:

- **Mainnet**:
  ```bash
  ./zyanyad --utxoindex
  ```

- **Testnet (Live Network)**:
  ```bash
  ./zyanyad --testnet --utxoindex
  ```

Wait 10 to 30 seconds for your node to connect to the peer network and sync the latest GHOSTDAG blocks.

---

### 6. Launch AstroBWTv3 CPU Miner

Open a second terminal window and point the standalone miner to your local node:

- **Mainnet**:
  ```bash
  ./zyanya-miner --threads 8 --mining-address <YOUR_ZYANYA_ADDRESS>
  ```

- **Testnet**:
  ```bash
  ./zyanya-miner --testnet --threads 8 --mining-address <YOUR_ZYANYATEST_ADDRESS>
  ```

Replace `8` with the number of CPU threads you wish to allocate. The miner will report your live hashrate and print block acceptance notices when a block solution is discovered.

---

### 7. HiveOS & Bare-Metal Mining Rigs

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
  • `--zyanyad-address`: IP of your Zyanya node daemon (default `127.0.0.1`).
  • `--port`: gRPC RPC port (`18110` for mainnet, `18210` for testnet).
  • `--mining-address`: Payout address receiving 50 ZYAN block rewards directly on-chain.
  • `--threads`: CPU worker thread count.
  • `--cpu-percent`: Target CPU load percentage (1-100).
  • `--dynamic`: Automatically scale threads to system load.

---

### 8. Verification & Explorer

Track your mined blocks, block height, and network difficulty on the live block explorer:

- 🌐 **Web Explorer**: `https://zyanya.scottcloudhawk.org`
- 📊 **Network Info**: `https://zyanya.scottcloudhawk.org/api/info`
- 🤖 **WebMCP Gateway**: `https://zyanya.scottcloudhawk.org/mcp/rpc`
