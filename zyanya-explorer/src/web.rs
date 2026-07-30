pub const LOGO_SVG: &str = include_str!("/home/shawn/projects/zyanya-distro/brand/zyanya-logo.svg");
pub const HERO_BANNER_SVG: &str = include_str!("/home/shawn/projects/zyanya-distro/brand/zyanya-hero-banner.svg");
pub const ZYAN_COIN_SVG: &str = include_str!("/home/shawn/projects/zyanya-distro/brand/zyan-coin.svg");
pub const GHOST_TOKEN_SVG: &str = include_str!("/home/shawn/projects/zyanya-distro/brand/ghost-token.svg");
pub const GAS_BURN_SVG: &str = include_str!("/home/shawn/projects/zyanya-distro/brand/gas-burn-icon.svg");
pub const TOKEN_SET_SVG: &str = include_str!("/home/shawn/projects/zyanya-distro/brand/zyanya-token-set.svg");

pub const LANDING_HTML: &str = r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Zyanya: The Ghost in the IPv6 Machine</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;600&display=swap" rel="stylesheet">
    <style>
        :root {
            --void: #0A0F1C;
            --shadow-teal: #0D3B50;
            --spectral-blue: #7EC8D3;
            --text-color: #E0E0E0;
            --burn-red: #FF4D4D;
            --font-mono: 'Fira Code', monospace;
        }

        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }

        html, body {
            background-color: var(--void);
            color: var(--text-color);
            font-family: var(--font-mono);
            font-size: 16px;
            line-height: 1.6;
            overflow-x: hidden;
        }

        .grid-bg {
            position: fixed;
            top: 0;
            left: 0;
            width: 100vw;
            height: 100vh;
            background-image:
                linear-gradient(to right, rgba(13, 59, 80, 0.3) 1px, transparent 1px),
                linear-gradient(to bottom, rgba(13, 59, 80, 0.3) 1px, transparent 1px);
            background-size: 40px 40px;
            z-index: -2;
        }

        .grid-bg::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: radial-gradient(ellipse at center, rgba(13, 59, 80, 0.2), var(--void) 70%);
            z-index: -1;
        }

        .container {
            max-width: 1100px;
            margin: 0 auto;
            padding: 0 20px;
        }

        header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 2rem 0;
            border-bottom: 1px solid var(--shadow-teal);
        }

        #logo-container svg {
            height: 40px;
            width: auto;
        }

        nav a {
            color: var(--spectral-blue);
            text-decoration: none;
            margin-left: 2rem;
            font-weight: 600;
            transition: color 0.3s ease;
        }

        nav a:hover {
            color: var(--text-color);
            text-shadow: 0 0 5px var(--spectral-blue);
        }

        main {
            padding: 4rem 0;
        }

        section {
            margin-bottom: 6rem;
            text-align: center;
        }

        #hero {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 60vh;
        }

        #banner-container {
            width: 100%;
            max-width: 600px;
            margin-bottom: 2rem;
        }

        #banner-container svg {
            width: 100%;
            height: auto;
        }

        h1 {
            font-size: 1.8rem;
            font-weight: 600;
            margin-bottom: 0.5rem;
            color: var(--spectral-blue);
        }

        #hero p {
            font-size: 1.1rem;
            max-width: 600px;
            margin-bottom: 2.5rem;
        }

        .cta-buttons {
            display: flex;
            gap: 1.5rem;
            justify-content: center;
        }

        .btn {
            display: inline-block;
            padding: 0.8rem 1.8rem;
            text-decoration: none;
            border-radius: 4px;
            font-weight: 600;
            transition: all 0.3s ease;
            border: 2px solid transparent;
        }

        .btn-primary {
            background-color: var(--spectral-blue);
            color: var(--void);
        }

        .btn-primary:hover {
            background-color: transparent;
            color: var(--spectral-blue);
            border-color: var(--spectral-blue);
            box-shadow: 0 0 10px var(--spectral-blue);
        }

        .btn-secondary {
            background-color: transparent;
            color: var(--text-color);
            border: 2px solid var(--shadow-teal);
        }

        .btn-secondary:hover {
            background-color: var(--shadow-teal);
            color: var(--text-color);
            box-shadow: 0 0 10px var(--shadow-teal);
        }
        
        #status-banner {
            margin-top: -2rem;
            margin-bottom: 4rem;
            padding: 1rem 1.5rem;
            background-color: rgba(13, 59, 80, 0.5);
            border: 1px solid var(--shadow-teal);
            border-radius: 8px;
            display: inline-block;
            font-size: 1rem;
        }

        #status-banner a {
            color: var(--spectral-blue);
            font-weight: 600;
            text-decoration: none;
        }
        
        #status-banner a:hover {
            text-decoration: underline;
        }

        h2 {
            font-size: 2.5rem;
            margin-bottom: 3rem;
            color: var(--text-color);
            font-weight: 400;
            text-transform: uppercase;
            letter-spacing: 2px;
        }

        .grid-3 {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 2.5rem;
            text-align: left;
        }

        .card {
            background-color: var(--shadow-teal);
            padding: 2.5rem;
            border-radius: 8px;
            border: 1px solid rgba(126, 200, 211, 0.2);
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }

        .card:hover {
            transform: translateY(-5px);
            box-shadow: 0 10px 20px rgba(0, 0, 0, 0.2);
        }
        
        .card h3 {
            font-size: 1.2rem;
            margin-bottom: 1rem;
            color: var(--spectral-blue);
        }

        .icon-container svg {
            height: 32px;
            width: 32px;
            margin-right: 1rem;
            fill: var(--spectral-blue);
        }
        
        #economics .icon-container {
            display: flex;
            align-items: center;
            margin-bottom: 1rem;
        }
        
        #economics h3 { margin-bottom: 0; }
        
        .code-block {
            background-color: var(--void);
            color: var(--text-color);
            padding: 1.5rem;
            border-radius: 4px;
            text-align: left;
            overflow-x: auto;
            border: 1px solid var(--shadow-teal);
            margin: 1.5rem 0;
        }

        code {
            font-family: var(--font-mono);
        }

        #join p {
            max-width: 800px;
            margin: 0 auto 2rem auto;
        }

        #join a {
            color: var(--spectral-blue);
        }

        #roadmap ol {
            list-style: none;
            counter-reset: phase-counter;
            padding: 0;
            max-width: 800px;
            margin: 0 auto;
            text-align: left;
        }
        
        #roadmap li {
            counter-increment: phase-counter;
            position: relative;
            padding: 1.5rem 1.5rem 1.5rem 4rem;
            margin-bottom: 1.5rem;
            background-color: var(--shadow-teal);
            border-radius: 8px;
            border-left: 3px solid var(--spectral-blue);
        }

        #roadmap li::before {
            content: "Phase " counter(phase-counter, decimal-leading-zero);
            position: absolute;
            left: -15px;
            top: 50%;
            transform: translateY(-50%) rotate(-90deg);
            color: var(--spectral-blue);
            font-weight: 600;
            font-size: 0.8rem;
            letter-spacing: 1px;
        }

        #roadmap strong {
            display: block;
            color: var(--text-color);
            font-size: 1.1rem;
            margin-bottom: 0.5rem;
        }

        footer {
            text-align: center;
            padding: 3rem 0;
            margin-top: 4rem;
            border-top: 1px solid var(--shadow-teal);
            font-size: 0.9rem;
            color: rgba(224, 224, 224, 0.6);
        }
        
        .burn-red { color: var(--burn-red); }
    </style>
