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
    <title>Zyanya — The Ghost in the IPv6 Machine</title>
    <style>
        :root {
            --bg-base: #0A0F1C;
            --bg-shadow: #0D3B50;
            --accent-spectral: #7EC8D3;
            --text-main: #E0E0E0;
            --burn-red: #FF4D4D;
        }

        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }

        body {
            background-color: var(--bg-base);
            color: var(--text-main);
            font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
            line-height: 1.6;
            overflow-x: hidden;
        }

        .mono {
            font-family: 'Courier New', Courier, monospace;
        }

        /* Ambient Glow & Grid */
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
            padding: 1.5rem 3rem;
            border-bottom: 1px solid rgba(126, 200, 211, 0.15);
            background: rgba(10, 15, 28, 0.85);
            backdrop-filter: blur(12px);
            position: sticky;
            top: 0;
            z-index: 100;
        }

        .brand-logo {
            display: flex;
            align-items: center;
            gap: 1rem;
            text-decoration: none;
        }

        .brand-logo svg {
            height: 38px;
            width: auto;
        }

        .ipv6-badge {
            background: rgba(126, 200, 211, 0.1);
            border: 1px solid var(--accent-spectral);
            color: var(--accent-spectral);
            padding: 0.25rem 0.75rem;
            border-radius: 20px;
            font-size: 0.8rem;
            letter-spacing: 1px;
            box-shadow: 0 0 10px rgba(126, 200, 211, 0.2);
        }

        nav {
            display: flex;
            gap: 2rem;
            align-items: center;
        }

        nav a {
            color: var(--text-main);
            text-decoration: none;
            font-size: 0.95rem;
            letter-spacing: 1px;
            transition: all 0.3s ease;
        }

        nav a:hover {
            color: var(--accent-spectral);
            text-shadow: 0 0 8px rgba(126, 200, 211, 0.6);
        }

        .btn {
            background: rgba(13, 59, 80, 0.5);
            border: 1px solid var(--accent-spectral);
            color: var(--accent-spectral);
            padding: 0.75rem 1.5rem;
            border-radius: 4px;
            cursor: pointer;
            text-decoration: none;
            font-weight: 600;
            letter-spacing: 2px;
            transition: all 0.3s ease;
            display: inline-block;
        }

        .btn:hover {
            background: var(--accent-spectral);
            color: var(--bg-base);
            box-shadow: 0 0 20px rgba(126, 200, 211, 0.5);
        }

        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 0 2rem;
        }

        /* Hero */
        .hero {
            padding: 5rem 0 4rem;
            text-align: center;
        }

        .hero-banner-wrap {
            margin: 0 auto 2.5rem;
            max-width: 900px;
            border: 1px solid rgba(126, 200, 211, 0.2);
            border-radius: 8px;
            overflow: hidden;
            box-shadow: 0 0 30px rgba(13, 59, 80, 0.5);
            background: rgba(10, 15, 28, 0.6);
        }

        .hero-banner-wrap img, .hero-banner-wrap svg {
            width: 100%;
            height: auto;
            display: block;
        }

        .pitch {
            font-size: 1.25rem;
            max-width: 800px;
            margin: 0 auto 2.5rem;
            color: #B0C4CE;
            line-height: 1.8;
            font-weight: 300;
        }

        .pitch strong {
            color: var(--accent-spectral);
            font-weight: 600;
        }

        .hero-cta {
            display: flex;
            gap: 1.5rem;
            justify-content: center;
        }

        /* Pillars Section */
        .section-title {
            text-align: center;
            font-size: 1.8rem;
            letter-spacing: 4px;
            color: var(--accent-spectral);
            margin: 4rem 0 2.5rem;
            text-shadow: 0 0 10px rgba(126, 200, 211, 0.3);
        }

        .pillars-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
            gap: 2rem;
            margin-bottom: 5rem;
        }

        .card {
            background: rgba(13, 59, 80, 0.25);
            border: 1px solid rgba(126, 200, 211, 0.2);
            border-radius: 8px;
            padding: 2.5rem 2rem;
            transition: all 0.3s ease;
            backdrop-filter: blur(6px);
        }

        .card:hover {
            transform: translateY(-5px);
            border-color: var(--accent-spectral);
            box-shadow: 0 10px 30px rgba(13, 59, 80, 0.6);
        }

        .card-icon {
            width: 60px;
            height: 60px;
            margin-bottom: 1.5rem;
        }

        .card-title {
            font-size: 1.2rem;
            letter-spacing: 2px;
            color: var(--accent-spectral);
            margin-bottom: 1rem;
        }

        .card-desc {
            color: #A0B0BC;
            font-size: 0.95rem;
            line-height: 1.7;
        }

        /* Features & Economics */
        .two-col {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 3rem;
            margin-bottom: 5rem;
        }

        @media (max-width: 850px) {
            .two-col { grid-template-columns: 1fr; }
        }

        .feature-item {
            display: flex;
            gap: 1rem;
            margin-bottom: 1.5rem;
        }

        .feature-bullet {
            color: var(--accent-spectral);
            font-size: 1.2rem;
        }

        .burn-badge {
            color: var(--burn-red);
            border: 1px solid var(--burn-red);
            padding: 0.1rem 0.4rem;
            border-radius: 3px;
            font-size: 0.75rem;
            margin-left: 0.5rem;
        }

        /* How to Join */
        .join-box {
            background: rgba(10, 15, 28, 0.9);
            border: 1px solid var(--accent-spectral);
            border-radius: 8px;
            padding: 3rem;
            margin-bottom: 5rem;
            box-shadow: 0 0 30px rgba(126, 200, 211, 0.1);
        }

        .code-block {
            background: #050810;
            border: 1px solid rgba(126, 200, 211, 0.3);
            padding: 1.25rem;
            border-radius: 6px;
            color: var(--accent-spectral);
            font-size: 0.9rem;
            overflow-x: auto;
            margin-top: 1rem;
        }

        footer {
            border-top: 1px solid rgba(126, 200, 211, 0.15);
            padding: 3rem 0;
            text-align: center;
            color: #708090;
            font-size: 0.85rem;
        }
    </style>
