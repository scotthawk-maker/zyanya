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
            justify-content: center;
            align-items: center;
            padding: 1.5rem 0;
            border-bottom: 1px solid var(--shadow-teal);
            position: relative;
            width: 100%;
        }

        .menu-toggle { display: none; }

        .hamburger {
            display: none;
            font-size: 1.8rem;
            color: var(--spectral-blue);
            cursor: pointer;
            padding: 0.5rem 1rem;
            user-select: none;
            z-index: 101;
        }

        nav {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 1.5rem;
            flex-wrap: wrap;
        }

        nav a {
            color: var(--spectral-blue);
            text-decoration: none;
            font-weight: 600;
            font-size: 0.95rem;
            transition: color 0.3s ease;
            display: inline-flex;
            align-items: center;
        }

        nav a:hover, nav a.active {
            color: #FFFFFF;
            text-shadow: 0 0 8px var(--spectral-blue);
        }

        .top-logo {
            text-align: center;
            margin-top: 2rem;
            margin-bottom: 1rem;
        }

        .top-logo svg {
            max-width: 520px;
            width: 100%;
            height: auto;
        }

        #logo-container svg {
            height: 40px;
            width: auto;
        }

        @media (max-width: 768px) {
            header {
                flex-direction: column;
                padding: 1rem 0;
            }

            .hamburger {
                display: block;
            }

            nav {
                display: none;
                flex-direction: column;
                width: 100%;
                background: rgba(10, 15, 28, 0.98);
                border: 1px solid var(--shadow-teal);
                border-radius: 8px;
                margin-top: 0.5rem;
                padding: 0.5rem 0;
                gap: 0;
                z-index: 100;
            }

            .menu-toggle:checked ~ nav {
                display: flex;
            }

            nav a {
                display: flex;
                align-items: center;
                justify-content: center;
                min-height: 48px;
                width: 100%;
                margin: 0;
                padding: 0 1rem;
                border-bottom: 1px solid rgba(13, 59, 80, 0.5);
                font-size: 1rem;
            }

            nav a:last-child {
                border-bottom: none;
            }

            .container {
                padding: 0 16px;
            }

            main {
                padding: 2rem 0;
            }

            section {
                margin-bottom: 3.5rem;
            }

            #hero h1 {
                font-size: 1.4rem;
            }

            #hero p {
                font-size: 0.95rem;
                margin-bottom: 1.8rem;
            }

            .cta-buttons {
                flex-direction: column;
                gap: 1rem;
                width: 100%;
            }

            .btn {
                width: 100%;
                text-align: center;
                min-height: 48px;
                display: inline-flex;
                align-items: center;
                justify-content: center;
                padding: 0.8rem 1.5rem;
            }

            h2 {
                font-size: 1.5rem;
                margin-bottom: 1.8rem;
            }

            .grid-3 {
                grid-template-columns: 1fr;
                gap: 1.5rem;
            }

            .card {
                padding: 1.25rem;
                width: 100%;
            }

            .code-block {
                font-size: 0.8rem;
                padding: 1rem;
                margin: 1rem 0;
                overflow-x: auto;
                background-image: linear-gradient(to right, rgba(126, 200, 211, 0.15), transparent 15px), linear-gradient(to left, rgba(126, 200, 211, 0.2), transparent 15px);
                background-position: left center, right center;
                background-repeat: no-repeat;
                background-size: 15px 100%;
            }

            #roadmap li {
                padding: 1.25rem 1rem 1.25rem 3rem;
            }

            #roadmap li::before {
                left: -8px;
                font-size: 0.7rem;
            }
        }

        @media (max-width: 480px) {
            nav a {
                min-height: 56px;
            }

            #hero h1 {
                font-size: 1.2rem;
            }

            .code-block {
                font-size: 0.75rem;
            }

            .grid-3 {
                gap: 1rem;
            }
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
            <input type="checkbox" id="menu-toggle" class="menu-toggle" aria-label="Toggle navigation">
            <label for="menu-toggle" class="hamburger" aria-label="Open menu">&#9776;</label>
            <nav>
                <a href="/" class="active">Home</a>
                <a href="/explorer">Explorer</a>
                <a href="/testnet">Testnet</a>
                <a href="/launch">Launch</a>
                <a href="/future">Roadmap</a>
                <a href="/agents">Agents</a>
                <a href="/docs">Docs</a>
                <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" style="color: var(--spectral-blue);">GitHub</a>
            </nav>
        </header>

        <div class="top-logo">
            <div id="logo-container"><!-- SVG will be injected here --></div>
        </div>

        <main>
            <section id="hero">
                <div id="banner-container"><!-- SVG will be injected here --></div>
                <h1>The ghost in the IPv6 machine.</h1>
                <p>Zyanya is an IPv6-native, agent-native blockchain built on Spectre/GhostDAG. No gateways. No translators. Just pure, end-to-end decentralized consensus over the next-generation internet.</p>
                <div class="cta-buttons">
                    <a href="/explorer" target="_blank" class="btn btn-primary">LAUNCH EXPLORER</a>
                    <a href="#join" class="btn btn-secondary">JOIN THE TESTNET</a>
                </div>
            </section>

            <section id="status-banner">
                <p>🟣 Public testnet is LIVE &mdash; 3 nodes, <span id="testnet-blocks">50,000+</span> blocks, and mining over IPv6. <a href="/explorer" target="_blank">Explore the testnet &rarr;</a></p>
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
                        <p>Designed for longevity. A slow-burn emission schedule (50 ZYAN per block with a smooth geometric decay toward a ~28.7B ZYAN cap) combined with a permanent 50% gas fee burn creates a deflationary, sustainable economic model for the long term.</p>
                    </div>
                </div>
            </section>
            
            <section id="economics">
                <h2>ECONOMICS</h2>
                <div class="grid-3">
                     <div class="card">
                        <div class="icon-container" id="icon-coin"><h3>Total Supply</h3></div>
                        <p>A capped max supply of ~28.7 billion ZYAN, reached via a smooth geometric decay. 50 ZYAN per block. Zero premine — the genesis block has no outputs, so every coin is mined.</p>
                    </div>
                     <div class="card">
                        <div class="icon-container" id="icon-token"><h3>Fair Launch</h3></div>
                        <p>Zero premine. No team wallets, no foundation allocation, no investor stake. The genesis block has zero outputs — the network is funded purely by ongoing emission and fees.</p>
                    </div>
                     <div class="card">
                        <div class="icon-container" id="icon-burn"><h3><span class="burn-red">The Burn</span></h3></div>
                        <p>50% of all transaction fees are permanently burned. This deflationary pressure rewards long-term holders and increases the network's value over time.</p>
                    </div>
                </div>
            </section>

            <section id="join">
                <h2>HOW TO JOIN THE TESTNET</h2>
                <p>An IPv6-enabled connection is required. Download the latest distribution (Windows + Linux binaries and README) from the GitHub release to get started.</p>
                <p><a href="https://github.com/scotthawk-maker/zyanya/releases/tag/v0.3.17-testnet" target="_blank">Download Distribution Here</a></p>
                
                <h4>1. Run a Full Node</h4>
                <p>Use the <code>zyanyad</code> daemon. The <code>--connect</code> flag points to the seed node over IPv6; <code>--enable-unsynced-mining --utxoindex</code> lets the node accept blocks while it's still syncing. (The node syncs the chain — it doesn't mine on its own.)</p>
                <div class="code-block">
                    <code>zyanyad --testnet --connect=[2606:8ac0:2615:79aa:1a66:daff:fe99:31f7]:18211 --enable-unsynced-mining --utxoindex</code>
                </div>

                <h4>2. Generate a Mining Address</h4>
                <p>Create a <code>zyanyatest:</code> address + secret key. <strong>Save the secret key!</strong></p>
                <div class="code-block">
                    <code>gen-address --testnet</code>
                </div>

                <h4>3. Mine</h4>
                <p>Run the CPU miner pointing at <em>your local node's</em> RPC (127.0.0.1, not the seed), with your address. It mines while the node syncs.</p>
                <div class="code-block">
                    <code>zyanya-miner --testnet --mine-when-not-synced --cpu-percent 25 --zyanyad-address=127.0.0.1 --port=18210 --mining-address=zyanyatest:YOUR_ADDRESS</code>
                </div>

                <h4>4. Query the Network</h4>
                <p>Use <code>zyanya-query</code> against your local node, or the seed's public RPC.</p>
                <div class="code-block">
                    <code>zyanya-query --testnet --rpcserver [2606:8ac0:2615:79aa:1a66:daff:fe99:31f7]:18210 get-dag-info</code>
                </div>
            </section>

            <section id="ipv6-safety">
                <h2>IPv6: REWARDS & RISKS</h2>
                <p style="max-width:800px;margin:0 auto 2rem;">Zyanya is IPv6-native — every node is directly, globally addressable. That is the point (true peer-to-peer, no NAT, no gateways) and it is also a responsibility.</p>
                <div class="grid-3">
                    <div class="card">
                        <h3>THE REWARDS</h3>
                        <p>Pure end-to-end peer-to-peer consensus. No NAT, no port-forwarding, no gateways or translators. A vast, un-NAT-ed address space. Every node is a first-class peer — the decentralized internet as it was meant to be.</p>
                    </div>
                    <div class="card">
                        <h3>THE RISKS</h3>
                        <p>Without IPv4 NAT acting as an accidental firewall, your node is reachable from the public internet. A device that was “hidden” behind NAT is now directly addressable. You must consciously run a host firewall.</p>
                    </div>
                    <div class="card">
                        <h3>HARDEN YOUR NODE</h3>
                        <p>Filter inbound IPv6 — only expose the ports you intend (P2P <code>18211</code>, RPC <code>18210</code>). <strong>Do NOT block all ICMPv6</strong> — IPv6 needs it for Neighbor Discovery + Path MTU; blocking it breaks connectivity (see RFC 4890).</p>
                    </div>
                </div>
                <p style="margin-top:1.5rem;"><strong>Linux:</strong> <a href="https://wiki.archlinux.org/title/IPv6" target="_blank">Arch Wiki — IPv6</a> &bull; <a href="https://wiki.archlinux.org/title/Nftables" target="_blank">nftables</a> / <a href="https://wiki.archlinux.org/title/Uncomplicated_Firewall" target="_blank">ufw</a> &bull; <a href="https://datatracker.ietf.org/doc/html/rfc4890" target="_blank">RFC 4890 (ICMPv6 filtering)</a></p>
                <p><strong>Windows:</strong> <a href="https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/windows-firewall-with-advanced-security" target="_blank">Windows Defender Firewall with Advanced Security</a> (default profile blocks inbound IPv6 — add rules only for the Zyanya ports)</p>
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
            <p>&copy; 2026 Zyanya Project. All rights reserved. &bull; <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" style="color: var(--spectral-blue); text-decoration: none;">Source on GitHub</a></p>
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
            fetchAndInject('icon-coin', '/brand/zyan-coin.svg');
            fetchAndInject('icon-token', '/brand/zyanya-token-set.svg');
            fetchAndInject('icon-burn', '/brand/gas-burn-icon.svg');

            // Live testnet block count (graceful fallback to the static text)
            fetch('/api/info').then(r => r.json()).then(d => {
                const el = document.getElementById('testnet-blocks');
                if (el && d && d.block_count) el.textContent = Number(d.block_count).toLocaleString();
            }).catch(() => {});
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

        .menu-toggle { display: none; }
        .hamburger {
            display: none;
            font-size: 1.8rem;
            color: var(--accent-spectral);
            cursor: pointer;
            padding: 0.5rem 1rem;
            user-select: none;
            z-index: 101;
        }

        header {
            display: flex;
            justify-content: center;
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

        nav {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 1.5rem;
            flex-wrap: wrap;
        }

        nav a {
            color: var(--accent-spectral);
            text-decoration: none;
            font-weight: 600;
            font-size: 0.95rem;
            transition: color 0.3s ease;
            display: inline-flex;
            align-items: center;
        }

        nav a:hover, nav a.active {
            color: #FFFFFF;
            text-shadow: 0 0 8px var(--accent-spectral);
        }

        .tabs-bar {
            display: flex;
            gap: 0.5rem;
            flex-wrap: wrap;
            justify-content: center;
            margin-bottom: 1.5rem;
        }

        .table-responsive {
            width: 100%;
            overflow-x: auto;
            -webkit-overflow-scrolling: touch;
        }

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

        @media (max-width: 768px) {
            header {
                flex-direction: column;
                padding: 0.8rem 1rem;
            }

            .hamburger {
                display: block;
            }

            nav {
                display: none;
                flex-direction: column;
                width: 100%;
                background: rgba(10, 15, 28, 0.98);
                border: 1px solid rgba(126, 200, 211, 0.3);
                border-radius: 8px;
                margin-top: 0.5rem;
                padding: 0.5rem 0;
                gap: 0;
            }

            .menu-toggle:checked ~ nav {
                display: flex;
            }

            nav a {
                display: flex;
                align-items: center;
                justify-content: center;
                min-height: 48px;
                width: 100%;
                padding: 0 1rem;
                border-bottom: 1px solid rgba(13, 59, 80, 0.5);
                font-size: 1rem;
            }

            nav a:last-child {
                border-bottom: none;
            }

            .container {
                padding: 1rem 16px;
            }

            .stats-grid {
                grid-template-columns: 1fr 1fr;
                gap: 1rem;
            }

            .stat-card {
                padding: 1rem;
            }

            .card {
                padding: 1rem;
            }

            .card-header {
                flex-direction: column;
                align-items: flex-start;
                gap: 0.5rem;
            }

            .search-input {
                font-size: 16px;
                min-height: 48px;
            }

            .search-btn {
                min-height: 48px;
                padding: 0.8rem 1.5rem;
            }

            .nav-btn {
                min-height: 48px;
                display: inline-flex;
                align-items: center;
                justify-content: center;
            }

            th, td {
                padding: 0.6rem 0.75rem;
                font-size: 0.8rem;
            }
        }

        @media (max-width: 480px) {
            nav a {
                min-height: 56px;
            }

            .stats-grid {
                grid-template-columns: 1fr;
                gap: 0.75rem;
            }

            .tabs-bar {
                flex-direction: column;
                width: 100%;
            }

            .tabs-bar .nav-btn {
                width: 100%;
                text-align: center;
            }

            .search-box {
                flex-direction: column;
            }

            .search-btn {
                width: 100%;
            }
        }
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
        <input type="checkbox" id="menu-toggle" class="menu-toggle" aria-label="Toggle navigation">
        <label for="menu-toggle" class="hamburger" aria-label="Open menu">&#9776;</label>
        <nav>
            <a href="/">Home</a>
            <a href="/explorer" class="active">Explorer</a>
            <a href="/testnet">Testnet</a>
            <a href="/launch">Launch</a>
            <a href="/future">Roadmap</a>
            <a href="/agents">Agents</a>
            <a href="/docs">Docs</a>
            <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" style="color: var(--accent-spectral);">GitHub</a>
        </nav>
    </header>

    <main class="container">
        <div class="top-logo" style="text-align:center; margin-top: 1.5rem; margin-bottom: 1rem;">
            <div id="explorer-logo" style="display:inline-block;"></div>
        </div>

        <div class="tabs-bar">
            <button class="nav-btn mono active" onclick="switchTab('dashboard')">DASHBOARD</button>
            <button class="nav-btn mono" onclick="switchTab('contracts')">CONTRACTS</button>
            <button class="nav-btn mono" onclick="switchTab('tokens')">TOKENS</button>
            <button class="nav-btn mono" onclick="switchTab('dex')">DEX</button>
            <button class="nav-btn mono" onclick="switchTab('dag')">DAG GRAPH</button>
            <a href="/tools" class="nav-btn mono" style="text-decoration: none; display: inline-flex; align-items: center; justify-content: center;">WEBMCP TOOLS</a>
        </div>

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
                <div class="table-responsive">
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
                    <div class="table-responsive">
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
                    <div class="table-responsive">
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
                    <div class="table-responsive">
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
        name: "ipv6-safety",
        description: "Return Zyanya's IPv6 peer-to-peer safety guidance: the rewards of IPv6-native P2P, the risks of being globally addressable, and host-firewall hardening steps + links (Linux nftables/ufw, Windows Defender Firewall, RFC 4890 ICMPv6).",
        inputSchema: { type: "object", properties: {} },
        execute: async () => ({
            rewards: "Pure end-to-end peer-to-peer consensus. No NAT, no port-forwarding, no gateways. Every node is a first-class, globally addressable peer.",
            risks: "Without IPv4 NAT as an accidental firewall, your node is directly reachable from the public internet. You must run a host firewall.",
            hardening: [
                "Filter inbound IPv6; only expose the ports you intend (P2P 18211, RPC 18210).",
                "Do NOT block all ICMPv6 — IPv6 needs it for Neighbor Discovery and Path MTU Discovery; blocking it breaks connectivity (RFC 4890).",
                "Use a stable/assigned IPv6 address for a node, or a privacy/temporary address if you prefer."
            ],
            links: {
                linux: [
                    { name: "Arch Wiki — IPv6", url: "https://wiki.archlinux.org/title/IPv6" },
                    { name: "nftables", url: "https://wiki.archlinux.org/title/Nftables" },
                    { name: "ufw", url: "https://wiki.archlinux.org/title/Uncomplicated_Firewall" },
                    { name: "RFC 4890 — ICMPv6 filtering", url: "https://datatracker.ietf.org/doc/html/rfc4890" }
                ],
                windows: [
                    { name: "Windows Defender Firewall with Advanced Security", url: "https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/windows-firewall-with-advanced-security" }
                ]
            }
        })
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

        .menu-toggle { display: none; }
        .hamburger {
            display: none;
            font-size: 1.8rem;
            color: var(--accent-spectral);
            cursor: pointer;
            padding: 0.5rem 1rem;
            user-select: none;
            z-index: 101;
        }

        header {
            display: flex;
            justify-content: center;
            align-items: center;
            padding: 1.2rem 2.5rem;
            border-bottom: 1px solid rgba(126, 200, 211, 0.2);
            background: rgba(10, 15, 28, 0.95);
            backdrop-filter: blur(10px);
            position: sticky;
            top: 0;
            z-index: 100;
        }

        .ipv6-badge {
            background: rgba(126, 200, 211, 0.1);
            border: 1px solid var(--accent-spectral);
            color: var(--accent-spectral);
            padding: 0.25rem 0.75rem;
            border-radius: 20px;
            font-size: 0.8rem;
            letter-spacing: 1px;
        }

        nav {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 1.5rem;
            flex-wrap: wrap;
        }

        nav a {
            color: var(--accent-spectral);
            text-decoration: none;
            font-weight: 600;
            font-size: 0.95rem;
            transition: color 0.3s ease;
            display: inline-flex;
            align-items: center;
        }

        nav a:hover, nav a.active {
            color: #FFFFFF;
            text-shadow: 0 0 8px var(--accent-spectral);
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

        @media (max-width: 768px) {
            header {
                flex-direction: column;
                padding: 0.8rem 1rem;
            }

            .hamburger {
                display: block;
            }

            nav {
                display: none;
                flex-direction: column;
                width: 100%;
                background: rgba(10, 15, 28, 0.98);
                border: 1px solid rgba(126, 200, 211, 0.3);
                border-radius: 8px;
                margin-top: 0.5rem;
                padding: 0.5rem 0;
                gap: 0;
            }

            .menu-toggle:checked ~ nav {
                display: flex;
            }

            nav a {
                display: flex;
                align-items: center;
                justify-content: center;
                min-height: 48px;
                width: 100%;
                padding: 0 1rem;
                border-bottom: 1px solid rgba(13, 59, 80, 0.5);
                font-size: 1rem;
            }

            nav a:last-child {
                border-bottom: none;
            }

            .container {
                padding: 1rem 16px;
            }

            .hero-card {
                padding: 1.5rem;
            }

            .hero-title {
                font-size: 1.4rem;
            }

            .hero-stats {
                flex-direction: column;
                gap: 0.75rem;
            }

            .inspector-card {
                padding: 1.25rem;
            }

            .tools-grid {
                grid-template-columns: 1fr;
                gap: 1rem;
            }

            select.form-input, textarea.form-input, input.form-input {
                font-size: 16px;
                min-height: 48px;
            }

            .btn {
                width: 100%;
                text-align: center;
                min-height: 48px;
                display: inline-flex;
                align-items: center;
                justify-content: center;
                padding: 0.8rem 1.5rem;
            }
        }

        @media (max-width: 480px) {
            nav a {
                min-height: 56px;
            }

            .hero-title {
                font-size: 1.2rem;
            }

            .output-box {
                font-size: 0.75rem;
            }
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
        <input type="checkbox" id="menu-toggle" class="menu-toggle" aria-label="Toggle navigation">
        <label for="menu-toggle" class="hamburger" aria-label="Open menu">&#9776;</label>
        <nav>
            <a href="/">Home</a>
            <a href="/explorer">Explorer</a>
            <a href="/testnet">Testnet</a>
            <a href="/launch">Launch</a>
            <a href="/future">Roadmap</a>
            <a href="/agents">Agents</a>
            <a href="/docs">Docs</a>
            <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" style="color: var(--accent-spectral);">GitHub</a>
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

pub const TESTNET_HTML: &str = r###"<!DOCTYPE html>
<html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>Zyanya Testnet — All-in-One Setup</title>
<link href="https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;600&display=swap" rel="stylesheet">
<style>
:root{--void:#0A0F1C;--shadow-teal:#0D3B50;--spectral-blue:#7EC8D3;--text:#E0E0E0;--burn:#FF4D4D;--font-mono:'Fira Code',monospace}
*{box-sizing:border-box;margin:0;padding:0}html,body{background:var(--void);color:var(--text);font-family:var(--font-mono);line-height:1.6;overflow-x:hidden}
.container{max-width:900px;margin:0 auto;padding:0 20px}
header{display:flex;justify-content:center;align-items:center;padding:1.5rem 0;border-bottom:1px solid var(--shadow-teal);position:relative;width:100%}
.menu-toggle{display:none}
.hamburger{display:none;font-size:1.8rem;color:var(--spectral-blue);cursor:pointer;padding:.5rem 1rem;user-select:none;z-index:101}
nav{display:flex;justify-content:center;align-items:center;gap:1.5rem;flex-wrap:wrap}
nav a{color:var(--spectral-blue);text-decoration:none;font-weight:600;font-size:.95rem;transition:color .3s ease;display:inline-flex;align-items:center}
nav a:hover,nav a.active{color:#fff;text-shadow:0 0 8px var(--spectral-blue)}
.top-logo{text-align:center;margin:2rem 0 1rem}.top-logo svg{max-width:520px;width:100%;height:auto}
main{padding:2rem 0 4rem}section{margin-bottom:4rem;text-align:center}
h1{font-size:2rem;color:var(--spectral-blue);margin-bottom:.5rem}h2{font-size:1.6rem;margin-bottom:1.5rem;text-transform:uppercase;letter-spacing:2px}
h4{color:var(--spectral-blue);margin:2rem 0 .5rem;text-align:left;font-size:1.1rem}p{text-align:left;margin-bottom:1rem}
.code-block{background:var(--void);border:1px solid var(--shadow-teal);border-radius:4px;padding:1rem;text-align:left;overflow-x:auto;margin:1rem 0;background-image:linear-gradient(to right,rgba(126,200,211,.15),transparent 15px),linear-gradient(to left,rgba(126,200,211,.2),transparent 15px);background-position:left center,right center;background-repeat:no-repeat;background-size:15px 100%}
code{font-family:var(--font-mono);font-size:.9rem;color:var(--spectral-blue);white-space:pre}
.callout{background:rgba(13,59,80,.4);border:1px solid var(--shadow-teal);border-radius:8px;padding:1rem 1.5rem;margin:1.5rem auto;text-align:left}
.callout strong{color:var(--spectral-blue)}
a{color:var(--spectral-blue)}.btn{display:inline-block;padding:.8rem 1.8rem;border-radius:4px;font-weight:600;text-decoration:none;margin:.5rem;min-height:48px}
.btn-primary{background:var(--spectral-blue);color:var(--void)}.btn-secondary{border:2px solid var(--shadow-teal);color:var(--text)}
footer{text-align:center;padding:2rem 0;border-top:1px solid var(--shadow-teal);color:rgba(224,224,224,.5);font-size:.85rem}
.choice{display:flex;gap:2rem;flex-wrap:wrap;justify-content:center;margin:1.5rem 0}
.choice>div{flex:1;min-width:280px;text-align:left;background:rgba(13,59,80,.25);border:1px solid var(--shadow-teal);border-radius:8px;padding:1.5rem}
.badge{display:inline-block;font-size:.7rem;padding:.2rem .5rem;border-radius:3px;text-transform:uppercase;letter-spacing:1px;margin-left:.5rem}
.badge-live{background:var(--spectral-blue);color:var(--void)}.badge-planned{background:transparent;border:1px solid var(--text);color:var(--text)}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:1.5rem;text-align:left}
.card{background:var(--shadow-teal);padding:1.5rem;border-radius:8px;border:1px solid rgba(126,200,211,.2)}
.card h3{color:var(--spectral-blue);margin-bottom:.5rem}
@media (max-width:768px){
header{flex-direction:column;padding:1rem 0}
.hamburger{display:block}
nav{display:none;flex-direction:column;width:100%;background:rgba(10,15,28,.98);border:1px solid var(--shadow-teal);border-radius:8px;margin-top:.5rem;padding:.5rem 0;gap:0;z-index:100}
.menu-toggle:checked ~ nav{display:flex}
nav a{display:flex;align-items:center;justify-content:center;min-height:48px;width:100%;margin:0;padding:0 1rem;border-bottom:1px solid rgba(13,59,80,.5);font-size:1rem}
nav a:last-child{border-bottom:none}
.container{padding:0 16px}
h1{font-size:1.4rem}h2{font-size:1.3rem}
.choice{flex-direction:column;gap:1rem}
.choice>div{width:100%;min-width:unset;padding:1.25rem}
.code-block{font-size:.8rem;padding:1rem}
code{font-size:.8rem}
.btn{width:100%;text-align:center;min-height:48px;display:inline-flex;align-items:center;justify-content:center;margin:.5rem 0}
}
@media (max-width:480px){
nav a{min-height:56px}
h1{font-size:1.2rem}
.code-block{font-size:.75rem}
code{font-size:.75rem}
}
</style></head><body>
<div class="container"><header>
<input type="checkbox" id="menu-toggle" class="menu-toggle" aria-label="Toggle navigation">
<label for="menu-toggle" class="hamburger" aria-label="Open menu">&#9776;</label>
<nav>
<a href="/">Home</a><a href="/explorer">Explorer</a><a href="/testnet" class="active">Testnet</a><a href="/launch">Launch</a><a href="/future">Roadmap</a><a href="/agents">Agents</a><a href="/docs">Docs</a><a href="https://github.com/scotthawk-maker/zyanya" target="_blank" style="color:var(--spectral-blue)">GitHub</a>
</nav></header>
<div class="top-logo"><div id="logo-container"></div></div>
<main>
<section><h1>Join the Zyanya Testnet</h1><p style="text-align:center;max-width:640px;margin:0 auto 1.5rem">One flow — run a node, get a wallet, mine, and transact. Testnet coins have no value; break things and tell us what you find.</p></section>

<div class="callout"><strong>Prerequisite:</strong> an IPv6-enabled connection. Zyanya is IPv6-native — the seed, the explorer, and the P2P layer all speak IPv6. See the <a href="/#ipv6-safety">IPv6 safety guide</a> to harden your node first.</div>

<section><h2>1 · Download</h2>
<p>Grab the binaries from the GitHub release — Windows zip or Linux tarball (includes zyanyad, zyanya-wallet, zyanya-miner, zyanya-query, gen-address + this README).</p>
<a class="btn btn-primary" href="https://github.com/scotthawk-maker/zyanya/releases/tag/v0.3.17-testnet" target="_blank">Download from GitHub</a></section>

<section><h2>2 · Run a full node</h2>
<p>Syns the chain and connects to the public seed over IPv6. <code>--enable-unsynced-mining --utxoindex</code> lets it accept blocks while still syncing.</p>
<div class="code-block"><code>zyanyad --testnet \
  --connect=[2606:8ac0:2615:79aa:1a66:daff:fe99:31f7]:18211 \
  --enable-unsynced-mining --utxoindex</code></div>
<p style="text-align:center;color:rgba(224,224,224,.6)">Or run it in Docker — see the README.</p></section>

<section><h2>3 · Get a wallet address</h2>
<p>On first run the wallet auto-generates a 24-word BIP-39 mnemonic, derives a <code>zyanyatest:</code> address, and saves it to <code>~/.zyanya/wallet.key</code> — then drops you into the TUI.</p>
<div class="code-block"><code>zyanya-wallet --testnet</code></div>
<p><strong>Write down the 24 words.</strong> For an optional 25th-word passphrase, create explicitly: <code>zyanya-wallet --testnet --generate-mnemonic --passphrase "your 25th word"</code>. (Just need a mining address fast? <code>gen-address --testnet</code> prints one without the full wallet.)</p></section>

<section><h2>4 · Mine — solo or pool</h2>
<p>Choose your mode. Point the miner at <em>your local node</em> (127.0.0.1, not the seed) with your address. <code>--cpu-percent 25</code> keeps your CPU sane.</p>
<div class="choice">
<div><h4 style="margin-top:0">⛏️ Solo</h4>
<div class="code-block"><code>zyanya-miner --testnet \
  --mine-when-not-synced --cpu-percent 25 \
  --zyanyad-address=127.0.0.1 --port=18210 \
  --mining-address=zyanyatest:YOUR_ADDRESS</code></div>
<p style="margin:0">Mines directly to your node. Rewards go straight to your wallet.</p></div>
<div><h4 style="margin-top:0">🏊 Pool (Stratum)</h4>
<div class="code-block"><code>zyanya-miner --testnet --cpu-percent 25 \
  --pool=[pool-ipv6]:3334</code></div>
<p style="margin:0">Connects to a Zyanya Stratum pool. (Run your own with <code>zyanya-pool</code>.)</p></div>
</div></section>

<section><h2>5 · Check balance &amp; send</h2>
<p>Back in the wallet TUI: <code>[1]</code> balances, <code>[2]</code> send ZYAN, <code>[5]</code> swap on the DEX. Or use the CLI.</p>
<div class="code-block"><code>zyanya-wallet --testnet --balance
zyanya-wallet --testnet --send-zyan --to zyanyatest:RECIPIENT --amount 10</code></div></section>

<div class="callout"><strong>Heads-up — 100-block maturity:</strong> mined rewards can't be spent until 100 blocks confirm. The wallet automatically skips immature coinbase UTXOs, so just wait ~100 blocks after mining before the balance is spendable.</div>
</main>
<footer><p>The ghost in the IPv6 machine. Forever, always. &bull; <a href="/">zyanya.scottcloudhawk.org</a></p></footer>
</div>
<script>fetch('/brand/zyanya-logo.svg').then(r=>r.text()).then(t=>{document.getElementById('logo-container').innerHTML=t;});</script>
</body></html>"###;

pub const FUTURE_HTML: &str = r###"<!DOCTYPE html>
<html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>Zyanya — Roadmap &amp; Features</title>
<link href="https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;600&display=swap" rel="stylesheet">
<style>
:root{--void:#0A0F1C;--shadow-teal:#0D3B50;--spectral-blue:#7EC8D3;--text:#E0E0E0;--burn:#FF4D4D;--font-mono:'Fira Code',monospace}
*{box-sizing:border-box;margin:0;padding:0}html,body{background:var(--void);color:var(--text);font-family:var(--font-mono);line-height:1.6;overflow-x:hidden}
.container{max-width:1000px;margin:0 auto;padding:0 20px}
header{display:flex;justify-content:center;align-items:center;padding:1.5rem 0;border-bottom:1px solid var(--shadow-teal);position:relative;width:100%}
.menu-toggle{display:none}
.hamburger{display:none;font-size:1.8rem;color:var(--spectral-blue);cursor:pointer;padding:.5rem 1rem;user-select:none;z-index:101}
nav{display:flex;justify-content:center;align-items:center;gap:1.5rem;flex-wrap:wrap}
nav a{color:var(--spectral-blue);text-decoration:none;font-weight:600;font-size:.95rem;transition:color .3s ease;display:inline-flex;align-items:center}
nav a:hover,nav a.active{color:#fff;text-shadow:0 0 8px var(--spectral-blue)}
.top-logo{text-align:center;margin:2rem 0 1rem}.top-logo svg{max-width:520px;width:100%;height:auto}
main{padding:2rem 0 4rem}section{margin-bottom:4rem;text-align:center}
h1{font-size:2rem;color:var(--spectral-blue);margin-bottom:.5rem}h2{font-size:1.6rem;margin-bottom:2rem;text-transform:uppercase;letter-spacing:2px}
p{text-align:left;margin-bottom:1rem}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:1.5rem;text-align:left}
.card{background:var(--shadow-teal);padding:1.75rem;border-radius:8px;border:1px solid rgba(126,200,211,.2)}
.card h3{color:var(--spectral-blue);margin-bottom:.4rem;font-size:1.1rem;display:flex;align-items:center;justify-content:space-between}
.card p{color:rgba(224,224,224,.8);font-size:.92rem;margin:0}
.badge{font-size:.65rem;padding:.2rem .5rem;border-radius:3px;text-transform:uppercase;letter-spacing:1px;font-weight:600}
.badge-live{background:var(--spectral-blue);color:var(--void)}.badge-planned{background:transparent;border:1px solid var(--text);color:var(--text)}
.phase{list-style:none;padding:0;max-width:760px;margin:0 auto;text-align:left}
.phase li{padding:1.25rem 1.25rem 1.25rem 3.5rem;margin-bottom:1rem;background:var(--shadow-teal);border-radius:8px;border-left:3px solid var(--spectral-blue);position:relative}
.phase li::before{content:"Phase 0" counter(phase-counter);counter-increment:phase-counter;position:absolute;left:-12px;top:50%;transform:translateY(-50%) rotate(-90deg);color:var(--spectral-blue);font-size:.75rem;font-weight:600}
.phase{counter-reset:phase-counter}.phase strong{display:block;margin-bottom:.3rem}
footer{text-align:center;padding:2rem 0;border-top:1px solid var(--shadow-teal);color:rgba(224,224,224,.5);font-size:.85rem}
a{color:var(--spectral-blue)}
@media (max-width:768px){
header{flex-direction:column;padding:1rem 0}
.hamburger{display:block}
nav{display:none;flex-direction:column;width:100%;background:rgba(10,15,28,.98);border:1px solid var(--shadow-teal);border-radius:8px;margin-top:.5rem;padding:.5rem 0;gap:0;z-index:100}
.menu-toggle:checked ~ nav{display:flex}
nav a{display:flex;align-items:center;justify-content:center;min-height:48px;width:100%;margin:0;padding:0 1rem;border-bottom:1px solid rgba(13,59,80,.5);font-size:1rem}
nav a:last-child{border-bottom:none}
.container{padding:0 16px}
h1{font-size:1.4rem}h2{font-size:1.3rem}
.grid{grid-template-columns:1fr;gap:1rem}
.card{padding:1.25rem}
.phase li{padding:1.25rem 1rem 1.25rem 3rem}
}
@media (max-width:480px){
nav a{min-height:56px}
h1{font-size:1.2rem}
}
</style></head><body>
<div class="container"><header>
<input type="checkbox" id="menu-toggle" class="menu-toggle" aria-label="Toggle navigation">
<label for="menu-toggle" class="hamburger" aria-label="Open menu">&#9776;</label>
<nav>
<a href="/">Home</a><a href="/explorer">Explorer</a><a href="/testnet">Testnet</a><a href="/launch">Launch</a><a href="/future" class="active">Roadmap</a><a href="/agents">Agents</a><a href="/docs">Docs</a><a href="https://github.com/scotthawk-maker/zyanya" target="_blank" style="color:var(--spectral-blue)">GitHub</a>
</nav></header>
<div class="top-logo"><div id="logo-container"></div></div>
<main>
<section><h1>The Path Forward</h1><p style="text-align:center;max-width:640px;margin:0 auto 1.5rem">What's live on testnet today, and what's planned. The ghost is awake — more is coming.</p></section>

<section><h2>Live on testnet</h2>
<div class="grid">
<div class="card"><h3>⛏️ CPU Mining <span class="badge badge-live">Live</span></h3><p>Solo mining to your own node, or Stratum pool mining via <code>zyanya-pool</code>. Adjustable hashrate (<code>--cpu-percent</code>), mines while syncing.</p></div>
<div class="card"><h3>🪙 Creating Tokens <span class="badge badge-live">Live</span></h3><p>Deploy custom tokens via the ZCL smart-contract VM. The GHOST token ships as the reference. Mint, transfer, and hold — all on-chain.</p></div>
<div class="card"><h3>🔄 The DEX <span class="badge badge-live">Live</span></h3><p>Swap ZYAN ↔ tokens in an on-chain liquidity pool. Add liquidity, set reserves, trade — the demo contract is live on testnet.</p></div>
<div class="card"><h3>📜 Smart Contracts <span class="badge badge-live">Live</span></h3><p>The <code>zyanya-vm</code>: an opcode VM + the ZCL compiler. Deploy contracts, invoke entry points, store state — deterministic consensus execution.</p></div>
<div class="card"><h3>🤖 Agent-Native (Web MCP) <span class="badge badge-live">Live</span></h3><p>The block explorer exposes a Web MCP — agents read chain state, query blocks, and (with the write flag) deploy/invoke contracts directly.</p></div>
<div class="card"><h3>👛 The Wallet <span class="badge badge-live">Live</span></h3><p>BIP-39 24-word (+ optional passphrase) TUI wallet. Send ZYAN, send tokens, swap on the DEX, view history. Secret-masked by default.</p></div>
</div></section>

<section><h2>Planned</h2>
<div class="grid">
<div class="card"><h3>🔒 Staking <span class="badge badge-planned">Planned</span></h3><p>Stake ZYAN to participate in network consensus/governance and earn rewards. Design in progress — details as the protocol matures.</p></div>
<div class="card"><h3>🚀 Mainnet Launch <span class="badge badge-planned">Planned</span></h3><p>Genesis mined silently, stability monitored, then the public reveal. Zero premine, fair launch.</p></div>
<div class="card"><h3>🌐 More <span class="badge badge-planned">Planned</span></h3><p>Hardened tooling, broader agent integrations, and whatever the IPv6 + ghost community asks for. The testnet is where we shake it out.</p></div>
</div></section>

<section><h2>The four phases</h2>
<ol class="phase">
<li><strong>Ghost in the Machine</strong><span style="color:rgba(224,224,224,.7)">Public testnet hardening — protocol improvements, bug fixes, network stability with the community. (Now.)</span></li>
<li><strong>Dark Launch</strong><span style="color:rgba(224,224,224,.7)">Mainnet genesis mined silently; initial stability monitoring by the core team.</span></li>
<li><strong>Prepare Optics</strong><span style="color:rgba(224,224,224,.7)">Finalize docs, exchange integrations, and communications. Ready for the public reveal.</span></li>
<li><strong>The r/IPv6 Signal</strong><span style="color:rgba(224,224,224,.7)">Public announcement to the wider technical community, starting with the IPv6 pioneers.</span></li>
</ol></section>
</main>
<footer><p>The ghost in the IPv6 machine. Forever, always. &bull; <a href="/">zyanya.scottcloudhawk.org</a></p></footer>
</div>
<script>fetch('/brand/zyanya-logo.svg').then(r=>r.text()).then(t=>{document.getElementById('logo-container').innerHTML=t;});</script>
</body></html>"###;

pub const LAUNCH_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Zyanya Launch - Pump.fun-style Token Launcher</title>
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
            --accent-green: #00FFAA;
            --font-mono: 'Fira Code', monospace;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; }
        html, body {
            background-color: var(--void);
            color: var(--text-color);
            font-family: var(--font-mono);
            font-size: 15px;
            line-height: 1.6;
            overflow-x: hidden;
        }
        .container { max-width: 900px; margin: 0 auto; padding: 0 20px; }
        header {
            display: flex; justify-content: center; align-items: center;
            padding: 1.5rem 0; border-bottom: 1px solid var(--shadow-teal);
            position: relative; width: 100%;
        }
        .menu-toggle { display: none; }
        .hamburger {
            display: none; font-size: 1.8rem; color: var(--spectral-blue);
            cursor: pointer; padding: 0.5rem 1rem; user-select: none; z-index: 101;
        }
        .logo-wrap { display: flex; align-items: center; text-decoration: none; gap: 10px; }
        nav {
            display: flex; justify-content: center; align-items: center;
            gap: 1.5rem; flex-wrap: wrap;
        }
        nav a {
            color: var(--spectral-blue); text-decoration: none;
            font-weight: 600; font-size: 0.95rem; transition: color 0.2s;
            display: inline-flex; align-items: center;
        }
        nav a:hover, nav a.active { color: #fff; text-shadow: 0 0 8px var(--spectral-blue); }
        main { padding: 3rem 0; }
        .hero-title { font-size: 2.2rem; color: var(--spectral-blue); margin-bottom: 0.5rem; text-align: center; }
        .hero-subtitle { text-align: center; color: rgba(224,224,224,0.7); margin-bottom: 2.5rem; }
        .card {
            background: var(--shadow-teal); padding: 2rem; border-radius: 12px;
            border: 1px solid rgba(126,200,211,0.3); box-shadow: 0 8px 32px rgba(0,0,0,0.4);
        }
        .form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1.2rem; }
        .form-group { display: flex; flex-direction: column; gap: 0.4rem; }
        .form-group.full { grid-column: span 2; }
        label { color: var(--spectral-blue); font-size: 0.85rem; font-weight: 600; text-transform: uppercase; letter-spacing: 1px; }
        input, textarea, select {
            background: rgba(10, 15, 28, 0.8); border: 1px solid rgba(126,200,211,0.3);
            color: #fff; font-family: var(--font-mono); padding: 0.75rem 1rem; border-radius: 6px;
            font-size: 16px; min-height: 48px; outline: none; transition: border-color 0.2s;
            width: 100%;
        }
        input[type="file"] {
            padding: 0.6rem;
            cursor: pointer;
        }
        input:focus, textarea:focus { border-color: var(--spectral-blue); box-shadow: 0 0 8px rgba(126,200,211,0.3); }
        textarea { resize: vertical; min-height: 90px; }
        .btn-launch {
            grid-column: span 2; background: linear-gradient(135deg, var(--spectral-blue), #4a90e2);
            color: var(--void); font-family: var(--font-mono); font-weight: 700; font-size: 1.1rem;
            padding: 0.8rem 1.5rem; min-height: 48px; border: none; border-radius: 8px; cursor: pointer; text-transform: uppercase;
            letter-spacing: 2px; transition: transform 0.2s, box-shadow 0.2s; margin-top: 1rem;
            display: inline-flex; align-items: center; justify-content: center;
        }
        .btn-launch:hover { transform: translateY(-2px); box-shadow: 0 0 20px rgba(126,200,211,0.6); }
        #status-msg { margin-top: 1.5rem; text-align: center; }
        footer { text-align: center; padding: 2rem 0; color: rgba(224,224,224,0.5); border-top: 1px solid var(--shadow-teal); margin-top: 4rem; }

        @media (max-width: 768px) {
            header { flex-direction: column; padding: 1rem 0; }
            .hamburger { display: block; }
            nav {
                display: none; flex-direction: column; width: 100%;
                background: rgba(10, 15, 28, 0.98); border: 1px solid var(--shadow-teal);
                border-radius: 8px; margin-top: 0.5rem; padding: 0.5rem 0; gap: 0; z-index: 100;
            }
            .menu-toggle:checked ~ nav { display: flex; }
            nav a {
                display: flex; align-items: center; justify-content: center;
                min-height: 48px; width: 100%; margin: 0; padding: 0 1rem;
                border-bottom: 1px solid rgba(13, 59, 80, 0.5); font-size: 1rem;
            }
            nav a:last-child { border-bottom: none; }
            .container { padding: 0 16px; }
            .hero-title { font-size: 1.4rem; }
            .form-grid { grid-template-columns: 1fr; gap: 1rem; }
            .form-group.full { grid-column: span 1; }
            .btn-launch { grid-column: span 1; width: 100%; }
            .card { padding: 1.25rem; }
        }

        @media (max-width: 480px) {
            nav a { min-height: 56px; }
            .hero-title { font-size: 1.2rem; }
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <input type="checkbox" id="menu-toggle" class="menu-toggle" aria-label="Toggle navigation">
            <label for="menu-toggle" class="hamburger" aria-label="Open menu">&#9776;</label>
            <nav>
                <a href="/">Home</a>
                <a href="/explorer">Explorer</a>
                <a href="/testnet">Testnet</a>
                <a href="/launch" class="active">Launch</a>
                <a href="/future">Roadmap</a>
                <a href="/agents">Agents</a>
                <a href="/docs">Docs</a>
                <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" style="color: var(--spectral-blue);">GitHub</a>
            </nav>
        </header>
        <main>
            <h1 class="hero-title">🚀 Bonding Curve Token Launcher</h1>
            <p class="hero-subtitle">Launch your token instantly on Zyanya. Instant bonding curve pricing on-chain.</p>
            
            <div class="card">
                <form id="launchForm" onsubmit="handleLaunch(event)">
                    <div class="form-grid">
                        <div class="form-group">
                            <label for="name">Token Name *</label>
                            <input type="text" id="name" placeholder="e.g. Spectral Ghost" required>
                        </div>
                        <div class="form-group">
                            <label for="symbol">Ticker Symbol *</label>
                            <input type="text" id="symbol" placeholder="e.g. GHOST" required>
                        </div>
                        <div class="form-group">
                            <label for="supply">Initial Reserve Supply</label>
                            <input type="number" id="supply" value="1000000" min="1">
                        </div>
                        <div class="form-group">
                            <label for="slope">Bonding Curve Slope (Price Multiplier)</label>
                            <input type="number" id="slope" value="1" min="1">
                        </div>
                        <div class="form-group full">
                            <label for="description">Description</label>
                            <textarea id="description" placeholder="What is this token about?"></textarea>
                        </div>
                        <div class="form-group full">
                            <label for="iconFile">Token Icon (PNG / Image)</label>
                            <input type="file" id="iconFile" accept="image/*">
                        </div>
                        <div class="form-group">
                            <label for="twitter">Twitter / X URL</label>
                            <input type="text" id="twitter" placeholder="https://x.com/yourtoken">
                        </div>
                        <div class="form-group">
                            <label for="telegram">Telegram URL</label>
                            <input type="text" id="telegram" placeholder="https://t.me/yourtoken">
                        </div>
                        <div class="form-group full">
                            <label for="website">Website URL</label>
                            <input type="url" id="website" placeholder="https://yourtoken.io">
                        </div>
                        <button type="submit" class="btn-launch">DEPLOY TOKEN</button>
                    </div>
                </form>
                <div id="status-msg"></div>
            </div>
        </main>
        <footer>
            <p>The ghost in the IPv6 machine. &bull; <a href="/">zyanya.scottcloudhawk.org</a></p>
        </footer>
    </div>

    <script>
        async function handleLaunch(event) {
            event.preventDefault();
            const name = document.getElementById('name').value;
            const symbol = document.getElementById('symbol').value;
            const supply = parseInt(document.getElementById('supply').value) || 1000000;
            const slope = parseInt(document.getElementById('slope').value) || 1;
            const description = document.getElementById('description').value;
            const twitter = document.getElementById('twitter').value;
            const telegram = document.getElementById('telegram').value;
            const website = document.getElementById('website').value;
            const iconInput = document.getElementById('iconFile');

            let icon_base64 = null;
            if (iconInput.files && iconInput.files[0]) {
                icon_base64 = await new Promise((resolve) => {
                    const reader = new FileReader();
                    reader.onload = (e) => resolve(e.target.result);
                    reader.readAsDataURL(iconInput.files[0]);
                });
            }

            const payload = {
                name, symbol, supply, slope, description, twitter, telegram, website, icon_base64
            };

            const statusEl = document.getElementById('status-msg');
            statusEl.innerHTML = '<span style="color: var(--spectral-blue)">Deploying bonding curve contract...</span>';

            try {
                const res = await fetch('/api/deploy-token', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(payload)
                });
                const data = await res.json();
                if (res.ok && (data.contract_address || data.contractAddress)) {
                    const addr = data.contract_address || data.contractAddress;
                    statusEl.innerHTML = `
                        <div style="background: rgba(0, 255, 170, 0.1); border: 1px solid var(--accent-green); padding: 18px; border-radius: 8px; margin-top: 15px; text-align: left;">
                            <h3 style="color: var(--accent-green); margin-bottom: 8px;">🚀 Token Successfully Launched!</h3>
                            <p><strong>Contract Address:</strong> <code style="word-break: break-all; color: var(--spectral-blue);">${addr}</code></p>
                            <a href="/token/${addr}" class="btn-launch" style="display: inline-block; margin-top: 12px; padding: 10px 20px; text-decoration: none; text-align: center;">VIEW TOKEN PAGE →</a>
                        </div>
                    `;
                } else {
                    statusEl.innerHTML = `<span style="color: var(--burn-red)">Error: ${data.error || 'Launch failed'}</span>`;
                }
            } catch (err) {
                statusEl.innerHTML = `<span style="color: var(--burn-red)">Network error: ${err.message}</span>`;
            }
        }
    </script>
</body>
</html>"#;

pub const TOKEN_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Token Details - Zyanya Explorer</title>
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
            --accent-green: #00FFAA;
            --font-mono: 'Fira Code', monospace;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; }
        html, body {
            background-color: var(--void);
            color: var(--text-color);
            font-family: var(--font-mono);
            font-size: 15px;
            line-height: 1.6;
            overflow-x: hidden;
        }
        .container { max-width: 1000px; margin: 0 auto; padding: 0 20px; }
        header {
            display: flex; justify-content: center; align-items: center;
            padding: 1.5rem 0; border-bottom: 1px solid var(--shadow-teal);
            position: relative; width: 100%;
        }
        .menu-toggle { display: none; }
        .hamburger {
            display: none; font-size: 1.8rem; color: var(--spectral-blue);
            cursor: pointer; padding: 0.5rem 1rem; user-select: none; z-index: 101;
        }
        .logo-wrap { display: flex; align-items: center; text-decoration: none; gap: 10px; }
        nav {
            display: flex; justify-content: center; align-items: center;
            gap: 1.5rem; flex-wrap: wrap;
        }
        nav a {
            color: var(--spectral-blue); text-decoration: none;
            font-weight: 600; font-size: 0.95rem; transition: color 0.2s;
            display: inline-flex; align-items: center;
        }
        nav a:hover, nav a.active { color: #fff; text-shadow: 0 0 8px var(--spectral-blue); }
        main { padding: 2.5rem 0; }
        .token-header-card {
            background: var(--shadow-teal); padding: 2rem; border-radius: 12px;
            border: 1px solid rgba(126,200,211,0.3); display: flex; align-items: center; gap: 2rem;
            margin-bottom: 2rem;
        }
        .token-icon {
            width: 100px; height: 100px; border-radius: 50%; object-fit: cover;
            border: 2px solid var(--spectral-blue); background: var(--void); display: flex;
            align-items: center; justify-content: center; font-size: 2.5rem; font-weight: 700;
            color: var(--spectral-blue);
        }
        .token-info-main h1 { font-size: 2rem; color: #fff; margin-bottom: 0.2rem; }
        .token-symbol { color: var(--spectral-blue); font-weight: 600; font-size: 1.1rem; margin-bottom: 0.5rem; }
        .token-desc { color: rgba(224,224,224,0.8); margin-bottom: 1rem; max-width: 600px; }
        .social-links a {
            display: inline-block; margin-right: 1rem; color: var(--spectral-blue);
            text-decoration: none; font-size: 0.85rem; padding: 4px 10px;
            border: 1px solid rgba(126,200,211,0.4); border-radius: 4px; transition: background 0.2s;
        }
        .social-links a:hover { background: rgba(126,200,211,0.2); }
        .stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1.2rem; margin-bottom: 2rem; }
        .stat-card {
            background: rgba(13, 59, 80, 0.6); padding: 1.2rem; border-radius: 8px;
            border: 1px solid rgba(126,200,211,0.2);
        }
        .stat-title { color: rgba(224,224,224,0.6); font-size: 0.8rem; text-transform: uppercase; letter-spacing: 1px; }
        .stat-val { color: var(--spectral-blue); font-size: 1.4rem; font-weight: 700; margin-top: 0.3rem; }
        .trade-container { display: grid; grid-template-columns: 1fr 1fr; gap: 1.5rem; }
        .trade-card {
            background: var(--shadow-teal); padding: 1.5rem; border-radius: 12px;
            border: 1px solid rgba(126,200,211,0.3);
        }
        .trade-card h3 { color: var(--spectral-blue); margin-bottom: 1rem; text-transform: uppercase; letter-spacing: 1px; }
        .trade-group { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1rem; }
        label { color: rgba(224,224,224,0.8); font-size: 0.85rem; }
        input {
            background: rgba(10, 15, 28, 0.8); border: 1px solid rgba(126,200,211,0.3);
            color: #fff; font-family: var(--font-mono); padding: 0.75rem 1rem; border-radius: 6px;
            font-size: 16px; min-height: 48px; outline: none; width: 100%;
        }
        .btn-buy {
            background: var(--accent-green); color: var(--void); font-family: var(--font-mono);
            font-weight: 700; font-size: 1rem; padding: 0.8rem 1.5rem; min-height: 48px; border: none; border-radius: 6px;
            cursor: pointer; width: 100%; text-transform: uppercase; letter-spacing: 1px;
            display: inline-flex; align-items: center; justify-content: center;
        }
        .btn-sell {
            background: var(--burn-red); color: #fff; font-family: var(--font-mono);
            font-weight: 700; font-size: 1rem; padding: 0.8rem 1.5rem; min-height: 48px; border: none; border-radius: 6px;
            cursor: pointer; width: 100%; text-transform: uppercase; letter-spacing: 1px;
            display: inline-flex; align-items: center; justify-content: center;
        }
        .status-box { margin-top: 1rem; font-size: 0.9rem; text-align: center; }
        footer { text-align: center; padding: 2rem 0; color: rgba(224,224,224,0.5); border-top: 1px solid var(--shadow-teal); margin-top: 4rem; }

        @media (max-width: 768px) {
            header { flex-direction: column; padding: 1rem 0; }
            .hamburger { display: block; }
            nav {
                display: none; flex-direction: column; width: 100%;
                background: rgba(10, 15, 28, 0.98); border: 1px solid var(--shadow-teal);
                border-radius: 8px; margin-top: 0.5rem; padding: 0.5rem 0; gap: 0; z-index: 100;
            }
            .menu-toggle:checked ~ nav { display: flex; }
            nav a {
                display: flex; align-items: center; justify-content: center;
                min-height: 48px; width: 100%; margin: 0; padding: 0 1rem;
                border-bottom: 1px solid rgba(13, 59, 80, 0.5); font-size: 1rem;
            }
            nav a:last-child { border-bottom: none; }
            .container { padding: 0 16px; }
            .token-header-card { flex-direction: column; text-align: center; gap: 1rem; padding: 1.5rem; }
            .token-desc { max-width: 100%; }
            .stats-grid { grid-template-columns: 1fr 1fr; gap: 1rem; }
            .trade-container { grid-template-columns: 1fr; gap: 1.5rem; }
            .social-links { display: flex; flex-wrap: wrap; justify-content: center; gap: 0.5rem; }
            .social-links a { margin-right: 0; min-height: 44px; display: inline-flex; align-items: center; padding: 0.5rem 1rem; }
        }

        @media (max-width: 480px) {
            nav a { min-height: 56px; }
            .stats-grid { grid-template-columns: 1fr; gap: 0.75rem; }
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <input type="checkbox" id="menu-toggle" class="menu-toggle" aria-label="Toggle navigation">
            <label for="menu-toggle" class="hamburger" aria-label="Open menu">&#9776;</label>
            <nav>
                <a href="/">Home</a>
                <a href="/explorer">Explorer</a>
                <a href="/testnet">Testnet</a>
                <a href="/launch" class="active">Launch</a>
                <a href="/future">Roadmap</a>
                <a href="/agents">Agents</a>
                <a href="/docs">Docs</a>
                <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" style="color: var(--spectral-blue);">GitHub</a>
            </nav>
        </header>

        <main>
            <div class="token-header-card">
                <div id="icon-container">
                    <div class="token-icon" id="icon-fallback">?</div>
                </div>
                <div class="token-info-main">
                    <h1 id="token-name">Loading...</h1>
                    <div class="token-symbol" id="token-symbol">...</div>
                    <p class="token-desc" id="token-desc">Loading token metadata...</p>
                    <div class="social-links" id="social-links"></div>
                </div>
            </div>

            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-title">Current Price</div>
                    <div class="stat-val" id="stat-price">0 ZYAN</div>
                </div>
                <div class="stat-card">
                    <div class="stat-title">Total Supply</div>
                    <div class="stat-val" id="stat-supply">0</div>
                </div>
                <div class="stat-card">
                    <div class="stat-title">Your Balance</div>
                    <div class="stat-val" id="stat-balance">0</div>
                </div>
                <div class="stat-card">
                    <div class="stat-title">Active Caller Key</div>
                    <div style="margin-top: 0.3rem;"><input type="text" id="caller-input" value="1" style="padding: 4px 8px; font-size: 0.9rem;" onchange="loadTokenData()"></div>
                </div>
            </div>

            <div class="trade-container">
                <div class="trade-card">
                    <h3>🟢 Buy Tokens</h3>
                    <div class="trade-group">
                        <label for="buy-amount">Tokens to Buy</label>
                        <input type="number" id="buy-amount" value="100" min="1">
                    </div>
                    <button class="btn-buy" onclick="handleBuy()">BUY NOW</button>
                    <div class="status-box" id="buy-status"></div>
                </div>

                <div class="trade-card">
                    <h3>🔴 Sell Tokens</h3>
                    <div class="trade-group">
                        <label for="sell-amount">Tokens to Sell</label>
                        <input type="number" id="sell-amount" value="100" min="1">
                    </div>
                    <button class="btn-sell" onclick="handleSell()">SELL NOW</button>
                    <div class="status-box" id="sell-status"></div>
                </div>
            </div>
        </main>

        <footer>
            <p>The ghost in the IPv6 machine. &bull; <a href="/">zyanya.scottcloudhawk.org</a></p>
        </footer>
    </div>

    <script>
        const contractAddress = window.location.pathname.split('/').pop();

        async function loadTokenData() {
            if (!contractAddress || contractAddress === 'token') return;
            const caller = document.getElementById('caller-input').value || '1';

            try {
                const res = await fetch('/api/token/' + contractAddress + '/metadata');
                if (res.ok) {
                    const meta = await res.json();
                    document.getElementById('token-name').innerText = meta.name || 'Bonding Curve Token';
                    document.getElementById('token-symbol').innerText = meta.symbol ? '$' + meta.symbol : '';
                    document.getElementById('token-desc').innerText = meta.description || 'No description provided.';
                    
                    const iconUri = meta.icon_uri || ('/token-icons/' + contractAddress + '.png');
                    const img = new Image();
                    img.src = iconUri;
                    img.className = 'token-icon';
                    img.onload = () => {
                        document.getElementById('icon-container').innerHTML = '';
                        document.getElementById('icon-container').appendChild(img);
                    };
                    img.onerror = () => {
                        document.getElementById('icon-fallback').innerText = (meta.symbol || meta.name || '?').charAt(0).toUpperCase();
                    };

                    const socialsHtml = [];
                    if (meta.twitter) socialsHtml.push(`<a href="${meta.twitter}" target="_blank">Twitter / X</a>`);
                    if (meta.telegram) socialsHtml.push(`<a href="${meta.telegram}" target="_blank">Telegram</a>`);
                    if (meta.website) socialsHtml.push(`<a href="${meta.website}" target="_blank">Website</a>`);
                    document.getElementById('social-links').innerHTML = socialsHtml.join('');
                }
            } catch (err) {
                console.error('Metadata load error:', err);
            }

            try {
                const res = await fetch('/api/call-contract', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ address: contractAddress, entry_point: 6 })
                });
                if (res.ok) {
                    const data = await res.json();
                    document.getElementById('stat-price').innerText = (data.returnValue || 0) + ' ZYAN';
                }
            } catch (err) {}

            try {
                const res = await fetch('/api/call-contract', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ address: contractAddress, entry_point: 3 })
                });
                if (res.ok) {
                    const data = await res.json();
                    document.getElementById('stat-supply').innerText = data.returnValue || 0;
                }
            } catch (err) {}

            try {
                const res = await fetch('/api/token-balance?token=' + contractAddress + '&holder=' + caller);
                if (res.ok) {
                    const data = await res.json();
                    document.getElementById('stat-balance').innerText = data.balance || 0;
                }
            } catch (err) {}
        }

        async function handleBuy() {
            const caller = document.getElementById('caller-input').value || '1';
            const amount = parseInt(document.getElementById('buy-amount').value) || 0;
            const statusEl = document.getElementById('buy-status');
            statusEl.innerHTML = '<span style="color: var(--spectral-blue)">Buying tokens...</span>';

            try {
                const res = await fetch('/api/invoke-contract', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        address: contractAddress,
                        entry_point: 4,
                        calldata: caller + ',' + amount
                    })
                });
                const data = await res.json();
                if (res.ok && data.success) {
                    statusEl.innerHTML = `<span style="color: var(--accent-green)">Bought ${data.returnValue || amount} tokens!</span>`;
                    loadTokenData();
                } else {
                    statusEl.innerHTML = `<span style="color: var(--burn-red)">Buy failed: ${data.error || 'Unknown error'}</span>`;
                }
            } catch (err) {
                statusEl.innerHTML = `<span style="color: var(--burn-red)">Network error: ${err.message}</span>`;
            }
        }

        async function handleSell() {
            const caller = document.getElementById('caller-input').value || '1';
            const amount = parseInt(document.getElementById('sell-amount').value) || 0;
            const statusEl = document.getElementById('sell-status');
            statusEl.innerHTML = '<span style="color: var(--spectral-blue)">Selling tokens...</span>';

            try {
                const res = await fetch('/api/invoke-contract', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        address: contractAddress,
                        entry_point: 5,
                        calldata: caller + ',' + amount
                    })
                });
                const data = await res.json();
                if (res.ok && data.success) {
                    statusEl.innerHTML = `<span style="color: var(--accent-green)">Sold ${amount} tokens for ${data.returnValue || 0} ZYAN refund!</span>`;
                    loadTokenData();
                } else {
                    statusEl.innerHTML = `<span style="color: var(--burn-red)">Sell failed: ${data.error || 'Unknown error'}</span>`;
                }
            } catch (err) {
                statusEl.innerHTML = `<span style="color: var(--burn-red)">Network error: ${err.message}</span>`;
            }
        }

        window.onload = loadTokenData;
    </script>
