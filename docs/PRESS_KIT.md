# 🏛️ Zyanya Press & Exchange Integration Kit

Official media assets, project boilerplates, and technical integration specifications for centralized exchanges (CEXs), decentralized exchanges (DEXs), market aggregators (CoinGecko, CoinMarketCap), and crypto media outlets.

---

## 📌 Project Identity

- **Official Project Name**: Zyanya
- **Native Asset Ticker**: `ZYAN`
- **Asset Type**: Native Layer 1 Blockchain (UTXO-based GhostDAG)
- **Decimals**: 8 (Smallest unit: `1 Sompi = 0.00000001 ZYAN`)
- **Block Time**: ~1.0 block per second (sub-second confirmations)
- **Primary Repository**: https://github.com/scotthawk-maker/zyanya
- **License**: Open Source (Apache 2.0 / MIT)

---

## 📝 Official Project Boilerplates

### Short Pitch (20 Words)
Zyanya is an ultra-fast, 1-block-per-second GhostDAG Layer 1 in Rust, natively wired with WebMCP for autonomous AI coding agents.

### One-Paragraph Summary (75 Words)
Zyanya is an agent-native Layer 1 blockchain powered by GhostDAG consensus in Rust. Instead of forcing users into brittle, centralized web interfaces, Zyanya features a native Model Context Protocol (WebMCP) gateway. This allows any AI assistant—from Claude and Cursor to Antigravity and local LLMs—to build bespoke wallets, analytics dashboards, and trading tools on demand, while maintaining sub-second UTXO security and high transaction throughput.

### Full Overview (175 Words)
Zyanya eliminates the traditional frontend bottleneck of blockchain development. Built on high-throughput GhostDAG consensus implemented in pure Rust, the network achieves continuous ~1.4 block-per-second block creation with instant UTXO validation. 

Rather than deploying complex, high-maintenance web frontends, Zyanya exposes its entire RPC and state machine directly to AI developer agents via the WebMCP standard. Builders and holders do not need official browser extensions or static React dashboards; they simply prompt their AI assistants to inspect balances, assemble transactions, monitor network health, or deploy smart contracts directly against the node.

The network maintains an active, multi-region distributed backbone with seed relays deployed across US East, Europe (London), and Asia-Pacific (Tokyo). Zyanya is completely open-source, permissionless, and designed from the ground up for the autonomous AI economy.

---

## 🎨 Official Brand Assets & Palette

All production-ready SVG vectors and raster PNG files are hosted directly in the repository under `/brand/`:

### Primary Visual Assets
- **Token Icon (Square/Coin)**:
  • Vector: [`brand/zyan-coin.svg`](file:///C:/Users/Shawn/zyanya-build/rusty-spectre-git/brand/zyan-coin.svg)  
  • Raster: [`brand/zyan-coin.png`](file:///C:/Users/Shawn/zyanya-build/rusty-spectre-git/brand/zyan-coin.png) (240x240, 512x512)
- **Horizontal Logo / Wordmark**:
  • Vector: [`brand/zyanya-logo.svg`](file:///C:/Users/Shawn/zyanya-build/rusty-spectre-git/brand/zyanya-logo.svg)  
  • Raster: [`brand/zyanya-logo.png`](file:///C:/Users/Shawn/zyanya-build/rusty-spectre-git/brand/zyanya-logo.png)
- **Social / Header Banner (1200x630)**:
  • Vector: [`brand/zyanya-hero-banner.svg`](file:///C:/Users/Shawn/zyanya-build/rusty-spectre-git/brand/zyanya-hero-banner.svg)  
  • Raster: [`brand/zyanya-hero-banner.png`](file:///C:/Users/Shawn/zyanya-build/rusty-spectre-git/brand/zyanya-hero-banner.png)
- **Mobile / App Squircle Icon**:
  • Vector: [`brand/zyn-squircle.svg`](file:///C:/Users/Shawn/zyanya-build/rusty-spectre-git/brand/zyn-squircle.svg)

### Official Color Codes
- **Spectral Cyan (Primary Accent)**: `#7EC8D3` | `rgb(126, 200, 211)`
- **Abyssal Blue (Gradient Mid)**: `#0D3B50` | `rgb(13, 59, 80)`
- **Midnight Void (Background)**: `#0A0F1C` | `rgb(10, 15, 28)`
- **Typeface**: `Fira Code` / `Monospace` (Roman weights only, no faux italics)

---

## ⚙️ Exchange Listing & Node Integration Specifications

Centralized exchanges and custody providers can use these technical parameters for wallet integration:

### 1. Address Format
- **Encoding**: Bech32 standard
- **Mainnet Prefix**: `zyanya:<payload>`
- **Testnet Prefix**: `zyanyatest:<payload>`
- **Checksum**: BCH-based polymod checksum (standard Bech32 validation)

### 2. Node Daemon & RPC Ports
- **Daemon Binary**: `zyanyad` (compiled Rust)
- **Query Tool**: `zyanya-query` (lightweight CLI query client)
- **Port Mapping**:
  • **P2P Gossip**: `18111` (Mainnet) | `18211` (Testnet)  
  • **gRPC Protobuf**: `18110` (Mainnet) | `18210` (Testnet)  
  • **wRPC (Borsh/WebSocket)**: `19110` (Mainnet) | `19210` (Testnet)  
  • **WebMCP Gateway**: `8055`

### 3. Recommended Exchange Node Specs
- **CPU**: 4 vCPUs / physical cores (AMD EPYC or Intel Xeon)
- **RAM**: 8 GB minimum (16 GB recommended)
- **Storage**: 100 GB NVMe SSD (fast random read/write for UTXO index)
- **Network**: 100 Mbps uplink minimum
- **OS**: Ubuntu 24.04 LTS x64 / Debian 12

### 4. Cold Storage & Custody Mechanics
- **UTXO Engine**: Pure UTXO accounting similar to Bitcoin/Kaspa. Standard multi-input, change-output transaction structure.
- **Schnorr Signatures**: High-efficiency cryptographic signing on secp256k1 curves.
- **Deposit Confirmations**: Recommended 10 to 20 blocks (~10 to 20 seconds) for irreversible finality on the DAG mergeset.

---

## 🔗 Official Verification Links

- **Source Code**: https://github.com/scotthawk-maker/zyanya
- **WebMCP Documentation**: https://github.com/scotthawk-maker/zyanya/blob/main/docs/WEBMCP.md
- **Smart Contracts & VM**: https://github.com/scotthawk-maker/zyanya/blob/main/docs/SMART_CONTRACTS_DESIGN.md
- **Community Launch Kit**: https://github.com/scotthawk-maker/zyanya/blob/main/docs/COMMUNITY_LAUNCH_KIT.md