</head>
<body>
    <div class="grid-bg"></div>

    <header>
        <a href="/" class="brand-logo">
            <div id="logo-container"></div>
        </a>
        <div class="ipv6-badge mono">[::]:8098 • IPv6-ONLY</div>
        <nav>
            <a href="#pillars" class="mono">PILLARS</a>
            <a href="#economics" class="mono">ECONOMICS</a>
            <a href="#join" class="mono">JOIN</a>
            <a href="/explorer" class="btn mono">BLOCK EXPLORER</a>
        </nav>
    </header>

    <main class="container">
        <section class="hero">
            <div class="hero-banner-wrap" id="banner-container"></div>
            
            <p class="pitch">
                <strong>Zyanya</strong> is the pure IPv6-native blockchain. Built with Spectre/GhostDAG consensus for parallel block DAG resolution. Zero NAT. Zero port forwarding. Every node is a first-class citizen with direct peer-to-peer reachability.
            </p>

            <div class="hero-cta">
                <a href="/explorer" class="btn mono">LAUNCH BLOCK EXPLORER</a>
                <a href="#join" class="btn mono" style="border-color: #0D3B50; color: #E0E0E0;">JOIN DEVNET</a>
            </div>
        </section>

        <h2 class="section-title mono" id="pillars">THE THREE PILLARS</h2>
        <div class="pillars-grid">
            <div class="card">
                <div class="card-icon" id="icon-coin"></div>
                <h3 class="card-title mono">I. THE GHOST</h3>
                <p class="card-desc">
                    GhostDAG consensus parallel block resolution. High transaction throughput with instant DAG finality, eliminating orphan blocks and securing the chain against 51% reorgs.
                </p>
            </div>
            <div class="card">
                <div class="card-icon" id="icon-token"></div>
                <h3 class="card-title mono">II. THE SECRET</h3>
                <p class="card-desc">
                    Pure IPv6 subnet architecture. No middleboxes, no relay nodes, no NAT traversal hacks. Direct global IPv6 connectivity for total network decentralization.
                </p>
            </div>
            <div class="card">
                <div class="card-icon" id="icon-burn"></div>
                <h3 class="card-title mono">III. THE FOREVER</h3>
                <p class="card-desc">
                    Eternally locked coinbase vesting. 50% liquid + 50% CSV-locked vested outputs over 12 months. Miners remain long-term aligned with sustainable tokenomics.
                </p>
            </div>
        </div>

        <div class="two-col" id="economics">
            <div class="card">
                <h3 class="card-title mono" style="margin-bottom: 1.5rem;">PROTOCOL FEATURES</h3>
                <div class="feature-item">
                    <span class="feature-bullet">•</span>
                    <div>
                        <strong>Deflationary Smart Contracts <span class="burn-badge mono">50% BURN</span></strong>
                        <p class="card-desc">50% of gas fees consumed during ZCL contract execution are permanently destroyed.</p>
                    </div>
                </div>
                <div class="feature-item">
                    <span class="feature-bullet">•</span>
                    <div>
                        <strong>ZCL Contract Language & VM</strong>
                        <p class="card-desc">Assembly-compiled stack-based VM with native persistent storage operations (<code class="mono">SSTORE</code> / <code class="mono">SLOAD</code>).</p>
                    </div>
                </div>
                <div class="feature-item">
                    <span class="feature-bullet">•</span>
                    <div>
                        <strong>Custom Reference Tokens</strong>
                        <p class="card-desc">Native ERC-20 style token minting, transfers, and supply tracking out of the box.</p>
                    </div>
                </div>
            </div>

            <div class="card">
                <h3 class="card-title mono" style="margin-bottom: 1.5rem;">COIN ECONOMICS</h3>
                <div class="feature-item">
                    <span class="feature-bullet">•</span>
                    <div>
                        <strong>Block Subsidy: 50 ZYAN / Block</strong>
                        <p class="card-desc">25.0 ZYAN (50%) liquid payout directly to miner's address.</p>
                    </div>
                </div>
                <div class="feature-item">
                    <span class="feature-bullet">•</span>
                    <div>
                        <strong>12-Month Vesting Schedule</strong>
                        <p class="card-desc">25.0 ZYAN (50%) locked across 12 monthly relative lock-time CSV outputs.</p>
                    </div>
                </div>
                <div class="feature-item">
                    <span class="feature-bullet">•</span>
                    <div>
                        <strong>CPU-Mineable Proof-of-Work</strong>
                        <p class="card-desc">Designed for broad hardware participation and decentralized consensus voting.</p>
                    </div>
                </div>
            </div>
        </div>

        <div class="join-box" id="join">
            <h2 class="card-title mono" style="font-size: 1.5rem; margin-bottom: 1rem;">HOW TO JOIN THE DEVNET</h2>
            <p style="color: #A0B0BC; margin-bottom: 1.5rem;">
                Zyanya is bound strictly over IPv6. Connect your node directly to the seed address below:
            </p>
            <div class="mono" style="color: var(--accent-spectral); margin-bottom: 1rem;">
                <strong>SEED IPV6 NODE:</strong> 2606:8ac0:2615:79aa:5a47:caff:fe7b:d473
            </div>
            
            <div class="code-block mono">
