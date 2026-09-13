
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
                "Do NOT block all ICMPv6 ,  IPv6 needs it for Neighbor Discovery and Path MTU Discovery; blocking it breaks connectivity (RFC 4890).",
                "Use a stable/assigned IPv6 address for a node, or a privacy/temporary address if you prefer."
            ],
            links: {
                linux: [
                    { name: "Arch Wiki ,  IPv6", url: "https://wiki.archlinux.org/title/IPv6" },
                    { name: "nftables", url: "https://wiki.archlinux.org/title/Nftables" },
                    { name: "ufw", url: "https://wiki.archlinux.org/title/Uncomplicated_Firewall" },
                    { name: "RFC 4890 ,  ICMPv6 filtering", url: "https://datatracker.ietf.org/doc/html/rfc4890" }
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
                contract_address: params.contractAddress,
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
                contract_address: params.contractAddress,
                calldata: params.calldata || "",
                entry_point: params.entryPoint || 0,
                gas: params.gas || 100000
            })
        })
    });

    mc.registerTool({
        name: "deploy-token",
        description: "Deploy a custom reference ERC-20 style token contract with specified supply, owner key, symbol, slope, and metadata.",
        inputSchema: {
            type: "object",
            properties: {
                name: { type: "string", description: "Token name" },
                symbol: { type: "string", description: "Token ticker symbol (e.g. ZYAN)" },
                supply: { type: "number", description: "Initial total supply" },
                owner: { type: "string", description: "Owner address or key ID (default 1)" },
                slope: { type: "number", description: "Bonding curve slope parameter (default 1)" },
                description: { type: "string", description: "Token description" },
                twitter: { type: "string", description: "Twitter handle or URL" },
                telegram: { type: "string", description: "Telegram group or handle" },
                website: { type: "string", description: "Project website URL" },
                icon_base64: { type: "string", description: "Base64-encoded token icon image" }
            },
            required: ["supply"]
        },
        execute: async (params) => await apiFetch('/api/deploy-token', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                name: params.name || "Token",
                symbol: params.symbol || "TKN",
                supply: params.supply,
                owner: params.owner || "1",
                slope: params.slope || 1,
                description: params.description || "",
                twitter: params.twitter || "",
                telegram: params.telegram || "",
                website: params.website || "",
                icon_base64: params.icon_base64 || null
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

    mc.registerTool({
        name: "get-staking-info",
        description: "Query Zyanya Staking Vault metrics: total staked ZYAN, accumulated 0.3% protocol fee rewards distributed, base APR (18.4%), and 90-day covenant boosted APR (46.0%). Optionally query user position.",
        inputSchema: {
            type: "object",
            properties: {
                userAddress: { type: "string", description: "Optional staker address (zyanya:...)" }
            }
        },
        execute: async (params) => {
            const url = params && params.userAddress ? '/api/staking-info?user=' + encodeURIComponent(params.userAddress) : '/api/staking-info';
            return await apiFetch(url);
        }
    });

    mc.registerTool({
        name: "stake-zyan",
        description: "Lock ZYAN in the Subnetwork 3 Staking Vault with optional covenant timelock (0 for Flexible 1.0x, 30 for 1.5x, 90 for 2.5x APR boost).",
        inputSchema: {
            type: "object",
            properties: {
                user: { type: "string", description: "Staker address or identifier" },
                amount: { type: "number", description: "Amount of ZYAN to stake" },
                covenantDays: { type: "number", description: "Covenant lock duration in days (0, 30, or 90)" }
            },
            required: ["user", "amount"]
        },
        execute: async (params) => await apiFetch('/api/stake', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                user: params.user,
                amount: params.amount,
                covenant_days: params.covenantDays || 0
            })
        })
    });

    mc.registerTool({
        name: "unstake-zyan",
        description: "Unstake ZYAN from the Subnetwork 3 Staking Vault if the covenant timelock period has elapsed.",
        inputSchema: {
            type: "object",
            properties: {
                user: { type: "string", description: "Staker address or identifier" },
                amount: { type: "number", description: "Amount of ZYAN to unstake" }
            },
            required: ["user", "amount"]
        },
        execute: async (params) => await apiFetch('/api/unstake', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                user: params.user,
                amount: params.amount
            })
        })
    });

    mc.registerTool({
        name: "claim-staking-dividends",
        description: "Claim accumulated 0.3% protocol fee rewards/dividends from AMM swaps and token launches.",
        inputSchema: {
            type: "object",
            properties: {
                user: { type: "string", description: "Staker address or identifier" }
            },
            required: ["user"]
        },
        execute: async (params) => await apiFetch('/api/claim-rewards', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ user: params.user })
        })
    });

    mc.registerTool({
        name: "get-dex-pools",
        description: "Retrieve all active AMM constant-product liquidity pools, reserves, 24h volume, and real-time prices.",
        inputSchema: { type: "object", properties: {} },
        execute: async () => await apiFetch('/api/dex')
    });

    mc.registerTool({
        name: "add-dex-liquidity",
        description: "Add liquidity to a Zyanya AMM liquidity pool to earn LP shares and trading fee yield.",
        inputSchema: {
            type: "object",
            properties: {
                dexAddress: { type: "string", description: "DEX contract address" },
                amountA: { type: "number", description: "Amount of ZYAN in sompi" },
                amountB: { type: "number", description: "Amount of Token B in units" }
            },
            required: ["dexAddress", "amountA", "amountB"]
        },
        execute: async (params) => await apiFetch('/api/add-liquidity', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                dex: params.dexAddress,
                amount_a: params.amountA,
                amount_b: params.amountB
            })
        })
    });

    console.log("[WebMCP] Zyanya Web MCP Tool Suite initialized. Total tools:", mc.getTools().length);
})();