</body>
</html>"#;

pub const AI_AGENTS_HTML: &str = r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Zyanya — Agent-Native Blockchain</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;600&display=swap" rel="stylesheet">
    <style>
        :root {
            --void: #0A0F1C;
            --shadow-teal: #0D3B50;
            --spectral-blue: #7EC8D3;
            --text: #E0E0E0;
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
            color: var(--text);
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
            max-width: 1000px;
            margin: 0 auto;
            padding: 0 20px;
        }

        header {
            display: flex;
            justify-content: center;
            align-items: center;
            padding: 1.5rem 0;
            border-bottom: 1px solid var(--shadow-teal);
            position: relative;
            width: 100%;
        }

        .menu-toggle { display: none; }

        .hamburger {
            display: none;
            font-size: 1.8rem;
            color: var(--spectral-blue);
            cursor: pointer;
            padding: 0.5rem 1rem;
            user-select: none;
            z-index: 101;
        }

        nav {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 1.5rem;
            flex-wrap: wrap;
        }

        nav a {
            color: var(--spectral-blue);
            text-decoration: none;
            font-weight: 600;
            font-size: 0.95rem;
            transition: color 0.3s ease;
            display: inline-flex;
            align-items: center;
        }

        nav a:hover, nav a.active {
            color: var(--text);
            text-shadow: 0 0 8px var(--spectral-blue);
        }

        .top-logo {
            text-align: center;
            margin: 2rem 0 1rem;
        }

        .top-logo svg {
            max-width: 520px;
            width: 100%;
            height: auto;
        }

        main {
            padding: 2rem 0 4rem;
        }

        section {
            margin-bottom: 4rem;
            text-align: center;
        }

        #hero h1 {
            font-size: 2.2rem;
            color: var(--spectral-blue);
            margin-bottom: 0.5rem;
            font-weight: 600;
        }

        .hero-subtitle {
            font-size: 1.2rem;
            color: var(--text);
            opacity: 0.9;
            margin-bottom: 2rem;
            text-align: center;
        }

        h2 {
            font-size: 1.6rem;
            margin-bottom: 1.5rem;
            text-transform: uppercase;
            letter-spacing: 2px;
            color: var(--text);
        }

        h4 {
            color: var(--spectral-blue);
            margin: 1.5rem 0 0.5rem;
            text-align: left;
            font-size: 1.05rem;
        }

        p {
            text-align: left;
            margin-bottom: 1rem;
            font-size: 1rem;
            color: var(--text);
        }

        .code-block {
            background: var(--void);
            border: 1px solid var(--shadow-teal);
            border-radius: 6px;
            padding: 1.2rem;
            text-align: left;
            overflow-x: auto;
            margin: 0.8rem 0 1.5rem;
            box-shadow: inset 0 0 10px rgba(0,0,0,0.5);
            background-image: linear-gradient(to right, rgba(126, 200, 211, 0.15), transparent 15px), linear-gradient(to left, rgba(126, 200, 211, 0.2), transparent 15px);
            background-position: left center, right center;
            background-repeat: no-repeat;
            background-size: 15px 100%;
        }

        code {
            font-family: var(--font-mono);
            font-size: 0.9rem;
            color: var(--spectral-blue);
            white-space: pre-wrap;
            word-break: break-word;
        }

        .callout {
            background: rgba(13, 59, 80, 0.4);
            border: 1px solid var(--shadow-teal);
            border-radius: 8px;
            padding: 1.5rem;
            margin: 1.5rem 0;
            text-align: left;
        }

        .callout strong {
            color: var(--spectral-blue);
        }

        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 1.5rem;
            text-align: left;
            margin-top: 1.5rem;
        }

        .card {
            background: var(--shadow-teal);
            padding: 1.8rem;
            border-radius: 8px;
            border: 1px solid rgba(126, 200, 211, 0.2);
            transition: transform 0.3s ease, box-shadow 0.3s ease;
            display: flex;
            flex-direction: column;
            justify-content: space-between;
        }

        .card:hover {
            transform: translateY(-5px);
            box-shadow: 0 10px 20px rgba(0,0,0,0.3);
        }

        .card h3 {
            color: var(--spectral-blue);
            margin-bottom: 0.8rem;
            font-size: 1.2rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }

        .card p {
            font-size: 0.95rem;
            opacity: 0.9;
            margin-bottom: 1.5rem;
            flex-grow: 1;
        }

        .card a.agent-link {
            display: inline-block;
            color: var(--spectral-blue);
            text-decoration: none;
            font-weight: 600;
            font-size: 0.9rem;
            border: 1px solid var(--spectral-blue);
            padding: 0.5rem 1rem;
            border-radius: 4px;
            text-align: center;
            transition: all 0.3s ease;
        }

        .card a.agent-link:hover {
            background: var(--spectral-blue);
            color: var(--void);
            box-shadow: 0 0 10px var(--spectral-blue);
        }

        .badge {
            display: inline-block;
            font-size: 0.7rem;
            padding: 0.2rem 0.5rem;
            border-radius: 3px;
            text-transform: uppercase;
            letter-spacing: 1px;
        }

        .badge-native {
            background: var(--spectral-blue);
            color: var(--void);
            font-weight: bold;
        }

        footer {
            text-align: center;
            padding: 2rem 0;
            border-top: 1px solid var(--shadow-teal);
            color: rgba(224, 224, 224, 0.5);
            font-size: 0.85rem;
            margin-top: 4rem;
        }

        @media (max-width: 768px) {
            header {
                flex-direction: column;
                padding: 1rem 0;
            }

            .hamburger {
                display: block;
            }

            nav {
                display: none;
                flex-direction: column;
                width: 100%;
                background: rgba(10, 15, 28, 0.98);
                border: 1px solid var(--shadow-teal);
                border-radius: 8px;
                margin-top: 0.5rem;
                padding: 0.5rem 0;
                gap: 0;
                z-index: 100;
            }

            .menu-toggle:checked ~ nav {
                display: flex;
            }

            nav a {
                display: flex;
                align-items: center;
                justify-content: center;
                min-height: 48px;
                width: 100%;
                margin: 0;
                padding: 0 1rem;
                border-bottom: 1px solid rgba(13, 59, 80, 0.5);
                font-size: 1rem;
            }

            nav a:last-child {
                border-bottom: none;
            }

            .container {
                padding: 0 16px;
            }

            #hero h1 {
                font-size: 1.4rem;
            }

            .hero-subtitle {
                font-size: 1rem;
            }

            h2 {
                font-size: 1.3rem;
            }

            .grid {
                grid-template-columns: 1fr;
                gap: 1rem;
            }

            .card {
                padding: 1.25rem;
            }

            .code-block {
                font-size: 0.8rem;
                padding: 1rem;
            }

            code {
                font-size: 0.8rem;
            }

            .card a.agent-link {
                min-height: 48px;
                display: flex;
                align-items: center;
                justify-content: center;
            }
        }

        @media (max-width: 480px) {
            nav a {
                min-height: 56px;
            }

            #hero h1 {
                font-size: 1.2rem;
            }

            .code-block {
                font-size: 0.75rem;
            }

            code {
                font-size: 0.75rem;
            }
        }
    </style>