</head>
<body>
    <div class="grid-bg"></div>
    <div class="container">
        <header>
            <div id="logo-container"><!-- SVG will be injected here --></div>
            <nav>
                <a href="#pillars">Pillars</a>
                <a href="#economics">Economics</a>
                <a href="#join">Join Testnet</a>
                <a href="https://testnet.zyanya.scottcloudhawk.org/" target="_blank">Explorer</a>
                <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" class="mono" style="color: var(--accent-spectral);">GitHub</a>
            </nav>
        </header>

        <main>
            <section id="hero">
                <div id="banner-container"><!-- SVG will be injected here --></div>
                <h1>The ghost in the IPv6 machine.</h1>
                <p>Zyanya is an IPv6-native, agent-native blockchain built on Spectre/GhostDAG. No gateways. No translators. Just pure, end-to-end decentralized consensus over the next-generation internet.</p>
                <div class="cta-buttons">
                    <a href="https://testnet.zyanya.scottcloudhawk.org/" target="_blank" class="btn btn-primary">LAUNCH EXPLORER</a>
                    <a href="#join" class="btn btn-secondary">JOIN THE TESTNET</a>
                </div>
            </section>

            <section id="status-banner">
                <p>🟣 Public testnet is LIVE &mdash; 3 nodes, 15,000+ blocks, and mining over IPv6. <a href="https://testnet.zyanya.scottcloudhawk.org/" target="_blank">Explore the testnet &rarr;</a></p>
            </section>

            <section id="pillars">
                <h2>THE THREE PILLARS</h2>
                <div class="grid-3">
                    <div class="card">
                        <h3>I. THE GHOST</h3>
                        <p>Built on Spectre, a blockDAG protocol. Achieves high throughput and low confirmation times without sacrificing decentralization. Blocks are never orphaned; they are woven into the directed acyclic graph of the ledger.</p>
                    </div>
                    <div class="card">
                        <h3>II. THE SECRET</h3>
                        <p>IPv6-native. The protocol speaks IPv6 from the ground up, shedding the legacy constraints of IPv4. This is a commitment to the future of the internet, unlocking a vast, un-NAT-ed address space for true peer-to-peer communication.</p>
                    </div>
                    <div class="card">
                        <h3>III. THE FOREVER</h3>
                        <p>Designed for longevity. A slow-burn emission schedule with time-locked vesting for the foundation, combined with a permanent 50% gas fee burn, creates a deflationary, sustainable economic model for the long term.</p>
                    </div>
                </div>
            </section>
            
            <section id="economics">
                <h2>ECONOMICS</h2>
                <div class="grid-3">
                     <div class="card">
                        <div class="icon-container" id="icon-coin"><h3>Total Supply</h3></div>
                        <p>2.1 billion ZYA. A fixed supply, ensuring predictable scarcity. No pre-mine. The genesis block starts a fair launch for all participants.</p>
                    </div>
                     <div class="card">
                        <div class="icon-container" id="icon-token"><h3>Vesting Schedule</h3></div>
                        <p>A 10% foundation allocation is locked in a 10-year linear vesting contract. This aligns long-term incentives and ensures sustained development.</p>
                    </div>
                     <div class="card">
                        <div class="icon-container" id="icon-burn"><h3><span class="burn-red">The Burn</span></h3></div>
                        <p>50% of all transaction fees are permanently burned. This deflationary pressure rewards long-term holders and increases the network's value over time.</p>
                    </div>
                </div>
            </section>

            <section id="join">
                <h2>HOW TO JOIN THE TESTNET</h2>
                <p>An IPv6-enabled connection is required. Download the latest distribution (Docker image, Windows binaries, and README) from the seed node to get started.</p>
                <p><a href="/distro/" target="_blank">Download Distribution Here</a></p>
                
                <h4>1. Run a Full Node & Mine</h4>
                <p>Use the `zyanyad` daemon. The `--connect` flag points to the seed node, and `--enable-unsynced-mining` lets you start mining immediately.</p>
                <div class="code-block">
                    <code>zyanyad --testnet --connect=[2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18211 --enable-unsynced-mining</code>
                </div>

                <h4>2. Query the Network</h4>
                <p>Use the `zyanya-query` tool to interact with your node's RPC server. The testnet seed also runs a public RPC endpoint.</p>
                <div class="code-block">
                    <code>zyanya-query --testnet --rpcserver [2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18210 get-dag-info</code>
                </div>
            </section>

            <section id="roadmap">
                <h2>THE PATH FORWARD</h2>
                <ol>
                    <li>
                        <strong>Phase 01: Ghost in the Machine</strong>
                        <span>Public testnet hardening. Protocol improvements, bug fixes, and network stability testing with the community.</span>
                    </li>
                    <li>
                        <strong>Phase 02: Dark Launch</strong>
                        <span>Mainnet genesis block is mined and the network is deployed silently. Initial stability monitoring by the core team.</span>
                    </li>
                    <li>
                        <strong>Phase 03: Prepare Optics</strong>
                        <span>Finalize public documentation, exchange integrations, and communication materials. Prepare for the public reveal.</span>
                    </li>
                    <li>
                        <strong>Phase 04: The r/IPv6 Signal</strong>
                        <span>Public announcement and invitation to the wider technical community, starting with the IPv6 pioneers.</span>
                    </li>
                </ol>
            </section>

        </main>

        <footer>
            <p>The ghost in the IPv6 machine. Forever, always.</p>
            <p>&copy; 2024 Zyanya Project. All rights reserved. &bull; <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" style="color: var(--accent-spectral); text-decoration: none;">Source on GitHub</a></p>
        </footer>
    </div>

    <script>
        // Fetches brand SVGs and injects them into the page.
        // This keeps the HTML clean and allows for easy SVG management.
        document.addEventListener('DOMContentLoaded', () => {
            const fetchAndInject = (id, url) => {
                const container = document.getElementById(id);
                if (!container) return;
                fetch(url)
                    .then(response => response.text())
                    .then(svgText => {
                        container.innerHTML = svgText;
                    })
                    .catch(error => console.error(`Failed to load SVG ${url}:`, error));
            };

            fetchAndInject('logo-container', '/brand/zyanya-logo.svg');
            fetchAndInject('banner-container', '/brand/zyanya-hero-banner.svg');
            fetchAndInject('icon-coin', '/brand/icon-coin.svg');
            fetchAndInject('icon-token', '/brand/icon-token.svg');
            fetchAndInject('icon-burn', '/brand/icon-burn.svg');
        });
    </script>
    <script src="/webmcp.js"></script>
</body>
</html>
"###;

pub const EXPLORER_HTML: &str = r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Zyanya Block Explorer</title>
    <style>
        :root {
            --bg-base: #0A0F1C;
            --bg-shadow: #0D3B50;
            --accent-spectral: #7EC8D3;
            --text-main: #E0E0E0;
            --burn-red: #FF4D4D;
        }

        * { box-sizing: border-box; margin: 0; padding: 0; }

        body {
            background-color: var(--bg-base);
            color: var(--text-main);
            font-family: 'Segoe UI', system-ui, sans-serif;
            line-height: 1.5;
        }

        .mono { font-family: 'Courier New', Courier, monospace; }

        header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 1.2rem 2.5rem;
            border-bottom: 1px solid rgba(126, 200, 211, 0.2);
            background: rgba(10, 15, 28, 0.95);
            backdrop-filter: blur(10px);
            position: sticky;
            top: 0;
            z-index: 100;
        }

        .logo-wrap { display: flex; align-items: center; gap: 1rem; text-decoration: none; }
        .logo-wrap svg { height: 32px; width: auto; }

        nav { display: flex; gap: 1.5rem; }
        .nav-btn {
            background: transparent;
            border: 1px solid transparent;
            color: var(--text-main);
            padding: 0.5rem 1rem;
            border-radius: 4px;
            cursor: pointer;
            font-size: 0.9rem;
            letter-spacing: 1px;
            transition: all 0.2s;
        }
        .nav-btn:hover, .nav-btn.active {
            border-color: var(--accent-spectral);
            color: var(--accent-spectral);
            background: rgba(13, 59, 80, 0.4);
            box-shadow: 0 0 10px rgba(126, 200, 211, 0.2);
        }

        .container { max-width: 1300px; margin: 0 auto; padding: 2rem; }

        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1.25rem;
            margin-bottom: 2rem;
        }

        .stat-card {
            background: rgba(13, 59, 80, 0.3);
            border: 1px solid rgba(126, 200, 211, 0.2);
            border-radius: 6px;
            padding: 1.25rem;
            backdrop-filter: blur(4px);
        }

        .stat-label { font-size: 0.75rem; color: #90A0B0; letter-spacing: 1px; margin-bottom: 0.4rem; }
        .stat-value { font-size: 1.4rem; font-weight: 700; color: var(--accent-spectral); }

        .card {
            background: rgba(13, 59, 80, 0.25);
            border: 1px solid rgba(126, 200, 211, 0.2);
            border-radius: 6px;
            padding: 1.5rem;
            margin-bottom: 2rem;
        }

        .card-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1.25rem;
            border-bottom: 1px solid rgba(126, 200, 211, 0.15);
            padding-bottom: 0.75rem;
        }

        .card-title { font-size: 1.1rem; color: var(--accent-spectral); letter-spacing: 2px; }

        table { width: 100%; border-collapse: collapse; text-align: left; }
        th, td { padding: 0.85rem 1rem; border-bottom: 1px solid rgba(126, 200, 211, 0.1); font-size: 0.9rem; }
        th { color: var(--accent-spectral); font-size: 0.8rem; letter-spacing: 1px; }
        tr:hover { background: rgba(126, 200, 211, 0.05); }
        .new-block { animation: newBlockFlash 1.6s ease-out; }
        @keyframes newBlockFlash {
            0% { background-color: rgba(126, 200, 211, 0.45); }
            70% { background-color: rgba(126, 200, 211, 0.12); }
            100% { background-color: transparent; }
        }

        a.link { color: var(--accent-spectral); text-decoration: none; }
        a.link:hover { text-decoration: underline; }

        .tag {
            display: inline-block;
            padding: 0.15rem 0.5rem;
            border-radius: 3px;
            font-size: 0.75rem;
            font-weight: 600;
        }
        .tag-liquid { background: rgba(126, 200, 211, 0.2); color: var(--accent-spectral); border: 1px solid var(--accent-spectral); }
        .tag-vested { background: rgba(255, 255, 255, 0.1); color: #E0E0E0; border: 1px solid #708090; }

        .search-box {
            display: flex;
            gap: 0.5rem;
            margin-bottom: 2rem;
        }
        .search-input {
            flex: 1;
            background: #050810;
            border: 1px solid rgba(126, 200, 211, 0.3);
            color: var(--text-main);
            padding: 0.75rem 1rem;
            border-radius: 4px;
            font-family: inherit;
        }
        .search-btn {
            background: var(--bg-shadow);
            border: 1px solid var(--accent-spectral);
            color: var(--accent-spectral);
            padding: 0.75rem 1.5rem;
            border-radius: 4px;
            cursor: pointer;
            font-weight: 600;
        }

        .tab-content { display: none; }
        .tab-content.active { display: block; }

        #dag-svg {
            width: 100%;
            height: 380px;
            background: #050810;
            border: 1px solid rgba(126, 200, 211, 0.2);
            border-radius: 6px;
        }
    </style>