# Connect node to devnet seed<br>
zyanyad --devnet --outpeers=8 --addpeer=2606:8ac0:2615:79aa:5a47:caff:fe7b:d473<br><br>
# Query chain state over gRPC<br>
zyanya-query --rpcserver="[2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18610" get-info
            </div>
        </div>
    </main>

    <footer>
        <div class="container">
            <p class="mono">ZYANYA PROTOCOL • THE GHOST IN THE IPv6 MACHINE • FOREVER. ALWAYS.</p>
            <p style="margin-top: 0.5rem; opacity: 0.6;">Served exclusively over IPv6 sockets on port 8098. Pure P2P positioning verified.</p>
        </div>
    </footer>

    <script>
        fetch('/brand/zyanya-logo.svg').then(r => r.text()).then(html => {
            document.getElementById('logo-container').innerHTML = html;
        });
        fetch('/brand/zyanya-hero-banner.svg').then(r => r.text()).then(html => {
            document.getElementById('banner-container').innerHTML = html;
        });
        fetch('/brand/zyan-coin.svg').then(r => r.text()).then(html => {
            document.getElementById('icon-coin').innerHTML = html;
        });
        fetch('/brand/ghost-token.svg').then(r => r.text()).then(html => {
            document.getElementById('icon-token').innerHTML = html;
        });
        fetch('/brand/gas-burn-icon.svg').then(r => r.text()).then(html => {
            document.getElementById('icon-burn').innerHTML = html;
        });
    </script>
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
            <button class="nav-btn mono" onclick="switchTab('dag')">DAG GRAPH</button>
            <a href="/" class="nav-btn mono" style="text-decoration: none;">WEBSITE</a>
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
                    <span class="mono" style="font-size: 0.8rem; color: #708090;">Auto-refreshing live devnet</span>
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
                    <h3 class="card-title mono">SMART CONTRACTS (ZCL VM)</h3>
                </div>
                <p style="color: #A0B0BC; margin-bottom: 1rem;">
                    Contracts deployed on Zyanya feature 50% gas burn deflationary mechanics and state key storage.
                </p>
                <div id="contracts-container"></div>
            </div>
        </div>

        <div id="tab-tokens" class="tab-content">
            <div class="card">
                <div class="card-header">
                    <h3 class="card-title mono">NATIVE CUSTOM TOKENS</h3>
                </div>
                <p style="color: #A0B0BC; margin-bottom: 1rem;">
                    ERC-20 style custom tokens deployed on-chain with total supply and key holder balances.
                </p>
                <div id="tokens-container"></div>
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
            event.target.classList.add('active');
            closeDetail();

            if (name === 'contracts') loadContracts();
            if (name === 'tokens') loadTokens();
            if (name === 'dag') loadDag();
        }

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
                    html += '<tr>' +
                        '<td><a href="#" class="link mono" onclick="viewBlock(\'' + b.hash + '\')">' + shortHash + '</a></td>' +
                        '<td class="mono">' + b.blue_score + '</td>' +
                        '<td class="mono">' + b.daa_score + '</td>' +
                        '<td class="mono">' + timeStr + '</td>' +
                        '<td class="mono">' + b.tx_count + '</td>' +
                        '<td class="mono"><a href="#" class="link" onclick="viewBlock(\'' + b.selected_parent + '\')">' + shortParent + '</a></td>' +
                    '</tr>';
                });
                document.getElementById('blocks-tbody').innerHTML = html;
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
            document.getElementById('contracts-container').innerHTML = '<div class="card" style="background:#050810;">' +
                '<h4 class="mono" style="color:var(--accent-spectral);">Query Contract State Inspector</h4>' +
                '<div style="display:flex; gap:0.5rem; margin-top:0.75rem;">' +
                    '<input type="text" id="contract-addr-input" class="search-input mono" placeholder="Contract Address (e.g. 0000000000000000000000000000000000000000000000000000000000000000)">' +
                    '<input type="text" id="contract-key-input" class="search-input mono" style="max-width:150px;" placeholder="Key (e.g. 0)">' +
                    '<button class="search-btn mono" onclick="queryContractState()">QUERY STATE</button>' +
                '</div>' +
                '<div id="contract-query-result" style="margin-top:1rem;" class="mono"></div>' +
            '</div>';
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
            document.getElementById('tokens-container').innerHTML = '<div class="card" style="background:#050810;">' +
                '<h4 class="mono" style="color:var(--accent-spectral);">Reference Token Contract Standard</h4>' +
                '<p style="margin-top:0.5rem; color:#A0B0BC;">' +
                    'Tokens deployed using <code class="mono">zyanya-query deploy-token</code> store total supply at Key 0 and owner balance at Key 1.' +
                '</p>' +
            '</div>';
        }

        async function loadDag() {
            try {
                const res = await fetch('/api/dag');
                const data = await res.json();
                const svg = document.getElementById('dag-svg');
                svg.innerHTML = '';

                const width = svg.clientWidth || 800;
                const height = 380;
                const nodes = data.nodes;

                nodes.forEach((n, i) => {
                    const x = width - 60 - (i * 45);
                    const y = height / 2 + (Math.sin(i * 0.8) * 60);

                    if (i < nodes.length - 1) {
                        const nextX = width - 60 - ((i + 1) * 45);
                        const nextY = height / 2 + (Math.sin((i + 1) * 0.8) * 60);
                        const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
                        line.setAttribute('x1', x);
                        line.setAttribute('y1', y);
                        line.setAttribute('x2', nextX);
                        line.setAttribute('y2', nextY);
                        line.setAttribute('stroke', '#7EC8D3');
                        line.setAttribute('stroke-width', '1.5');
                        line.setAttribute('opacity', '0.4');
                        line.setAttribute('stroke-dasharray', '3,3');
                        svg.appendChild(line);
                    }

                    const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
                    circle.setAttribute('cx', x);
                    circle.setAttribute('cy', y);
                    circle.setAttribute('r', '14');
                    circle.setAttribute('fill', n.is_chain_block ? '#0D3B50' : '#0A0F1C');
                    circle.setAttribute('stroke', '#7EC8D3');
                    circle.setAttribute('stroke-width', '2');
                    circle.setAttribute('cursor', 'pointer');
                    circle.onclick = () => viewBlock(n.hash);
                    svg.appendChild(circle);

                    const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                    text.setAttribute('x', x);
                    text.setAttribute('y', y + 4);
                    text.setAttribute('font-size', '9');
                    text.setAttribute('fill', '#7EC8D3');
                    text.setAttribute('text-anchor', 'middle');
                    text.setAttribute('font-family', 'monospace');
                    text.textContent = n.blue_score;
                    svg.appendChild(text);
                });
            } catch (err) {
                console.error(err);
            }
        }

        function performSearch() {
            const query = document.getElementById('search-input').value.trim();
            if (query.length === 64) {
                viewBlock(query);
            } else if (query) {
                alert('Please enter a 64-character hex hash/address');
            }
        }

        loadDashboard();
        setInterval(loadDashboard, 10000);
    </script>
</body>
</html>
"###;
