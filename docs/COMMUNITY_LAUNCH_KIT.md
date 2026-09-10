# Zyanya Community Launch Kit (Anti-Slop Edition)

Real developer copy for Twitter/X, Discord, and Telegram. No corporate buzzwords, no PR fluff, no AI-generated clichés. Grounded in real architecture, honest developer pain points, and live testnet numbers.

---

## 🐦 Twitter / X Launch Thread

### Tweet 1 (The Frustration Hook)
Most new L1 teams waste 6 months building buggy React wallets and block explorers that break every time Vite bumps a minor version.

We decided that was a waste of time.

Meet Zyanya: a Rust GhostDAG L1 built specifically to be driven by AI coding agents. 🧵👇

---

### Tweet 2 (The Real Problem We Solved)
Frontend lock-in is dead. 

Instead of maintaining 10 different web apps, we stripped out 1.39M lines of frontend bloat and gave the node a native WebMCP (Model Context Protocol) gateway.

What that means in practice:
You connect Claude, Cursor, Antigravity, or a local LLM to your node.
You tell it: "Build me a retro terminal wallet" or "Build me a dark-mode staking chart."
It writes and runs it in 30 seconds.

---

### Tweet 3 (The Raw Engine Numbers)
Under the hood, it's not a toy:

• 1 block per second continuous GhostDAG consensus
• Rust-native engine with strict UTXO accounting
• Sub-second transaction confirmations
• Multi-region testnet live right now:
  - 🇺🇸 US East (local hardware)
  - 🇬🇧 London (96 ms ping to US)
  - 🇯🇵 Tokyo (199 ms ping to US)

---

### Tweet 4 (Open Source & No Gatekeeping)
Everything is open source from day one:

• 28/28 VM tests green
• 76/76 consensus tests passing
• 0 VC allocations, 0 pre-mined insider wallets
• Public seed nodes up and peerable right now

Run `git clone`, compile the daemon, and your node joins the mesh in under 2 minutes.

---

### Tweet 5 (Call to Action)
The future isn't another cookie-cutter web app. It's autonomous agents building whatever tools you need on demand.

Code, node setup scripts, and WebMCP docs are live on GitHub:
🔗 https://github.com/scotthawk-maker/zyanya
📖 Docs: https://github.com/scotthawk-maker/zyanya/tree/main/docs

Drop a star, spin up a node, and tell your agent to start building.

---

## 💬 Discord Announcement (`#announcements`)

```markdown
@everyone **Zyanya is live on GitHub and public testnet**

Quick summary of what we built and why we built it this way:

### What is Zyanya?
Zyanya is a high-speed GhostDAG Layer 1 written in Rust that does ~1.4 blocks per second. 

Instead of building official web wallets or shiny dashboards, we wired the entire node into **WebMCP** (Anthropic's Model Context Protocol). 

That means there is zero official frontend lock-in. When you run a node, you hook your AI assistant (Antigravity, Cursor, Claude, local Ollama) straight into it. If you want a mobile tracker, a terminal wallet, or a Telegram bot, your agent builds it on demand in seconds.

### Network Status
Our initial tri-region testnet mesh is running right now:
• 🇺🇸 US East: Local Dell Micro core (206k+ blocks processed)
• 🇬🇧 London: Vultr Canary Wharf seed (96 ms latency)
• 🇯🇵 Tokyo: Vultr Minamishinagawa seed (199 ms latency)

### Where to start
1. Clone the repo: `https://github.com/scotthawk-maker/zyanya`
2. Check the architecture: Read `docs/WEBMCP.md` and `docs/SMART_CONTRACTS_DESIGN.md`
3. Spin up a node: Follow the one-click scripts in `/deploy`

### Channels
• `#dev-chat`: Node setup, Rust internals, daemon debugging
• `#agent-builds`: Post the tools, UIs, and scripts your AI agent created
• `#general`: Casual chat, ideas, roadmap feedback

No corporate hype here. Jump in, grab the code, and let us know what breaks.
```

---

## ✈️ Telegram Pinned Broadcast

```markdown
📌 **ZYANYA DEV TESTNET & CODEBASE LIVE**

Zyanya is an agent-native GhostDAG L1 written in Rust.

---

🔗 **Links**
• GitHub: https://github.com/scotthawk-maker/zyanya
• WebMCP Protocol Guide: https://github.com/scotthawk-maker/zyanya/blob/main/docs/WEBMCP.md
• Deploy Scripts: https://github.com/scotthawk-maker/zyanya/tree/main/deploy

---

⚡ **Core Facts**
• 1.4 blocks per second continuous throughput
• Sub-second DAG block confirmations
• Native WebMCP gateway: your AI coding assistant builds your UI on demand
• Global seeds live in US, London (96 ms), and Tokyo (199 ms)
• 100% open source (MIT/Apache 2.0)

---

⚠️ **Admin Notice**
We will NEVER DM you first, ask for keys, or sell private allocations. This is an open developer testbed. 

Introduce yourself below and tell us what you're building!
```