</head>
<body>
    <header>
        <a href="/" class="logo-wrap">
            <div id="explorer-logo"></div>
            <span class="mono" style="font-size: 1.1rem; color: var(--accent-spectral); letter-spacing: 2px;">EXPLORER</span>
        </a>
        <nav>
            <button class="nav-btn mono active" onclick="switchTab('dashboard')">DASHBOARD</button>
            <button class="nav-btn mono" onclick="switchTab('contracts')">CONTRACTS</button>
            <button class="nav-btn mono" onclick="switchTab('tokens')">TOKENS</button>
            <button class="nav-btn mono" onclick="switchTab('dex')">DEX</button>
            <button class="nav-btn mono" onclick="switchTab('dag')">DAG GRAPH</button>
            <a href="/tools" class="nav-btn mono" style="text-decoration: none;">WEBMCP TOOLS</a>
            <a href="/" class="nav-btn mono" style="text-decoration: none;">WEBSITE</a>
            <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" class="nav-btn mono" style="text-decoration: none; color: var(--accent-spectral);">GITHUB</a>
        </nav>
    </header>

    <main class="container">
        <div class="search-box">
            <input type="text" id="search-input" class="search-input mono" placeholder="Search by Block Hash or Contract Address...">
            <button class="search-btn mono" onclick="performSearch()">SEARCH</button>
        </div>

        <div id="tab-dashboard" class="tab-content active">
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-label mono">BLOCK COUNT</div>
                    <div class="stat-value mono" id="stat-blocks">---</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label mono">VIRTUAL DAA SCORE</div>
                    <div class="stat-value mono" id="stat-daa">---</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label mono">DIFFICULTY</div>
                    <div class="stat-value mono" id="stat-diff">---</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label mono">CIRCULATING ZYAN</div>
                    <div class="stat-value mono" id="stat-supply">---</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label mono">CONNECTED PEERS</div>
                    <div class="stat-value mono" id="stat-peers">---</div>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <h3 class="card-title mono">RECENT BLOCKS</h3>
                    <span class="mono" style="font-size: 0.8rem; color: #708090;">Auto-refreshing live chain &bull; last updated <span id="last-updated">&hellip;</span></span>
                </div>
                <table>
                    <thead>
                        <tr>
                            <th class="mono">BLOCK HASH</th>
                            <th class="mono">BLUE SCORE</th>
                            <th class="mono">DAA SCORE</th>
                            <th class="mono">TIMESTAMP</th>
                            <th class="mono">TXS</th>
                            <th class="mono">SELECTED PARENT</th>
                        </tr>
                    </thead>
                    <tbody id="blocks-tbody">
                        <tr><td colspan="6" style="text-align:center;">Loading recent blocks...</td></tr>
                    </tbody>
                </table>
            </div>
        </div>

        <div id="tab-contracts" class="tab-content">
            <div class="card">
                <div class="card-header">
                    <h3 class="card-title mono">DEPLOYED SMART CONTRACTS (ZCL VM)</h3>
                    <span class="mono" style="font-size: 0.8rem; color: #708090;">Auto-indexed on-chain</span>
                </div>
                <p style="color: #A0B0BC; margin-bottom: 1rem;">
                    Contracts deployed on Zyanya feature 50% gas burn deflationary mechanics, bytecode inspection, and key storage.
                </p>
                <div id="contracts-container">
                    <table>
                        <thead>
                            <tr>
                                <th class="mono">CONTRACT ADDRESS</th>
                                <th class="mono">TYPE</th>
                                <th class="mono">BYTECODE SIZE</th>
                                <th class="mono">STATUS</th>
                                <th class="mono">ACTIONS</th>
                            </tr>
                        </thead>
                        <tbody id="contracts-tbody">
                            <tr><td colspan="5" style="text-align:center;">Loading deployed contracts...</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>
            <div class="card" style="background:#050810; margin-top: 1.5rem;">
                <h4 class="mono" style="color:var(--accent-spectral);">Query Contract Storage State Inspector</h4>
                <div style="display:flex; gap:0.5rem; margin-top:0.75rem;">
                    <input type="text" id="contract-addr-input" class="search-input mono" placeholder="Contract Address...">
                    <input type="text" id="contract-key-input" class="search-input mono" style="max-width:150px;" placeholder="Key (e.g. 0)">
                    <button class="search-btn mono" onclick="queryContractState()">QUERY STATE</button>
                </div>
                <div id="contract-query-result" style="margin-top:1rem;" class="mono"></div>
            </div>
        </div>

        <div id="tab-tokens" class="tab-content">
            <div class="card">
                <div class="card-header">
                    <h3 class="card-title mono">NATIVE CUSTOM TOKENS</h3>
                    <span class="mono" style="font-size: 0.8rem; color: #708090;">ZCL VM Token Standard</span>
                </div>
                <p style="color: #A0B0BC; margin-bottom: 1rem;">
                    ERC-20 style custom tokens deployed on-chain with total supply and key holder balances.
                </p>
                <div id="tokens-container">
                    <table>
                        <thead>
                            <tr>
                                <th class="mono">TOKEN ADDRESS</th>
                                <th class="mono">NAME / SYMBOL</th>
                                <th class="mono">TOTAL SUPPLY</th>
                                <th class="mono">BYTECODE SIZE</th>
                                <th class="mono">ACTIONS</th>
                            </tr>
                        </thead>
                        <tbody id="tokens-tbody">
                            <tr><td colspan="5" style="text-align:center;">Loading tokens...</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>

        <div id="tab-dex" class="tab-content">
            <div class="card">
                <div class="card-header">
                    <h3 class="card-title mono">ON-CHAIN DEX AMM POOLS</h3>
                    <span class="mono" style="font-size: 0.8rem; color: #708090;">Constant Product AMM</span>
                </div>
                <p style="color: #A0B0BC; margin-bottom: 1rem;">
                    Automated market maker pools pairing ZYAN with custom tokens (GHOST). Reserves stored at Key 0 (ZYAN) and Key 1 (GHOST).
                </p>
                <div id="dex-container">
                    <table>
                        <thead>
                            <tr>
                                <th class="mono">DEX POOL ADDRESS</th>
                                <th class="mono">RESERVE A (ZYAN)</th>
                                <th class="mono">RESERVE B (GHOST)</th>
                                <th class="mono">TOTAL LP SUPPLY</th>
                                <th class="mono">IMPLIED PRICE</th>
                                <th class="mono">ACTIONS</th>
                            </tr>
                        </thead>
                        <tbody id="dex-tbody">
                            <tr><td colspan="6" style="text-align:center;">Loading DEX pools...</td></tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>

        <div id="tab-dag" class="tab-content">
            <div class="card">
                <div class="card-header">
                    <h3 class="card-title mono">GHOSTDAG GRAPH VISUALIZATION</h3>
                </div>
                <p style="color: #A0B0BC; margin-bottom: 1rem;">
                    Visual node-link graph showing recent parallel block DAG structure and parent edges.
                </p>
                <svg id="dag-svg"></svg>
            </div>
        </div>

        <div id="detail-modal" class="card" style="display: none;">
            <div class="card-header">
                <h3 class="card-title mono" id="detail-title">BLOCK DETAILS</h3>
                <button class="nav-btn mono" onclick="closeDetail()">CLOSE ✕</button>
            </div>
            <div id="detail-body"></div>
        </div>
    </main>

    <script>
        fetch('/brand/zyanya-logo.svg').then(r => r.text()).then(html => {
            document.getElementById('explorer-logo').innerHTML = html;
        });

        function switchTab(name) {
            document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
            document.querySelectorAll('.nav-btn').forEach(el => el.classList.remove('active'));
            document.getElementById('tab-' + name).classList.add('active');
            if (window.event && window.event.target) window.event.target.classList.add('active');
            closeDetail();

            if (name === 'contracts') loadContracts();
            if (name === 'tokens') loadTokens();
            if (name === 'dex') loadDex();
            if (name === 'dag') loadDag();
        }

        let seenBlocks = new Set();
        let firstDashboardLoad = true;

        async function loadDashboard() {
            try {
                const infoRes = await fetch('/api/info');
                const info = await infoRes.json();
                document.getElementById('stat-blocks').innerText = info.block_count;
                document.getElementById('stat-daa').innerText = info.virtual_daa_score;
                document.getElementById('stat-diff').innerText = info.difficulty.toFixed(2);
                document.getElementById('stat-supply').innerText = info.coin_supply_zyan.toLocaleString() + ' ZYAN';
                document.getElementById('stat-peers').innerText = info.peer_count;

                const blocksRes = await fetch('/api/blocks');
                const blocks = await blocksRes.json();
                let html = '';
                blocks.forEach(b => {
                    const shortHash = b.hash.substring(0, 12) + '...' + b.hash.substring(b.hash.length - 8);
                    const shortParent = b.selected_parent ? (b.selected_parent.substring(0, 10) + '...') : 'Genesis';
                    const timeStr = new Date(b.timestamp).toLocaleTimeString();
                    const isNew = !firstDashboardLoad && !seenBlocks.has(b.hash);
                    html += '<tr' + (isNew ? ' class="new-block"' : '') + '>' +
                        '<td><a href="#" class="link mono" onclick="viewBlock(\'' + b.hash + '\')">' + shortHash + '</a></td>' +
                        '<td class="mono">' + b.blue_score + '</td>' +
                        '<td class="mono">' + b.daa_score + '</td>' +
                        '<td class="mono">' + timeStr + '</td>' +
                        '<td class="mono">' + b.tx_count + '</td>' +
                        '<td class="mono"><a href="#" class="link" onclick="viewBlock(\'' + b.selected_parent + '\')">' + shortParent + '</a></td>' +
                    '</tr>';
                    seenBlocks.add(b.hash);
                });
                firstDashboardLoad = false;
                document.getElementById('blocks-tbody').innerHTML = html;
                const lu = document.getElementById('last-updated');
                if (lu) lu.innerText = new Date().toLocaleTimeString();
            } catch (err) {
                console.error(err);
            }
        }

        async function viewBlock(hash) {
            if (!hash || hash.startsWith('0000000000000')) return;
            try {
                const res = await fetch('/api/block/' + hash);
                const data = await res.json();

                document.getElementById('detail-title').innerText = 'BLOCK DETAIL: ' + data.hash.substring(0, 16) + '...';
                
                let outputsHtml = '';
                data.coinbase_vesting_outputs.forEach(o => {
                    const tag = o.is_liquid ? '<span class="tag tag-liquid">50% LIQUID</span>' : '<span class="tag tag-vested">VESTED (' + o.lock_months + 'M CSV LOCK)</span>';
                    outputsHtml += '<tr>' +
                        '<td class="mono">#' + o.index + '</td>' +
                        '<td>' + tag + '</td>' +
                        '<td class="mono">' + o.value_zyan.toFixed(4) + ' ZYAN</td>' +
                        '<td class="mono" style="font-size:0.8rem; color:#A0B0BC;">' + o.address + '</td>' +
                    '</tr>';
                });

                let bodyHtml = '<div style="margin-bottom:1.5rem;" class="mono">' +
                        '<p><strong>Hash:</strong> ' + data.hash + '</p>' +
                        '<p><strong>Blue Score:</strong> ' + data.blue_score + ' | <strong>DAA Score:</strong> ' + data.daa_score + '</p>' +
                        '<p><strong>Timestamp:</strong> ' + new Date(data.timestamp).toUTCString() + '</p>' +
                        '<p><strong>Selected Parent:</strong> ' + data.selected_parent + '</p>' +
                    '</div>' +
                    '<h4 class="mono" style="color:var(--accent-spectral); margin: 1rem 0 0.5rem;">COINBASE VESTING OUTPUTS (50% LIQUID + 50% CSV-LOCKED)</h4>' +
                    '<table style="margin-bottom:1.5rem;">' +
                        '<thead>' +
                            '<tr>' +
                                '<th class="mono">OUT #</th>' +
                                '<th class="mono">VESTING TYPE</th>' +
                                '<th class="mono">VALUE</th>' +
                                '<th class="mono">ADDRESS / SCRIPT</th>' +
                            '</tr>' +
                        '</thead>' +
                        '<tbody>' + outputsHtml + '</tbody>' +
                    '</table>' +
                    '<h4 class="mono" style="color:var(--accent-spectral); margin: 1rem 0 0.5rem;">TRANSACTIONS (' + data.transactions.length + ')</h4>' +
                    '<p style="color:#A0B0BC;" class="mono">Total Block Transactions: ' + data.transactions.length + '</p>';

                document.getElementById('detail-body').innerHTML = bodyHtml;
                document.getElementById('detail-modal').style.display = 'block';
                window.scrollTo({ top: document.getElementById('detail-modal').offsetTop - 100, behavior: 'smooth' });
            } catch (err) {
                alert('Error loading block: ' + err);
            }
        }

        function closeDetail() {
            document.getElementById('detail-modal').style.display = 'none';
        }

        async function loadContracts() {
            try {
                const res = await fetch('/api/contracts');
                const contracts = await res.json();
                let html = '';
                if (!Array.isArray(contracts) || contracts.length === 0) {
                    html = '<tr><td colspan="5" style="text-align:center; color:#A0B0BC;">No deployed contracts found</td></tr>';
                } else {
                    contracts.forEach(c => {
                        const tagClass = c.contract_type === 'DEX' ? 'tag-liquid' : 'tag-vested';
                        const shortAddr = c.address.substring(0, 12) + '...' + c.address.substring(c.address.length - 8);
                        html += '<tr>' +
                            '<td><a href="#" class="link mono" onclick="viewContract(\'' + c.address + '\')">' + shortAddr + '</a></td>' +
                            '<td><span class="tag ' + tagClass + '">' + c.contract_type + '</span></td>' +
                            '<td class="mono">' + c.bytecode_size.toLocaleString() + ' bytes</td>' +
                            '<td class="mono" style="color:#7EC8D3;">' + c.first_seen_block + '</td>' +
                            '<td><button class="nav-btn mono" style="padding:0.25rem 0.6rem; font-size:0.75rem;" onclick="viewContract(\'' + c.address + '\')">INSPECT</button></td>' +
                        '</tr>';
                    });
                }
                document.getElementById('contracts-tbody').innerHTML = html;
            } catch (err) {
                console.error(err);
            }
        }

        async function queryContractState() {
            const addr = document.getElementById('contract-addr-input').value.trim();
            const key = document.getElementById('contract-key-input').value.trim() || '0';
            if (!addr) return alert('Enter contract address');
            try {
                const res = await fetch('/api/contract/' + addr + '/state?key=' + key);
                const data = await res.json();
                document.getElementById('contract-query-result').innerHTML = '<p style="color:var(--accent-spectral);">Storage Key [' + key + '] Value: <strong>' + data.value + '</strong></p>';
            } catch (e) {
                document.getElementById('contract-query-result').innerText = 'Query Error: ' + e;
            }
        }

        async function loadTokens() {
            try {
                const res = await fetch('/api/tokens');
                const tokens = await res.json();
                let html = '';
                if (!Array.isArray(tokens) || tokens.length === 0) {
                    html = '<tr><td colspan="5" style="text-align:center; color:#A0B0BC;">No active tokens found</td></tr>';
                } else {
                    tokens.forEach(t => {
                        const shortAddr = t.contract_address.substring(0, 12) + '...' + t.contract_address.substring(t.contract_address.length - 8);
                        html += '<tr>' +
                            '<td><a href="#" class="link mono" onclick="viewContract(\'' + t.contract_address + '\')">' + shortAddr + '</a></td>' +
                            '<td class="mono" style="color:#7EC8D3; font-weight:bold;">' + t.name + ' (' + t.symbol + ')</td>' +
                            '<td class="mono">' + t.total_supply.toLocaleString() + ' ' + t.symbol + '</td>' +
                            '<td class="mono">' + t.bytecode_size.toLocaleString() + ' bytes</td>' +
                            '<td><a href="/tools" class="nav-btn mono" style="padding:0.25rem 0.6rem; font-size:0.75rem; text-decoration:none;">TRANSFER</a></td>' +
                        '</tr>';
                    });
                }
                document.getElementById('tokens-tbody').innerHTML = html;
            } catch (err) {
                console.error(err);
            }
        }

        async function loadDex() {
            try {
                const res = await fetch('/api/dex');
                const dexes = await res.json();
                let html = '';
                const list = Array.isArray(dexes) ? dexes : [dexes];
                if (list.length === 0) {
                    html = '<tr><td colspan="6" style="text-align:center; color:#A0B0BC;">No DEX pools active</td></tr>';
                } else {
                    list.forEach(d => {
                        const addr = d.address || d.dex;
                        const shortAddr = addr ? (addr.substring(0, 12) + '...' + addr.substring(addr.length - 8)) : '---';
                        const priceVal = d.price ? d.price : (d.reserveA ? (d.reserveB / d.reserveA) : 0.0);
                        const priceStr = priceVal > 0 ? priceVal.toFixed(4) + ' GHOST/ZYAN' : '---';
                        html += '<tr>' +
                            '<td><a href="#" class="link mono" onclick="viewContract(\'' + addr + '\')">' + shortAddr + '</a></td>' +
                            '<td class="mono" style="color:#7EC8D3;">' + (d.reserveA || 0).toLocaleString() + ' ZYAN</td>' +
                            '<td class="mono" style="color:#FF4D4D;">' + (d.reserveB || 0).toLocaleString() + ' GHOST</td>' +
                            '<td class="mono">' + (d.totalLPSupply || 0).toLocaleString() + ' LP</td>' +
                            '<td class="mono" style="font-weight:bold; color:#7EC8D3;">' + priceStr + '</td>' +
                            '<td><a href="/tools" class="nav-btn mono" style="padding:0.25rem 0.6rem; font-size:0.75rem; text-decoration:none;">SWAP ON DEX</a></td>' +
                        '</tr>';
                    });
                }
                document.getElementById('dex-tbody').innerHTML = html;
            } catch (err) {
                console.error(err);
            }
        }

        async function viewContract(address) {
            if (!address) return;
            try {
                const codeRes = await fetch('/api/contract/' + address + '/code');
                const code = await codeRes.json();
                
                const k0Res = await fetch('/api/contract/' + address + '/state?key=0');
                const k0 = await k0Res.json();
                const k1Res = await fetch('/api/contract/' + address + '/state?key=1');
                const k1 = await k1Res.json();
                const k2Res = await fetch('/api/contract/' + address + '/state?key=2');
                const k2 = await k2Res.json();

                document.getElementById('detail-title').innerText = 'CONTRACT DETAIL: ' + address.substring(0, 16) + '...';

                let bodyHtml = '<div style="margin-bottom:1.5rem;" class="mono">' +
                        '<p><strong>Address:</strong> ' + address + '</p>' +
                        '<p><strong>Bytecode Size:</strong> ' + (code.bytecode_size || 0) + ' bytes</p>' +
                        '<p><strong>Deploy Tx:</strong> ' + (code.deploy_tx_id || 'On-chain') + '</p>' +
                        '<p><strong>Status:</strong> <span style="color:#7EC8D3;">Active</span></p>' +
                    '</div>' +
                    '<h4 class="mono" style="color:var(--accent-spectral); margin: 1rem 0 0.5rem;">KEY STORAGE STATE</h4>' +
                    '<table style="margin-bottom:1.5rem;">' +
                        '<thead>' +
                            '<tr>' +
                                '<th class="mono">STORAGE KEY</th>' +
                                '<th class="mono">VALUE</th>' +
                                '<th class="mono">DESCRIPTION</th>' +
                            '</tr>' +
                        '</thead>' +
                        '<tbody>' +
                            '<tr><td class="mono">Key 0</td><td class="mono" style="color:#7EC8D3;">' + (k0.value || 0) + '</td><td>Total Supply / Reserve A</td></tr>' +
                            '<tr><td class="mono">Key 1</td><td class="mono" style="color:#7EC8D3;">' + (k1.value || 0) + '</td><td>Owner Balance / Reserve B</td></tr>' +
                            '<tr><td class="mono">Key 2</td><td class="mono" style="color:#7EC8D3;">' + (k2.value || 0) + '</td><td>Total LP Supply / State</td></tr>' +
                        '</tbody>' +
                    '</table>' +
                    '<h4 class="mono" style="color:var(--accent-spectral); margin: 1rem 0 0.5rem;">BYTECODE (HEX)</h4>' +
                    '<div style="background:#050810; padding:1rem; border-radius:4px; border:1px solid rgba(126,200,211,0.2); word-break:break-all; max-height:200px; overflow-y:auto;" class="mono">' +
                        (code.bytecode_hex || 'No bytecode') +
                    '</div>';

                document.getElementById('detail-body').innerHTML = bodyHtml;
                document.getElementById('detail-modal').style.display = 'block';
                window.scrollTo({ top: document.getElementById('detail-modal').offsetTop - 100, behavior: 'smooth' });
            } catch (err) {
                alert('Error loading contract detail: ' + err);
            }
        }

        async function performSearch() {
            const query = document.getElementById('search-input').value.trim();
            if (query.length === 64) {
                try {
                    const res = await fetch('/api/contract/' + query + '/code');
                    const code = await res.json();
                    if (code && code.bytecode_size > 0) {
                        return viewContract(query);
                    }
                } catch(e){}
                viewBlock(query);
            } else if (query) {
                alert('Please enter a 64-character hex hash/address');
            }
        }

        loadDashboard();
        setInterval(loadDashboard, 3000);
    </script>
    <script src="/webmcp.js"></script>