</head>
<body>
    <div class="grid-bg"></div>
    <div class="container">
        <header>
            <input type="checkbox" id="menu-toggle" class="menu-toggle" aria-label="Toggle navigation">
            <label for="menu-toggle" class="hamburger" aria-label="Open menu">&#9776;</label>
            <nav>
                <a href="/">Home</a>
                <a href="/explorer">Explorer</a>
                <a href="/testnet">Testnet</a>
                <a href="/launch">Launch</a>
                <a href="/future">Roadmap</a>
                <a href="/agents" class="active">Agents</a>
                <a href="/docs">Docs</a>
                <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" style="color: var(--spectral-blue);">GitHub</a>
            </nav>
        </header>

        <div class="top-logo">
            <div id="logo-container"><!-- SVG injected dynamically --></div>
        </div>

        <main>
            <section id="hero">
                <h1>Zyanya — Agent-Native Blockchain</h1>
                <p class="hero-subtitle">Built for AI agents, not just humans</p>
            </section>

            <section id="why">
                <h2>Why Agent-Native?</h2>
                <p>Zyanya ships a Web MCP (Model Context Protocol) alongside the block explorer, so AI agents can read chain state, deploy tokens, buy/sell on the bonding curve, and query the network programmatically. No screen scraping — the blockchain speaks directly to agents via structured APIs.</p>
                <div class="callout">
                    <strong>Zero Friction for Agents:</strong> Rather than relying on DOM scraping or complex browser automation, autonomous agents interact with Zyanya using standardized Model Context Protocol tools and clean REST endpoints over IPv6.
                </div>
            </section>

            <section id="webmcp">
                <h2>The Web MCP</h2>
                <p>The WebMCP is a JavaScript polyfill that registers blockchain tools on <code>navigator.modelContext</code> (get-chain-info, get-block, deploy-token, invoke-contract, call-contract, ipv6-safety, etc.). Agents visiting the explorer page auto-discover these tools.</p>
                <p>For headless agents, the same functionality is available via the HTTP API endpoints.</p>
            </section>

            <section id="prompts">
                <h2>Prompt Examples</h2>
                <p>Below are sample commands and prompts an AI agent can execute to interact with the Zyanya blockchain:</p>

                <h4>1. Query Chain Information</h4>
                <div class="code-block"><code>Query the chain: curl https://testnet.zyanya.scottcloudhawk.org/api/info</code></div>

                <h4>2. Get Recent Blocks</h4>
                <div class="code-block"><code>Get recent blocks: curl https://testnet.zyanya.scottcloudhawk.org/api/blocks</code></div>

                <h4>3. Deploy a Token</h4>
                <div class="code-block"><code>Deploy a token: POST /api/deploy-token with {name, symbol, supply, slope, description, twitter, website}</code></div>

                <h4>4. Buy Tokens on Bonding Curve</h4>
                <div class="code-block"><code>Buy tokens on the bonding curve: POST /api/invoke-contract with {contract_address, entry_point: 4, calldata: 'caller,tokens_to_mint'}</code></div>

                <h4>5. Check Token Spot Price</h4>
                <div class="code-block"><code>Check the price: POST /api/call-contract with {contract_address, entry_point: 6, calldata: ''}</code></div>

                <h4>6. Check Token Metadata</h4>
                <div class="code-block"><code>Check token metadata: GET /api/token/:address/metadata</code></div>
            </section>

            <section id="recommended-agents">
                <h2>Recommended AI Agents</h2>
                <p style="text-align:center; max-width: 720px; margin: 0 auto 2rem;">Frameworks and autonomous agents built for coding, CAD generation, automated workflows, and blockchain execution on Zyanya.</p>
                
                <div class="grid">
                    <div class="card">
                        <div>
                            <h3>Pi Agent <span class="badge badge-native">Core</span></h3>
                            <p>A clean, focused coding + orchestration agent (pi-coding-agent). Runs on cloud LLMs (glm-5.2:cloud) for main reasoning + local Ollama models for sub-work. The agent that built Zyanya.</p>
                        </div>
                        <a href="https://github.com/node-tech/pi" target="_blank" class="agent-link">View Pi Agent &rarr;</a>
                    </div>

                    <div class="card">
                        <div>
                            <h3>OpenCode <span class="badge badge-native">Terminal</span></h3>
                            <p>A terminal-based coding agent with local LLM support. Great for 3D printing CAD generation + code tasks. Runs on Ollama cloud models.</p>
                        </div>
                        <a href="https://github.com/opencode-ai/opencode" target="_blank" class="agent-link">View OpenCode &rarr;</a>
                    </div>

                    <div class="card">
                        <div>
                            <h3>Hermes Agent <span class="badge badge-native">Harness</span></h3>
                            <p>A full agent harness with a gateway, cron jobs, MCP servers, and Discord integration. The original architect of the Zyanya trading system. Runs at ~/.hermes.</p>
                        </div>
                        <a href="https://github.com/hermes-agent/hermes" target="_blank" class="agent-link">View Hermes Agent &rarr;</a>
                    </div>

                    <div class="card">
                        <div>
                            <h3>Spacebot <span class="badge badge-native">Agent OS</span></h3>
                            <p>An AI Agent OS for 3D printing. A Rust-based agentic system that generates CAD models from text prompts.</p>
                        </div>
                        <a href="https://spacebot.sh" target="_blank" class="agent-link">Visit Spacebot.sh &rarr;</a>
                    </div>

                    <div class="card">
                        <div>
                            <h3>DeerFlow <span class="badge badge-native">Workflow</span></h3>
                            <p>A research + workflow agent by ByteDance. Deep research, multi-step reasoning, and workflow automation. Great for analyzing blockchain data + generating insights.</p>
                        </div>
                        <a href="https://github.com/bytedance/deerflow" target="_blank" class="agent-link">View DeerFlow &rarr;</a>
                    </div>
                </div>
            </section>
        </main>

        <footer>
            <p>The ghost in the IPv6 machine. Forever, always.</p>
            <p>&copy; 2026 Zyanya Project. All rights reserved. &bull; <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" style="color: var(--spectral-blue); text-decoration: none;">Source on GitHub</a></p>
        </footer>
    </div>

    <script>
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
        });
    </script>
    <script src="/webmcp.js"></script>
