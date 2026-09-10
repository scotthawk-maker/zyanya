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

### 3. Generate Your Sovereign Mining Address

Run the native CLI wallet to create a new address:

- **Linux**:
  ```bash
  ./zyanya-wallet new-address
  ```

- **Windows**:
  ```powershell
  .\zyanya-wallet.exe new-address
  ```

Save your generated payout address (e.g. `zyanya:qrh5l43xvd05lq36g...`).

---

### 4. Start Your Sovereign Node Daemon

Launch the node with the UTXO index enabled:

- **Linux**:
  ```bash
  ./zyanyad --utxoindex
  ```

- **Windows**:
  ```powershell
  .\zyanyad.exe --utxoindex
  ```

Wait 10 to 30 seconds for your node to connect to the peer network and sync the latest GHOSTDAG blocks.

---

### 5. Launch AstroBWTv3 CPU Miner

Open a second terminal window and point the standalone miner to your local node:

- **Linux**:
  ```bash
  ./zyanya-miner --threads 8 --mining-address <YOUR_ZYANYA_ADDRESS>
  ```

- **Windows**:
  ```powershell
  .\zyanya-miner.exe --threads 8 --mining-address <YOUR_ZYANYA_ADDRESS>
  ```

Replace `8` with the number of CPU threads you wish to allocate. The miner will report your live hashrate and print block acceptance notices when a block solution is discovered.

---

### 6. HiveOS & Bare-Metal Mining Rigs

For headless mining rigs or HiveOS custom miner integration:

- **Miner Command**:
  ```bash
  ./zyanya-miner --rpc-url <NODE_IP>:16110 --mining-address <YOUR_ZYANYA_ADDRESS> --threads $(nproc)
  ```

- **Parameters**:
  • `--rpc-url`: gRPC Borsh RPC endpoint of any public or private Zyanya node (default `127.0.0.1:16110`).
  • `--mining-address`: Payout address receiving block rewards directly on-chain.
  • `--threads`: CPU worker thread count.

---

### 7. Verification & Explorer

Track your mined blocks, block height, and network difficulty on the live block explorer:

- 🌐 **Web Explorer**: `https://zyanya.scottcloudhawk.org`
- 📊 **Network Info**: `https://zyanya.scottcloudhawk.org/api/info`
- 🤖 **WebMCP Gateway**: `https://zyanya.scottcloudhawk.org/mcp/rpc`