</body>
</html>
"###;

pub const WEBMCP_SCRIPT: &str = r###"
/* Zyanya Web MCP Polyfill & Agent-Native Blockchain Tool Suite */
(function() {
    if (!navigator.modelContext) {
        const toolsMap = new Map();
        navigator.modelContext = {
            tools: toolsMap,
            registerTool: function(tool) {
                if (!tool || !tool.name) throw new Error("Tool name is required");
                toolsMap.set(tool.name, tool);
                window.dispatchEvent(new CustomEvent("webmcp:registerTool", { detail: { tool } }));
                console.log("[WebMCP] Registered tool:", tool.name);
                return true;
            },
            unregisterTool: function(name) {
                toolsMap.delete(name);
                window.dispatchEvent(new CustomEvent("webmcp:unregisterTool", { detail: { name } }));
            },
            getTools: function() {
                return Array.from(toolsMap.values()).map(t => ({
                    name: t.name,
                    description: t.description,
                    inputSchema: t.inputSchema
                }));
            },
            listTools: function() {
                return this.getTools();
            },
            executeTool: async function(name, params) {
                const tool = toolsMap.get(name);
                if (!tool) throw new Error("Tool not found: " + name);
                return await tool.execute(params || {});
            },
            callTool: async function(name, params) {
                return this.executeTool(name, params);
            }
        };
    } else if (!navigator.modelContext.executeTool) {
        navigator.modelContext.executeTool = async function(name, params) {
            if (navigator.modelContext.tools && navigator.modelContext.tools.get) {
                const tool = navigator.modelContext.tools.get(name);
                if (tool) return await tool.execute(params || {});
            }
            throw new Error("Tool execution failed for " + name);
        };
    }

    window.addEventListener("message", async (event) => {
        if (!event.data || event.data.target !== "WEBMCP_POLYFILL") return;
        const { action, id, name, params } = event.data;
        if (action === "LIST_TOOLS") {
            const tools = navigator.modelContext.listTools ? navigator.modelContext.listTools() : [];
            window.postMessage({ target: "WEBMCP_INSPECTOR", action: "TOOLS_LIST", id, tools }, "*");
        } else if (action === "EXECUTE_TOOL") {
            try {
                const res = await navigator.modelContext.executeTool(name, params);
                window.postMessage({ target: "WEBMCP_INSPECTOR", action: "TOOL_RESULT", id, result: res }, "*");
            } catch (err) {
                window.postMessage({ target: "WEBMCP_INSPECTOR", action: "TOOL_ERROR", id, error: err.message }, "*");
            }
        }
    });

    const mc = navigator.modelContext;

    async function apiFetch(url, options) {
        const res = await fetch(url, options);
        if (!res.ok) {
            const errJson = await res.json().catch(() => ({ error: res.statusText }));
            throw new Error(errJson.error || "HTTP " + res.status);
        }
        return await res.json();
    }

    mc.registerTool({
        name: "get-chain-info",
        description: "Query Zyanya blockchain state including block count, DAA score, difficulty, circulating supply, sink block, and peer count.",
        inputSchema: { type: "object", properties: {} },
        execute: async () => await apiFetch('/api/info')
    });

    mc.registerTool({
        name: "get-block",
        description: "Query block details by 64-char hex block hash or retrieve recent blocks if hash is omitted.",
        inputSchema: {
            type: "object",
            properties: {
                blockHash: { type: "string", description: "64-character hex block hash" }
            }
        },
        execute: async (params) => {
            if (params && params.blockHash) {
                return await apiFetch('/api/block/' + params.blockHash);
            }
            return await apiFetch('/api/blocks');
        }
    });

    mc.registerTool({
        name: "get-dag-info",
        description: "Query parallel GHOSTDAG structure, DAG nodes, and sink block hash.",
        inputSchema: { type: "object", properties: {} },
        execute: async () => await apiFetch('/api/dag')
    });

    mc.registerTool({
        name: "get-contract-state",
        description: "Query persistent storage key-value state of a ZCL smart contract address.",
        inputSchema: {
            type: "object",
            properties: {
                contractAddress: { type: "string", description: "64-character hex contract address" },
                key: { type: "string", description: "Storage key ID (u64 integer or hex string, default 0)" }
            },
            required: ["contractAddress"]
        },
        execute: async (params) => {
            const key = (params && params.key) || "0";
            return await apiFetch('/api/contract/' + params.contractAddress + '/state?key=' + key);
        }
    });

    mc.registerTool({
        name: "get-contract-code",
        description: "Query deployed ZCL bytecode hex and size for a smart contract address.",
        inputSchema: {
            type: "object",
            properties: {
                contractAddress: { type: "string", description: "64-character hex contract address" }
            },
            required: ["contractAddress"]
        },
        execute: async (params) => await apiFetch('/api/contract/' + params.contractAddress + '/code')
    });

    mc.registerTool({
        name: "get-token-balance",
        description: "Query custom token balance for a specific holder address or storage key.",
        inputSchema: {
            type: "object",
            properties: {
                tokenAddress: { type: "string", description: "Token contract address" },
                holder: { type: "string", description: "Holder address or key ID (default 1)" }
            },
            required: ["tokenAddress"]
        },
        execute: async (params) => {
            const holder = (params && params.holder) || "1";
            return await apiFetch('/api/token-balance?token=' + params.tokenAddress + '&holder=' + holder);
        }
    });

    mc.registerTool({
        name: "get-dex-reserves",
        description: "Query DEX liquidity pool reserves (Reserve A, Reserve B, LP Supply).",
        inputSchema: {
            type: "object",
            properties: {
                dexAddress: { type: "string", description: "DEX contract address" }
            },
            required: ["dexAddress"]
        },
        execute: async (params) => await apiFetch('/api/dex-reserves?dex=' + params.dexAddress)
    });

    mc.registerTool({
        name: "deploy-contract",
        description: "Deploy a compiled ZCL contract bytecode to Zyanya network. Note: requires network gas fees.",
        inputSchema: {
            type: "object",
            properties: {
                bytecode: { type: "string", description: "Hex-encoded ZCL contract bytecode" },
                gas: { type: "number", description: "Maximum gas limit (default 100000)" }
            },
            required: ["bytecode"]
        },
        execute: async (params) => await apiFetch('/api/deploy-contract', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ bytecode: params.bytecode, gas: params.gas || 100000 })
        })
    });

    mc.registerTool({
        name: "invoke-contract",
        description: "Invoke a smart contract entry point with calldata. Note: consumes gas (50% burned).",
        inputSchema: {
            type: "object",
            properties: {
                contractAddress: { type: "string", description: "Target contract address" },
                entryPoint: { type: "number", description: "Entry point ID (u16, default 0)" },
                calldata: { type: "string", description: "Calldata (hex string or integer)" },
                gas: { type: "number", description: "Gas limit (default 100000)" }
            },
            required: ["contractAddress"]
        },
        execute: async (params) => await apiFetch('/api/invoke-contract', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                address: params.contractAddress,
                entry_point: params.entryPoint || 0,
                calldata: params.calldata || "",
                gas: params.gas || 100000
            })
        })
    });

    mc.registerTool({
        name: "call-contract",
        description: "Read-only virtual execution of a smart contract function without submitting transaction on-chain.",
        inputSchema: {
            type: "object",
            properties: {
                contractAddress: { type: "string", description: "Target contract address" },
                calldata: { type: "string", description: "Calldata hex or integer parameter" },
                entryPoint: { type: "number", description: "Entry point ID (default 0)" },
                gas: { type: "number", description: "Gas limit (default 100000)" }
            },
            required: ["contractAddress"]
        },
        execute: async (params) => await apiFetch('/api/call-contract', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                address: params.contractAddress,
                calldata: params.calldata || "",
                entry_point: params.entryPoint || 0,
                gas: params.gas || 100000
            })
        })
    });

    mc.registerTool({
        name: "deploy-token",
        description: "Deploy a custom reference ERC-20 style token contract with specified supply and owner key.",
        inputSchema: {
            type: "object",
            properties: {
                name: { type: "string", description: "Token name or symbol" },
                supply: { type: "number", description: "Initial total supply" },
                owner: { type: "string", description: "Owner address or key ID (default 1)" }
            },
            required: ["supply"]
        },
        execute: async (params) => await apiFetch('/api/deploy-token', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                name: params.name || "Token",
                supply: params.supply,
                owner: params.owner || "1"
            })
        })
    });

    mc.registerTool({
        name: "token-transfer",
        description: "Transfer custom tokens from sender to recipient address.",
        inputSchema: {
            type: "object",
            properties: {
                tokenAddress: { type: "string", description: "Token contract address" },
                from: { type: "string", description: "Sender key ID (default 1)" },
                to: { type: "string", description: "Recipient key ID or address" },
                amount: { type: "number", description: "Amount of tokens to transfer" }
            },
            required: ["tokenAddress", "to", "amount"]
        },
        execute: async (params) => await apiFetch('/api/token-transfer', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                token: params.tokenAddress,
                from: params.from || "1",
                to: params.to,
                amount: params.amount
            })
        })
    });

    mc.registerTool({
        name: "swap-on-dex",
        description: "Perform an automated token swap on a Zyanya DEX liquidity pool.",
        inputSchema: {
            type: "object",
            properties: {
                dexAddress: { type: "string", description: "DEX contract address" },
                tokenIn: { type: "string", description: "Input token ('a', 'b', 'zyan', 'ghost', '0', '1')" },
                amountIn: { type: "number", description: "Input token amount" }
            },
            required: ["dexAddress", "tokenIn", "amountIn"]
        },
        execute: async (params) => await apiFetch('/api/swap-on-dex', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                dex: params.dexAddress,
                token_in: params.tokenIn,
                amount_in: params.amountIn
            })
        })
    });

    mc.registerTool({
        name: "compile-contract",
        description: "Compile ZCL high-level contract source code into executable VM bytecode hex.",
        inputSchema: {
            type: "object",
            properties: {
                source: { type: "string", description: "ZCL source code string" }
            },
            required: ["source"]
        },
        execute: async (params) => await apiFetch('/api/compile-contract', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ source: params.source })
        })
    });

    console.log("[WebMCP] Zyanya Web MCP Tool Suite initialized. Total tools:", mc.getTools().length);
})();
"###;