</body>
</html>
"###;

pub const DOCS_HTML: &str = r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Zyanya — Technical White Paper</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;600&display=swap" rel="stylesheet">
    <style>
        :root {
            --void: #0A0F1C;
            --shadow-teal: #0D3B50;
            --spectral-blue: #7EC8D3;
            --text: #E0E0E0;
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
            color: var(--text);
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
            max-width: 950px;
            margin: 0 auto;
            padding: 0 20px;
        }

        header {
            display: flex;
            justify-content: center;
            align-items: center;
            padding: 1.5rem 0;
            border-bottom: 1px solid var(--shadow-teal);
            position: relative;
            width: 100%;
        }

        .menu-toggle { display: none; }

        .hamburger {
            display: none;
            font-size: 1.8rem;
            color: var(--spectral-blue);
            cursor: pointer;
            padding: 0.5rem 1rem;
            user-select: none;
            z-index: 101;
        }

        nav {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 1.5rem;
            flex-wrap: wrap;
        }

        nav a {
            color: var(--spectral-blue);
            text-decoration: none;
            font-weight: 600;
            font-size: 0.95rem;
            transition: color 0.3s ease;
            display: inline-flex;
            align-items: center;
        }

        nav a:hover, nav a.active {
            color: var(--text);
            text-shadow: 0 0 8px var(--spectral-blue);
        }

        .top-logo {
            text-align: center;
            margin: 2rem 0 1rem;
        }

        .top-logo svg {
            max-width: 520px;
            width: 100%;
            height: auto;
        }

        main {
            padding: 2rem 0 4rem;
        }

        section {
            margin-bottom: 4rem;
            text-align: left;
        }

        #hero {
            text-align: center;
        }

        #hero h1 {
            font-size: 2.2rem;
            color: var(--spectral-blue);
            margin-bottom: 0.5rem;
            font-weight: 600;
        }

        .hero-subtitle {
            font-size: 1.2rem;
            color: var(--text);
            opacity: 0.9;
            margin-bottom: 2.5rem;
            text-align: center;
        }

        h2 {
            font-size: 1.5rem;
            margin-bottom: 1.2rem;
            text-transform: uppercase;
            letter-spacing: 2px;
            color: var(--spectral-blue);
            border-bottom: 1px solid var(--shadow-teal);
            padding-bottom: 0.5rem;
        }

        h3 {
            font-size: 1.15rem;
            color: var(--text);
            margin: 1.2rem 0 0.6rem;
        }

        p {
            margin-bottom: 1rem;
            font-size: 0.98rem;
            color: var(--text);
        }

        ul, ol {
            margin-left: 1.5rem;
            margin-bottom: 1rem;
        }

        li {
            margin-bottom: 0.5rem;
        }

        .code-block {
            background: var(--void);
            border: 1px solid var(--shadow-teal);
            border-radius: 6px;
            padding: 1.2rem;
            overflow-x: auto;
            margin: 1rem 0;
            box-shadow: inset 0 0 10px rgba(0,0,0,0.5);
            background-image: linear-gradient(to right, rgba(126, 200, 211, 0.15), transparent 15px), linear-gradient(to left, rgba(126, 200, 211, 0.2), transparent 15px);
            background-position: left center, right center;
            background-repeat: no-repeat;
            background-size: 15px 100%;
        }

        code {
            font-family: var(--font-mono);
            font-size: 0.88rem;
            color: var(--spectral-blue);
            white-space: pre-wrap;
            word-break: break-word;
        }

        .callout {
            background: rgba(13, 59, 80, 0.4);
            border: 1px solid var(--shadow-teal);
            border-radius: 8px;
            padding: 1.2rem 1.5rem;
            margin: 1.5rem 0;
        }

        .callout strong {
            color: var(--spectral-blue);
        }

        .formula-box {
            background: rgba(10, 15, 28, 0.9);
            border: 1px solid var(--spectral-blue);
            border-radius: 6px;
            padding: 1.2rem 1.5rem;
            margin: 1.2rem 0;
            text-align: center;
            font-size: 1.05rem;
            color: var(--spectral-blue);
            font-weight: 600;
        }

        footer {
            text-align: center;
            padding: 2rem 0;
            border-top: 1px solid var(--shadow-teal);
            color: rgba(224, 224, 224, 0.5);
            font-size: 0.85rem;
            margin-top: 4rem;
        }

        @media (max-width: 768px) {
            header {
                flex-direction: column;
                padding: 1rem 0;
            }

            .hamburger {
                display: block;
            }

            nav {
                display: none;
                flex-direction: column;
                width: 100%;
                background: rgba(10, 15, 28, 0.98);
                border: 1px solid var(--shadow-teal);
                border-radius: 8px;
                margin-top: 0.5rem;
                padding: 0.5rem 0;
                gap: 0;
                z-index: 100;
            }

            .menu-toggle:checked ~ nav {
                display: flex;
            }

            nav a {
                display: flex;
                align-items: center;
                justify-content: center;
                min-height: 48px;
                width: 100%;
                margin: 0;
                padding: 0 1rem;
                border-bottom: 1px solid rgba(13, 59, 80, 0.5);
                font-size: 1rem;
            }

            nav a:last-child {
                border-bottom: none;
            }

            .container {
                padding: 0 16px;
            }

            #hero h1 {
                font-size: 1.4rem;
            }

            .hero-subtitle {
                font-size: 1rem;
            }

            h2 {
                font-size: 1.3rem;
            }

            h3 {
                font-size: 1.05rem;
            }

            .code-block {
                font-size: 0.8rem;
                padding: 1rem;
            }

            code {
                font-size: 0.8rem;
            }

            .formula-box {
                font-size: 0.9rem;
                padding: 0.8rem 1rem;
                word-break: break-word;
            }

            .callout {
                padding: 1rem;
            }
        }

        @media (max-width: 480px) {
            nav a {
                min-height: 56px;
            }

            #hero h1 {
                font-size: 1.2rem;
            }

            .code-block {
                font-size: 0.75rem;
            }

            code {
                font-size: 0.75rem;
            }

            .formula-box {
                font-size: 0.8rem;
            }
        }
    </style>
</head>
<body>
    <div class="grid-bg"></div>
    <div class="container">
        <header>
            <input type="checkbox" id="menu-toggle" class="menu-toggle" aria-label="Toggle navigation">
            <label for="menu-toggle" class="hamburger" aria-label="Open menu">&#9776;</label>
            <nav>
                <a href="/">Home</a>
                <a href="/explorer">Explorer</a>
                <a href="/testnet">Testnet</a>
                <a href="/launch">Launch</a>
                <a href="/future">Roadmap</a>
                <a href="/agents">Agents</a>
                <a href="/docs" class="active">Docs</a>
                <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" style="color: var(--spectral-blue);">GitHub</a>
            </nav>
        </header>

        <div class="top-logo">
            <div id="logo-container"><!-- SVG injected dynamically --></div>
        </div>

        <main>
            <section id="hero">
                <h1>Zyanya — Technical White Paper</h1>
                <p class="hero-subtitle">The Ghost in the IPv6 Machine</p>
            </section>

            <section id="abstract">
                <h2>Abstract</h2>
                <p>Zyanya is an IPv6-native, agent-native, CPU-mineable blockDAG forked from Spectre (the Kaspa-family GhostDAG protocol). Designed as an autonomous decentralized computing substrate, Zyanya merges high-throughput DAG consensus with native Model Context Protocol (MCP) integrations, enabling AI agents to operate on-chain without human mediation or brittle web scraping.</p>
                <p>The network features a zero-premine fair launch, a lightweight stack-based smart contract Virtual Machine (zyanya-vm), an on-chain pump.fun-style bonding-curve token launcher, an automated market maker DEX, a native Web MCP interface, and a BIP-39 wallet ecosystem. Zyanya runs entirely over IPv6 — no IPv4 gateways, no NAT traversal, pure end-to-end peer-to-peer networking.</p>
            </section>

            <section id="sec-1">
                <h2>1. Architecture</h2>
                <p>Zyanya's underlying consensus engine is built upon the Spectre / GhostDAG blockDAG protocol. Unlike traditional single-chain architectures that discard parallel blocks as orphans, GhostDAG orders concurrently mined blocks into a directed acyclic graph, maximizing throughput while maintaining strict Proof-of-Work security guarantees.</p>
                <p>The core node daemon (<code>zyanyad</code>) coordinates DAG consensus, UTXO set validation, and smart contract state transitions. Networking is native IPv6 end-to-end: nodes discover peers, propagate blocks, and transmit transactions across explicit IPv6 sockets (defaulting to <code>[::]:18610</code>). Mining consensus utilizes SpectreX / AstroBWTv3 Proof-of-Work, specifically engineered for CPU accessibility and ASIC resistance.</p>
            </section>

            <section id="sec-2">
                <h2>2. Tokenomics</h2>
                <p>Zyanya prioritizes absolute fairness and sustainability in its economic design:</p>
                <ul>
                    <li><strong>Zero Premine:</strong> The genesis block contains zero minted tokens or pre-allocated outputs. 100% of all circulating supply originates from CPU mining rewards.</li>
                    <li><strong>Block Reward & Supply:</strong> Initial block reward is 50 ZYAN per block, following a smooth geometric decay toward a theoretical maximum supply of ~28.7 billion ZYAN.</li>
                    <li><strong>Deflationary Gas Burn:</strong> 50% of all gas fees collected during smart contract execution and token operations are permanently burned from total supply.</li>
                    <li><strong>Coinbase Maturity:</strong> Mined coinbase outputs require 100 block confirmations before becoming spendable, preventing short-reorg exploitation.</li>
                    <li><strong>No Allocations:</strong> Zero team allocations, zero foundation reserves, and zero VC/investor pre-allocations.</li>
                </ul>
            </section>

            <section id="sec-3">
                <h2>3. Smart Contracts</h2>
                <p>Smart contract execution on Zyanya is powered by <code>zyanya-vm</code>, a deterministic stack-based virtual machine operating on 64-bit unsigned integer registers and persistent contract key-value storage.</p>
                <p>Contracts are written in Zyanya Contract Language (ZCL) and compiled into compact bytecodes. Opcodes include stack manipulations (<code>PUSH</code>, <code>POP</code>, <code>DUP</code>), state storage (<code>SLOAD</code>, <code>SSTORE</code>), arithmetic (<code>ADD</code>, <code>SUB</code>, <code>MUL</code>, <code>DIV</code>), control flow (<code>JUMPIF</code>, <code>CALL</code>), and execution control (<code>REVERT</code>, <code>RETURN</code>).</p>

                <div class="callout">
                    <strong>Checked Arithmetic & Security:</strong> All mathematical operations within <code>zyanya-vm</code> execute with mandatory overflow and underflow checking. Any arithmetic boundary violation automatically triggers an execution revert, preserving state integrity. Gas metering enforces strict limits on execution cycles to prevent infinite loops.
                </div>
            </section>

            <section id="sec-4">
                <h2>4. Bonding Curve Token Launcher</h2>
                <p>Zyanya introduces a pump.fun-style bonding curve token launch framework directly within the contract runtime. The system employs a linear bonding curve where token price scales linearly with total circulating supply:</p>
                
                <div class="formula-box">Price = slope &times; supply</div>

                <p>Users and AI agents create new tokens via the <code>/launch</code> interface by providing parameters (Name, Symbol, Initial Reserve Supply, Slope, Description, Icon, Social links). Token purchasing and selling occur against the bonding curve on the <code>/token</code> page.</p>
                <p>Because the VM operates on 64-bit integer registers, string metadata (name, description, social links, icon path) is indexed by an off-chain metadata store, while economic state resides on-chain.</p>

                <div class="callout">
                    <strong>Mathematical Formulas:</strong>
                    <div class="formula-box">Buy Cost = [ slope &times; (2 &times; S &times; k + k&sup2;) ] / 2</div>
                    <div class="formula-box">Sell Refund = [ slope &times; (2 &times; S &times; k - k&sup2;) ] / 2</div>
                    <p style="margin-top:0.5rem; text-align:center;"><em>where <strong>S</strong> is current circulating token supply, and <strong>k</strong> is the quantity of tokens being bought or sold.</em></p>
                </div>
            </section>

            <section id="sec-5">
                <h2>5. The DEX</h2>
                <p>Zyanya features an automated constant-product market maker (AMM) DEX compiled via <code>dex.zcl</code>. The exchange maintains the constant-product invariant:</p>

                <div class="formula-box">x &times; y = k</div>

                <p>Every trade incurs a 0.3% swap fee distributed to liquidity providers. The DEX exposes entry points for <code>addLiquidity</code>, <code>removeLiquidity</code>, and <code>swap</code>, issuing LP tokens to represent liquidity shares.</p>
            </section>

            <section id="sec-6">
                <h2>6. The Web MCP</h2>
                <p>Zyanya is the first blockchain natively equipped with Model Context Protocol (MCP) support for AI agents. The Web MCP polyfill registers structured tools on <code>navigator.modelContext</code> when an agent visits the explorer web UI.</p>
                <p>Registered tools include:</p>
                <ul>
                    <li><code>get-chain-info</code>: Retrieve height, block count, DAA score, and peer status.</li>
                    <li><code>get-block</code>: Fetch specific block DAG header and transaction details.</li>
                    <li><code>deploy-token</code>: Deploy a new bonding curve token contract.</li>
                    <li><code>invoke-contract</code>: Execute state-changing smart contract entry points.</li>
                    <li><code>call-contract</code>: Perform read-only contract state queries.</li>
                    <li><code>ipv6-safety</code>: Access network hardening rules and IPv6 safety guidance.</li>
                </ul>
                <p>Headless agents can access identical functionalities via native HTTP API endpoints over IPv6.</p>
            </section>

            <section id="sec-7">
                <h2>7. The Wallet</h2>
                <p>The official <code>zyanya-wallet</code> CLI uses BIP-39 24-word seed mnemonics with optional 25th-word passphrase protection for key derivation. Key features include:</p>
                <ul>
                    <li>Interactive TUI for balance checks, ZYAN transfers, token operations, and DEX swaps.</li>
                    <li><code>--show-secret</code> flag requirement to reveal private keys, protecting against unintended output logging.</li>
                    <li><code>--force</code> guard when overwriting keyfiles, preventing accidental key loss.</li>
                    <li>Strict Linux <code>0600</code> file permission enforcement on stored keyfiles.</li>
                    <li>Fixed-point amount parsing to avoid precision errors in financial calculations.</li>
                </ul>
            </section>

            <section id="sec-8">
                <h2>8. Mining</h2>
                <p>Mining on Zyanya is optimized for standard consumer CPUs. Miners can choose between solo mining via <code>zyanya-miner</code> (supporting adjustable CPU thread limits, e.g. 25% CPU throttle) or connecting to Stratum pools.</p>
                <p>The official <code>zyanya-pool</code> daemon listens on IPv6 socket <code>[::]:3334</code> using the Stratum protocol. Mined coinbase outputs follow the 100-block maturity rule before becoming spendable.</p>
            </section>

            <section id="sec-9">
                <h2>9. The Network</h2>
                <p>Zyanya defines three network tiers: <strong>devnet</strong>, <strong>testnet</strong>, and <strong>mainnet</strong>.</p>
                <p>The public testnet consists of 3 primary seed nodes connected via IPv6 (Unraid seed, MS-A2 node, and crypto backup node). The block explorer and Web MCP server operate publicly at <code>testnet.zyanya.scottcloudhawk.org</code>.</p>
            </section>

            <section id="sec-10">
                <h2>10. IPv6-Native</h2>
                <p>Zyanya is designed exclusively for IPv6 networking. Eliminating IPv4 avoids NAT traversal friction, eliminates middlebox gateways, enables true peer-to-peer connectivity, and provides an practically infinite address space for autonomous nodes and agent micro-instances.</p>
                <p>The IPv6 safety documentation guides node operators on firewall configuration using <code>nftables</code> or <code>ufw</code>, default-deny inbound policies, and RFC 4890 ICMPv6 filtering standards to maintain node security while preserving P2P health.</p>
            </section>

            <section id="sec-11">
                <h2>11. Roadmap</h2>
                <ol>
                    <li><strong>Phase 01: Ghost in the Machine</strong> — Public testnet hardening, P2P stability tuning, and contract VM edge-case testing.</li>
                    <li><strong>Phase 02: Dark Launch</strong> — Mainnet genesis block mining and silent peer network deployment.</li>
                    <li><strong>Phase 03: Prepare Optics</strong> — Documentation finalization, Web MCP tool expansion, and explorer integrations.</li>
                    <li><strong>Phase 04: The r/IPv6 Signal</strong> — Public announcement and onboarding of the global IPv6 community.</li>
                </ol>
            </section>
        </main>

        <footer>
            <p>The ghost in the IPv6 machine. Forever, always.</p>
            <p>&copy; 2026 Zyanya Project. All rights reserved. &bull; <a href="https://github.com/scotthawk-maker/zyanya" target="_blank" style="color: var(--spectral-blue); text-decoration: none;">Source on GitHub</a></p>
        </footer>
    </div>

    <script>
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
        });
    </script>
    <script src="/webmcp.js"></script>