pub const TOOLS_HTML: &str = r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Zyanya Web MCP Tools — Agent-Native Blockchain</title>
    <style>
        :root {
            --bg-base: #0A0F1C;
            --bg-shadow: #0D3B50;
            --accent-spectral: #7EC8D3;
            --text-main: #E0E0E0;
            --burn-red: #FF4D4D;
            --badge-read: #2E7D32;
            --badge-op: #C62828;
        }

        * { box-sizing: border-box; margin: 0; padding: 0; }

        body {
            background-color: var(--bg-base);
            color: var(--text-main);
            font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
            line-height: 1.6;
        }

        .mono { font-family: 'Courier New', Courier, monospace; }

        .grid-bg {
            position: fixed;
            top: 0; left: 0; right: 0; bottom: 0;
            background-image: 
                radial-gradient(circle at 50% 30%, rgba(13, 59, 80, 0.35) 0%, transparent 70%),
                linear-gradient(rgba(126, 200, 211, 0.03) 1px, transparent 1px),
                linear-gradient(90deg, rgba(126, 200, 211, 0.03) 1px, transparent 1px);
            background-size: 100% 100%, 40px 40px, 40px 40px;
            z-index: -1;
            pointer-events: none;
        }

        header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 1.2rem 2.5rem;
            border-bottom: 1px solid rgba(126, 200, 211, 0.2);
            background: rgba(10, 15, 28, 0.95);
            backdrop-filter: blur(10px);
            position: sticky;
            top: 0;
            z-index: 100;
        }

        .logo-wrap { display: flex; align-items: center; gap: 1rem; text-decoration: none; }
        .logo-wrap svg { height: 32px; width: auto; }

        .ipv6-badge {
            background: rgba(126, 200, 211, 0.1);
            border: 1px solid var(--accent-spectral);
            color: var(--accent-spectral);
            padding: 0.25rem 0.75rem;
            border-radius: 20px;
            font-size: 0.8rem;
            letter-spacing: 1px;
        }

        nav { display: flex; gap: 1.5rem; align-items: center; }
        nav a {
            color: var(--text-main);
            text-decoration: none;
            font-size: 0.9rem;
            letter-spacing: 1px;
            transition: all 0.2s;
        }
        nav a:hover, nav a.active {
            color: var(--accent-spectral);
            text-shadow: 0 0 8px rgba(126, 200, 211, 0.6);
        }

        .btn {
            background: rgba(13, 59, 80, 0.5);
            border: 1px solid var(--accent-spectral);
            color: var(--accent-spectral);
            padding: 0.5rem 1.2rem;
            border-radius: 4px;
            cursor: pointer;
            text-decoration: none;
            font-weight: 600;
            font-size: 0.85rem;
            letter-spacing: 1px;
            transition: all 0.3s ease;
        }
        .btn:hover {
            background: var(--accent-spectral);
            color: var(--bg-base);
            box-shadow: 0 0 15px rgba(126, 200, 211, 0.5);
        }

        .container { max-width: 1250px; margin: 0 auto; padding: 2rem; }

        .hero-card {
            background: linear-gradient(135deg, rgba(13, 59, 80, 0.4), rgba(10, 15, 28, 0.8));
            border: 1px solid var(--accent-spectral);
            border-radius: 8px;
            padding: 2.5rem;
            margin-bottom: 2.5rem;
            box-shadow: 0 0 25px rgba(126, 200, 211, 0.15);
        }

        .hero-title {
            font-size: 1.8rem;
            color: var(--accent-spectral);
            letter-spacing: 2px;
            margin-bottom: 1rem;
        }

        .hero-text {
            font-size: 1.05rem;
            color: #B0C4CE;
            line-height: 1.7;
            margin-bottom: 1.5rem;
        }

        .hero-stats {
            display: flex;
            gap: 2rem;
            flex-wrap: wrap;
        }

        .hero-stat-item {
            background: rgba(5, 8, 16, 0.7);
            border: 1px solid rgba(126, 200, 211, 0.2);
            padding: 0.75rem 1.25rem;
            border-radius: 6px;
        }

        .hero-stat-val { font-size: 1.2rem; font-weight: 700; color: var(--accent-spectral); }
        .hero-stat-lbl { font-size: 0.75rem; color: #90A0B0; }

        .section-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin: 2rem 0 1.5rem;
            border-bottom: 1px solid rgba(126, 200, 211, 0.2);
            padding-bottom: 0.75rem;
        }

        .section-title { font-size: 1.3rem; color: var(--accent-spectral); letter-spacing: 2px; }

        /* Inspector card */
        .inspector-card {
            background: rgba(13, 59, 80, 0.3);
            border: 1px solid rgba(126, 200, 211, 0.3);
            border-radius: 8px;
            padding: 2rem;
            margin-bottom: 3rem;
        }

        .form-group { margin-bottom: 1.25rem; }
        .form-label { display: block; font-size: 0.85rem; color: var(--accent-spectral); margin-bottom: 0.4rem; letter-spacing: 1px; }

        select.form-input, textarea.form-input, input.form-input {
            width: 100%;
            background: #050810;
            border: 1px solid rgba(126, 200, 211, 0.3);
            color: var(--text-main);
            padding: 0.75rem 1rem;
            border-radius: 4px;
            font-family: 'Courier New', Courier, monospace;
            font-size: 0.9rem;
        }

        select.form-input option { background: #0A0F1C; color: var(--text-main); }

        .output-box {
            background: #04060C;
            border: 1px solid rgba(126, 200, 211, 0.25);
            border-radius: 6px;
            padding: 1.25rem;
            min-height: 120px;
            max-height: 350px;
            overflow-y: auto;
            color: #A0F0D0;
            white-space: pre-wrap;
            word-break: break-all;
            font-size: 0.85rem;
        }

        /* Tools grid */
        .tools-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(360px, 1fr));
            gap: 1.5rem;
            margin-bottom: 3rem;
        }

        .tool-card {
            background: rgba(13, 59, 80, 0.2);
            border: 1px solid rgba(126, 200, 211, 0.2);
            border-radius: 8px;
            padding: 1.5rem;
            transition: all 0.3s ease;
        }

        .tool-card:hover {
            border-color: var(--accent-spectral);
            transform: translateY(-3px);
            box-shadow: 0 8px 25px rgba(13, 59, 80, 0.5);
        }

        .tool-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; }
        .tool-name { font-size: 1.1rem; color: var(--accent-spectral); font-weight: 700; }

        .tag-query { background: rgba(46, 125, 50, 0.25); color: #81C784; border: 1px solid #4CAF50; padding: 0.15rem 0.5rem; border-radius: 3px; font-size: 0.7rem; }
        .tag-op { background: rgba(198, 40, 40, 0.25); color: #E57373; border: 1px solid #F44336; padding: 0.15rem 0.5rem; border-radius: 3px; font-size: 0.7rem; }

        .tool-desc { color: #A0B0BC; font-size: 0.9rem; margin-bottom: 1rem; line-height: 1.5; }

        .schema-box {
            background: #050810;
            border: 1px solid rgba(126, 200, 211, 0.15);
            padding: 0.75rem;
            border-radius: 4px;
            font-size: 0.8rem;
            color: #7EC8D3;
        }

        footer {
            border-top: 1px solid rgba(126, 200, 211, 0.15);
            padding: 2.5rem 0;
            text-align: center;
            color: #708090;
            font-size: 0.85rem;
        }
    </style>
</head>
<body>
    <div class="grid-bg"></div>

    <header>
        <a href="/" class="logo-wrap">
            <div id="tools-logo"></div>
            <span class="mono" style="font-size: 1.1rem; color: var(--accent-spectral); letter-spacing: 2px;">WEBMCP TOOLS</span>
        </a>
        <div class="ipv6-badge mono">[::]:8098 • AGENT-NATIVE</div>
        <nav>
            <a href="/" class="mono">WEBSITE</a>
            <a href="/explorer" class="mono">EXPLORER</a>
            <a href="/tools" class="mono active">WEBMCP TOOLS</a>
            <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" class="mono" style="color: var(--accent-spectral);">GITHUB</a>
        </nav>
    </header>

    <main class="container">
        <section class="hero-card">
            <h1 class="hero-title mono">ZYANYA: THE AGENT-NATIVE BLOCKCHAIN</h1>
            <p class="hero-text">
                Zyanya exposes all blockchain queries, smart contract execution, DEX swaps, and token operations as <strong>Web MCP Tools</strong> via <code class="mono">navigator.modelContext</code> (W3C Web MCP standard). AI agents visiting <code class="mono">zyanya.scottcloudhawk.org</code> interact with typed tool definitions directly — no OCR, no screenshots, and no clicking.
            </p>
            <div class="hero-stats mono">
                <div class="hero-stat-item">
                    <div class="hero-stat-val">14 TOOLS</div>
                    <div class="hero-stat-lbl">REGISTERED WEBMCP TOOLS</div>
                </div>
                <div class="hero-stat-item">
                    <div class="hero-stat-val">W3C SPEC</div>
                    <div class="hero-stat-lbl">NAVIGATOR.MODELCONTEXT</div>
                </div>
                <div class="hero-stat-item">
                    <div class="hero-stat-val">MCP-B POLYFILL</div>
                    <div class="hero-stat-lbl">CROSS-BROWSER ENABLED</div>
                </div>
                <div class="hero-stat-item">
                    <div class="hero-stat-val">IPv6 + AGENT</div>
                    <div class="hero-stat-lbl">WORLD FIRST POSITIONING</div>
                </div>
            </div>
        </section>

        <div class="section-header">
            <h2 class="section-title mono">INTERACTIVE WEBMCP TOOL INSPECTOR</h2>
            <span class="mono" style="font-size: 0.8rem; color: #708090;">Test Web MCP execution in browser</span>
        </div>

        <div class="inspector-card">
            <div class="form-group">
                <label class="form-label mono">SELECT WEBMCP TOOL</label>
                <select id="tool-select" class="form-input mono" onchange="onToolSelectChange()">
                    <option value="">Loading tools from navigator.modelContext...</option>
                </select>
            </div>

            <div class="form-group">
                <label class="form-label mono">TOOL DESCRIPTION & PARAMETERS (JSON INPUT)</label>
                <div id="tool-desc-display" style="color: #B0C4CE; font-size: 0.9rem; margin-bottom: 0.5rem;" class="mono"></div>
                <textarea id="tool-params-input" class="form-input mono" rows="4" placeholder="{}"></textarea>
            </div>

            <button class="btn mono" style="margin-bottom: 1.5rem; font-size: 0.95rem; padding: 0.75rem 2rem;" onclick="runSelectedTool()">EXECUTE WEBMCP TOOL ⚡</button>

            <div class="form-group">
                <label class="form-label mono">EXECUTION OUTPUT (RESULT JSON)</label>
                <div id="tool-output-box" class="output-box mono">Select a tool above and click Execute.</div>
            </div>
        </div>

        <div class="section-header">
            <h2 class="section-title mono">REGISTERED WEBMCP BLOCKCHAIN TOOLS</h2>
            <span class="mono" style="font-size: 0.8rem; color: #708090;">14 Exposed Operations</span>
        </div>

        <div class="tools-grid" id="tools-cards-grid">
            <!-- Dynamically populated from navigator.modelContext -->
        </div>
    </main>

    <footer>
        <div class="container">
            <p class="mono">ZYANYA PROTOCOL • THE FIRST IPv6-NATIVE + AGENT-NATIVE BLOCKCHAIN</p>
            <p style="margin-top: 0.5rem; opacity: 0.6;">Exposing typed blockchain operations via navigator.modelContext</p>
        </div>
    </footer>

    <script src="/webmcp.js"></script>
    <script>
        fetch('/brand/zyanya-logo.svg').then(r => r.text()).then(html => {
            document.getElementById('tools-logo').innerHTML = html;
        });

        function populateToolsUI() {
            if (!navigator.modelContext) return;
            const tools = navigator.modelContext.getTools();
            const select = document.getElementById('tool-select');
            select.innerHTML = '';

            let gridHtml = '';

            tools.forEach((t, idx) => {
                const opt = document.createElement('option');
                opt.value = t.name;
                opt.textContent = (idx + 1) + '. ' + t.name + ' — ' + t.description;
                select.appendChild(opt);

                const isQuery = t.name.startsWith('get-');
                const tagClass = isQuery ? 'tag-query' : 'tag-op';
                const tagText = isQuery ? 'READ-ONLY QUERY' : 'NETWORK OPERATION';

                gridHtml += '<div class="tool-card">' +
                    '<div class="tool-header">' +
                        '<span class="tool-name mono">' + t.name + '</span>' +
                        '<span class="' + tagClass + ' mono">' + tagText + '</span>' +
                    '</div>' +
                    '<p class="tool-desc">' + t.description + '</p>' +
                    '<div class="schema-box mono"><pre style="margin:0;">' + JSON.stringify(t.inputSchema, null, 2) + '</pre></div>' +
                '</div>';
            });

            document.getElementById('tools-cards-grid').innerHTML = gridHtml;
            onToolSelectChange();
        }

        function onToolSelectChange() {
            const select = document.getElementById('tool-select');
            const name = select.value;
            if (!name || !navigator.modelContext) return;

            const tools = navigator.modelContext.getTools();
            const tool = tools.find(t => t.name === name);
            if (!tool) return;

            document.getElementById('tool-desc-display').innerText = tool.description;

            let sample = {};
            if (name === 'get-block') sample = { blockHash: "" };
            if (name === 'get-contract-state') sample = { contractAddress: "0000000000000000000000000000000000000000000000000000000000000000", key: "0" };
            if (name === 'get-contract-code') sample = { contractAddress: "0000000000000000000000000000000000000000000000000000000000000000" };
            if (name === 'get-token-balance') sample = { tokenAddress: "0000000000000000000000000000000000000000000000000000000000000000", holder: "1" };
            if (name === 'get-dex-reserves') sample = { dexAddress: "0000000000000000000000000000000000000000000000000000000000000000" };
            if (name === 'deploy-contract') sample = { bytecode: "608060405234801561001057600080fd5b50", gas: 100000 };
            if (name === 'invoke-contract') sample = { contractAddress: "0000000000000000000000000000000000000000000000000000000000000000", entryPoint: 0, calldata: "1", gas: 100000 };
            if (name === 'call-contract') sample = { contractAddress: "0000000000000000000000000000000000000000000000000000000000000000", calldata: "1", entryPoint: 0, gas: 100000 };
            if (name === 'deploy-token') sample = { name: "MYTOKEN", supply: 1000000, owner: "1" };
            if (name === 'token-transfer') sample = { tokenAddress: "0000000000000000000000000000000000000000000000000000000000000000", from: "1", to: "2", amount: 100 };
            if (name === 'swap-on-dex') sample = { dexAddress: "0000000000000000000000000000000000000000000000000000000000000000", tokenIn: "zyan", amountIn: 50 };
            if (name === 'compile-contract') sample = { source: "entry main() { sstore(0, 100); }" };

            document.getElementById('tool-params-input').value = JSON.stringify(sample, null, 2);
        }

        async function runSelectedTool() {
            const select = document.getElementById('tool-select');
            const name = select.value;
            const outputBox = document.getElementById('tool-output-box');
            if (!name) return alert('Select a tool first');

            let params = {};
            try {
                const paramText = document.getElementById('tool-params-input').value.trim();
                if (paramText) params = JSON.parse(paramText);
            } catch (err) {
                return alert('Invalid JSON in parameters input: ' + err.message);
            }

            outputBox.innerText = 'Executing navigator.modelContext.executeTool("' + name + '")...';
            const startTime = performance.now();

            try {
                const result = await navigator.modelContext.executeTool(name, params);
                const elapsed = (performance.now() - startTime).toFixed(1);
                outputBox.innerText = '// Web MCP Execution Success (' + elapsed + 'ms)\n' + JSON.stringify(result, null, 2);
            } catch (err) {
                const elapsed = (performance.now() - startTime).toFixed(1);
                outputBox.innerText = '// Web MCP Execution Error (' + elapsed + 'ms)\n' + err.message;
            }
        }

        setTimeout(populateToolsUI, 100);
    </script>
</body>
</html>
"###;