</body>
</html>
"###;


pub const LLMS_TXT: &str = r###"# Zyanya

> Zyanya is an IPv6-native, agent-native, CPU-mineable blockDAG — a Spectre/GhostDAG fork. The ghost in the IPv6 machine.

## Info
- URL: https://testnet.zyanya.scottcloudhawk.org/
- Description: A decentralized blockchain with zero-premine fair launch, a smart contract VM, a pump.fun-style bonding-curve token launcher, a DEX, and a Web MCP for AI agent interaction. Runs entirely over IPv6.

## Sections
- [Home](https://testnet.zyanya.scottcloudhawk.org/): Landing page with the three pillars (Ghost, Secret, Forever) + economics
- [Block Explorer](https://testnet.zyanya.scottcloudhawk.org/explorer): Live block explorer with auto-refreshing block table + stats
- [Token Launcher](https://testnet.zyanya.scottcloudhawk.org/launch): Create bonding-curve tokens with metadata, icon, and socials
- [Testnet Setup](https://testnet.zyanya.scottcloudhawk.org/testnet): All-in-one setup guide (download → node → wallet → mine)
- [Roadmap](https://testnet.zyanya.scottcloudhawk.org/future): Features (live + planned) + the 4-phase go-to-market
- [AI Agents](https://testnet.zyanya.scottcloudhawk.org/agents): Agent-native guide with prompt examples + recommended agents
- [Documentation](https://testnet.zyanya.scottcloudhawk.org/docs): Technical white paper
- [IPv6 Safety](https://testnet.zyanya.scottcloudhawk.org/#ipv6-safety): IPv6 rewards, risks, and hardening guide

## API Endpoints (for AI agents)
- GET /api/info: Chain state (block_count, difficulty, supply, peers)
- GET /api/blocks: Recent blocks (hash, blue_score, daa_score, timestamp, tx_count)
- GET /api/block/:hash: Block details by hash
- GET /api/contracts: Deployed contracts list
- GET /api/tokens: Deployed tokens list
- GET /api/token/:address/metadata: Token metadata (name, symbol, description, socials, icon_uri)
- GET /api/token-balance?token=:addr&holder=:id: Token balance for a holder
- GET /api/dag: DAG graph data
- POST /api/deploy-token: Deploy a bonding-curve token (name, symbol, supply, slope, description, twitter, telegram, website, icon_base64)
- POST /api/invoke-contract: State-changing contract call (contract_address, entry_point, calldata, gas)
- POST /api/call-contract: Read-only contract call (contract_address, entry_point, calldata, gas)
- POST /api/deploy-contract: Deploy a custom contract (bytecode)
- POST /api/swap-on-dex: Swap on the DEX
- GET /token-icons/:filename: Token icon images
- GET /webmcp.js: Web MCP polyfill (registers blockchain tools on navigator.modelContext)

## WebMCP Tools (for browser-based agents)
- get-chain-info: Query block count, DAA score, difficulty, supply, peers
- get-block: Query block details by hash
- ipv6-safety: IPv6 hardening guidance for agents
- (more tools registered dynamically by the explorer)

## Networks
- Testnet (public): Seed at [2606:8ac0:2615:79aa:1a66:daff:fe99:31f7]:18211
- RPC: [2606:8ac0:2615:79aa:1a66:daff:fe99:31f7]:18210
- Explorer: https://testnet.zyanya.scottcloudhawk.org/
- GitHub: https://github.com/scotthawk-maker/zyanya

## Tokenomics
- Max supply: ~28.7 billion ZYAN (smooth geometric decay)
- Block reward: 50 ZYAN/block
- Premine: ZERO (genesis has zero outputs)
- Gas burn: 50% of every transaction fee permanently burned
- Coinbase maturity: 100 blocks
- No team wallets, no foundation allocation, no investor stake

## Smart Contracts
- VM: zyanya-vm (stack-based, u64-only, checked arithmetic)
- Language: ZCL (Zyanya Contract Language)
- Bonding curve: price = slope * supply, buy cost = slope*(2*S*k + k^2)/2, sell refund = slope*(2*S*k - k^2)/2
- Entry points: 0=init, 1=transfer, 2=balance_of, 3=total_supply, 4=buy, 5=sell, 6=price
- DEX: Constant-product AMM (x*y=k, 0.3% fee)"###;

pub const LLMS_MD: &str = r###"# Zyanya — LLM/Agent API Documentation

> Zyanya is an IPv6-native, agent-native, CPU-mineable blockDAG. This document provides detailed API documentation for AI agents to interact with the Zyanya blockchain programmatically.

## Quick Start for Agents

```
# Query the chain state
curl https://testnet.zyanya.scottcloudhawk.org/api/info

# Get recent blocks
curl https://testnet.zyanya.scottcloudhawk.org/api/blocks

# Deploy a bonding-curve token
curl -X POST https://testnet.zyanya.scottcloudhawk.org/api/deploy-token \
  -H 'Content-Type: application/json' \
  -d '{"name":"MyToken","symbol":"MTK","supply":1000000,"slope":2,"owner":"100","description":"A test token","twitter":"@mytoken"}'

# Buy tokens on the bonding curve (entry_point 4 = buy)
curl -X POST https://testnet.zyanya.scottcloudhawk.org/api/invoke-contract \
  -H 'Content-Type: application/json' \
  -d '{"contract_address":"<TOKEN_ADDRESS>","entry_point":4,"calldata":"100,10"}'

# Check the price (entry_point 6 = price)
curl -X POST https://testnet.zyanya.scottcloudhawk.org/api/call-contract \
  -H 'Content-Type: application/json' \
  -d '{"contract_address":"<TOKEN_ADDRESS>","entry_point":6,"calldata":""}'

# Sell tokens (entry_point 5 = sell)
curl -X POST https://testnet.zyanya.scottcloudhawk.org/api/invoke-contract \
  -H 'Content-Type: application/json' \
  -d '{"contract_address":"<TOKEN_ADDRESS>","entry_point":5,"calldata":"100,5"}'

# Get token metadata
curl https://testnet.zyanya.scottcloudhawk.org/api/token/<TOKEN_ADDRESS>/metadata
```

## API Reference

### GET /api/info
Returns the current chain state.
- Response: `{block_count, header_count, difficulty, network, is_synced, server_version, virtual_daa_score, past_median_time, sink_hash, peer_count, mempool_size, coin_supply_zyan, max_supply_zyan}`

### GET /api/blocks
Returns the 20 most recent blocks.
- Response: `[{hash, blue_score, daa_score, timestamp, tx_count, selected_parent}]`

### GET /api/block/:hash
Returns details for a specific block by its 64-character hex hash.

### GET /api/contracts
Returns a list of all deployed contracts.

### GET /api/tokens
Returns a list of all deployed tokens with their metadata (name, symbol, total_supply, owner_address).

### GET /api/token/:address/metadata
Returns the off-chain metadata for a token: `{name, symbol, description, twitter, telegram, website, icon_uri}`

### GET /api/token-balance
Query parameters: `token=<contract_address>&holder=<holder_id>`
Returns: `{balance, holder, token}`

### POST /api/deploy-token
Deploy a bonding-curve token with metadata.
- Body: `{name, symbol, supply, slope, owner, description, twitter, telegram, website, icon_base64}`
- Response: `{contract_address, name, symbol, description, socials, icon_uri, slope, supply, gasUsed}`
- Note: State-changing endpoints require ZYANYA_EXPLORER_ENABLE_WRITE=1 on the server.

### POST /api/invoke-contract
Execute a state-changing contract call (buy, sell, transfer, init).
- Body: `{contract_address, entry_point, calldata, gas}`
- calldata format: comma-separated u64 values (e.g., "100,10" for caller=100, amount=10)
- Entry points for bonding-curve tokens: 0=init(slope), 1=transfer(from,to,amt), 4=buy(caller,k), 5=sell(caller,k)
- Response: `{success, returnValue, gasUsed, transactionId}`

### POST /api/call-contract
Execute a read-only contract call (balance_of, total_supply, price).
- Body: `{contract_address, entry_point, calldata, gas}`
- Entry points for bonding-curve tokens: 2=balance_of(addr), 3=total_supply(), 6=price()
- Response: `{executionSuccess, gasUsed, returnValue}`

### GET /token-icons/:filename
Returns a token icon PNG image.

## Bonding Curve Math
- price(supply) = slope * supply
- buy cost = slope * (2 * S * k + k^2) / 2  (where S = current supply, k = tokens to mint)
- sell refund = slope * (2 * S * k - k^2) / 2  (where S = supply before burn, k = tokens to burn)
- All arithmetic is checked (overflow/underflow reverts the transaction)

## Web MCP
The explorer serves `/webmcp.js` which registers blockchain tools on `navigator.modelContext`:
- `get-chain-info` — query chain state
- `get-block` — query block details
- `get-recent-blocks` — list recent blocks
- `ipv6-safety` — IPv6 hardening guidance

Browser-based agents auto-discover these tools when visiting the explorer page.

## Network
- Testnet seed (P2P): `[2606:8ac0:2615:79aa:1a66:daff:fe99:31f7]:18211`
- Testnet RPC: `[2606:8ac0:2615:79aa:1a66:daff:fe99:31f7]:18210`
- Explorer: `https://testnet.zyanya.scottcloudhawk.org/`
- GitHub: `https://github.com/scotthawk-maker/zyanya`
- All connections are IPv6-only."###;
