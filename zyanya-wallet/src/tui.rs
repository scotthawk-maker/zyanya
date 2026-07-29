use std::io::{self, Write};
use std::str::FromStr;
use zyanya_consensus_core::constants::SOMPI_PER_ZYANYA;
use zyanya_rpc_core::api::rpc::RpcApi;
use crate::key_management::WalletKeypair;
use crate::wallet_ops::WalletOps;

pub struct WalletTui {
    ops: WalletOps,
    ghost_token_contract: Option<String>,
    dex_contract: Option<String>,
}

impl WalletTui {
    pub fn new(ops: WalletOps) -> Self {
        Self {
            ops,
            ghost_token_contract: None,
            dex_contract: None,
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.render_header().await;
            self.print_menu();

            print!("\nSelect an option [1-9, q]: ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }

            let choice = input.trim();
            match choice {
                "1" => self.view_balances().await,
                "2" => self.view_send_zyan().await,
                "3" => self.view_tokens().await,
                "4" => self.view_send_tokens().await,
                "5" => self.view_swap_dex().await,
                "6" => self.view_history().await,
                "7" => self.view_new_address().await,
                "8" => self.view_load_address().await,
                "9" => self.view_setup_demo().await,
                "q" | "Q" | "exit" => {
                    println!("\nExiting Zyanya Wallet. Goodbye!");
                    break;
                }
                _ => {
                    println!("Invalid option. Press Enter to continue...");
                    self.wait_keypress();
                }
            }
        }
    }

    async fn render_header(&self) {
        print!("\x1B[2J\x1B[1;1H"); // Clear screen
        println!("================================================================================");
        println!("                         ZYANYA TUI WALLET v0.3.17                             ");
        println!("================================================================================");
        println!(" Address: {}", self.ops.keypair.address);
        println!(" PubKey:  {}", self.ops.keypair.xonly_pubkey);
        println!(" RPC URL: {}", self.ops.rpc_url);

        if let Ok(client) = self.ops.connect_rpc().await {
            if let Ok((balance, _)) = self.ops.get_zyan_balance(&client).await {
                let zyan_val = balance as f64 / SOMPI_PER_ZYANYA as f64;
                println!(" Balance: {} ZYAN ({} sompi)", zyan_val, balance);
            } else {
                println!(" Balance: [Error fetching balance]");
            }
            let _ = client.disconnect().await;
        } else {
            println!(" Balance: [RPC Offline]");
        }
        println!("================================================================================");
    }

    fn print_menu(&self) {
        println!("\n  [1] Show Balances (ZYAN & Tokens)");
        println!("  [2] Send ZYAN");
        println!("  [3] Token List & Balances");
        println!("  [4] Send Tokens");
        println!("  [5] Swap on DEX");
        println!("  [6] Transaction History");
        println!("  [7] Generate New Address / Keypair");
        println!("  [8] Load Keypair from File");
        println!("  [9] Setup / Deploy GHOST Token & DEX");
        println!("  [q] Quit");
    }

    fn wait_keypress(&self) {
        let mut buf = String::new();
        let _ = io::stdin().read_line(&mut buf);
    }

    fn read_input(prompt: &str) -> String {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }

    async fn view_balances(&self) {
        println!("\n--- [1] Wallet Balances ---");
        let client = match self.ops.connect_rpc().await {
            Ok(c) => c,
            Err(e) => {
                println!("Error connecting to RPC: {}", e);
                self.wait_keypress();
                return;
            }
        };

        let (balance, utxos) = self.ops.get_zyan_balance(&client).await.unwrap_or((0, vec![]));
        println!("ZYAN Balance: {} ZYAN ({} sompi)", balance as f64 / SOMPI_PER_ZYANYA as f64, balance);
        println!("Available UTXOs count: {}", utxos.len());

        if let Some(ref ghost) = self.ghost_token_contract {
            let holder_u64 = WalletOps::holder_u64(&self.ops.keypair.address);
            let token_bal = self.ops.get_token_balance(&client, ghost, holder_u64).await.unwrap_or(0);
            println!("GHOST Token Balance (Contract: {}): {}", ghost, token_bal);
        } else {
            println!("GHOST Token: [Not configured - Option 9 to setup]");
        }

        let _ = client.disconnect().await;
        println!("\nPress Enter to return...");
        self.wait_keypress();
    }

    async fn view_send_zyan(&mut self) {
        println!("\n--- [2] Send ZYAN ---");
        let recipient = Self::read_input("Enter Recipient Address (e.g. zyanyadev:...): ");
        if recipient.is_empty() {
            println!("Cancelled.");
            self.wait_keypress();
            return;
        }

        let amount_str = Self::read_input("Enter Amount in ZYAN (e.g. 10.5): ");
        let zyan_val: f64 = match amount_str.parse() {
            Ok(v) => v,
            Err(_) => {
                println!("Invalid amount.");
                self.wait_keypress();
                return;
            }
        };

        let amount_sompi = (zyan_val * SOMPI_PER_ZYANYA as f64) as u64;
        println!("\nConfirm Send:");
        println!("  To:     {}", recipient);
        println!("  Amount: {} ZYAN ({} sompi)", zyan_val, amount_sompi);
        println!("  Signer: User Private Key ({})", self.ops.keypair.address);

        let confirm = Self::read_input("Proceed with signing & sending? (y/N): ");
        if confirm.to_lowercase() != "y" {
            println!("Transaction cancelled.");
            self.wait_keypress();
            return;
        }

        let client = match self.ops.connect_rpc().await {
            Ok(c) => c,
            Err(e) => {
                println!("RPC Error: {}", e);
                self.wait_keypress();
                return;
            }
        };

        println!("Signing transaction with user private key and submitting to node mempool...");
        match self.ops.send_zyan(&client, &recipient, amount_sompi).await {
            Ok(tx_id) => {
                println!("\nSUCCESS! Transaction submitted!");
                println!("Transaction ID: {}", tx_id);
            }
            Err(e) => {
                println!("\nTransaction Failed: {}", e);
            }
        }

        let _ = client.disconnect().await;
        println!("\nPress Enter to return...");
        self.wait_keypress();
    }

    async fn view_tokens(&self) {
        println!("\n--- [3] Token List & Balances ---");
        let client = match self.ops.connect_rpc().await {
            Ok(c) => c,
            Err(e) => {
                println!("RPC Error: {}", e);
                self.wait_keypress();
                return;
            }
        };

        if let Some(ref ghost) = self.ghost_token_contract {
            let holder = WalletOps::holder_u64(&self.ops.keypair.address);
            let bal = self.ops.get_token_balance(&client, ghost, holder).await.unwrap_or(0);
            println!("1. GHOST Token");
            println!("   Contract Address: {}", ghost);
            println!("   Your Key ID:     {}", holder);
            println!("   Your Balance:    {} GHOST", bal);
        } else {
            println!("No tokens tracked yet. Use Option [9] to deploy the GHOST reference token or add contract.");
        }

        let _ = client.disconnect().await;
        println!("\nPress Enter to return...");
        self.wait_keypress();
    }

    async fn view_send_tokens(&mut self) {
        println!("\n--- [4] Send Tokens ---");
        let default_contract = self.ghost_token_contract.clone().unwrap_or_default();
        let token_contract = Self::read_input(&format!("Enter Token Contract Address [default: {}]: ", default_contract));
        let token_contract = if token_contract.is_empty() { default_contract } else { token_contract };

        if token_contract.is_empty() {
            println!("Contract address required.");
            self.wait_keypress();
            return;
        }

        let from_holder = WalletOps::holder_u64(&self.ops.keypair.address);
        println!("Sender Holder Key: {}", from_holder);

        let to_str = Self::read_input("Enter Recipient Holder ID (e.g. 2 or address): ");
        let to_u64: u64 = if let Ok(val) = to_str.parse() {
            val
        } else if let Ok(addr) = zyanya_addresses::Address::try_from(to_str.as_str()) {
            WalletOps::holder_u64(&addr)
        } else {
            println!("Invalid recipient ID.");
            self.wait_keypress();
            return;
        };

        let amount_str = Self::read_input("Enter Token Amount to Transfer: ");
        let amount: u64 = match amount_str.parse() {
            Ok(a) => a,
            Err(_) => {
                println!("Invalid amount.");
                self.wait_keypress();
                return;
            }
        };

        println!("\nConfirm Token Transfer:");
        println!("  Token Contract: {}", token_contract);
        println!("  From Key:       {}", from_holder);
        println!("  To Key:         {}", to_u64);
        println!("  Amount:         {}", amount);

        let confirm = Self::read_input("Proceed with token transfer? (y/N): ");
        if confirm.to_lowercase() != "y" {
            println!("Cancelled.");
            self.wait_keypress();
            return;
        }

        let client = match self.ops.connect_rpc().await {
            Ok(c) => c,
            Err(e) => {
                println!("RPC Error: {}", e);
                self.wait_keypress();
                return;
            }
        };

        println!("Submitting InvokeContract for token transfer...");
        match self.ops.send_token(&client, &token_contract, from_holder, to_u64, amount).await {
            Ok(tx_id) => {
                println!("\nSUCCESS! Token transfer submitted!");
                println!("Transaction ID: {}", tx_id);
            }
            Err(e) => {
                println!("\nToken transfer failed: {}", e);
            }
        }

        let _ = client.disconnect().await;
        println!("\nPress Enter to return...");
        self.wait_keypress();
    }

    async fn view_swap_dex(&mut self) {
        println!("\n--- [5] Swap on DEX ---");
        let default_dex = self.dex_contract.clone().unwrap_or_default();
        let dex_contract = Self::read_input(&format!("Enter DEX Contract Address [default: {}]: ", default_dex));
        let dex_contract = if dex_contract.is_empty() { default_dex } else { dex_contract };

        if dex_contract.is_empty() {
            println!("DEX contract address required.");
            self.wait_keypress();
            return;
        }

        let client = match self.ops.connect_rpc().await {
            Ok(c) => c,
            Err(e) => {
                println!("RPC Error: {}", e);
                self.wait_keypress();
                return;
            }
        };

        if let Ok((res_a, res_b)) = self.ops.get_dex_reserves(&client, &dex_contract).await {
            println!("Current Pool Reserves:");
            println!("  Token A (ZYAN):  {}", res_a);
            println!("  Token B (GHOST): {}", res_b);
        }

        let token_in_str = Self::read_input("Enter Token In [0 = ZYAN / A, 1 = GHOST / B]: ");
        let token_in_val: u64 = match token_in_str.trim().to_lowercase().as_str() {
            "0" | "a" | "zyan" => 0,
            "1" | "b" | "ghost" => 1,
            _ => {
                println!("Invalid token selection.");
                let _ = client.disconnect().await;
                self.wait_keypress();
                return;
            }
        };

        let amount_str = Self::read_input("Enter Input Amount to Swap: ");
        let amount_in: u64 = match amount_str.parse() {
            Ok(a) => a,
            Err(_) => {
                println!("Invalid amount.");
                let _ = client.disconnect().await;
                self.wait_keypress();
                return;
            }
        };

        println!("\nConfirm DEX Swap:");
        println!("  DEX Contract: {}", dex_contract);
        println!("  Swap In:      {}", if token_in_val == 0 { "ZYAN (A)" } else { "GHOST (B)" });
        println!("  Amount In:    {}", amount_in);

        let confirm = Self::read_input("Proceed with swap? (y/N): ");
        if confirm.to_lowercase() != "y" {
            println!("Cancelled.");
            let _ = client.disconnect().await;
            self.wait_keypress();
            return;
        }

        println!("Submitting InvokeContract for DEX swap...");
        match self.ops.swap_on_dex(&client, &dex_contract, token_in_val, amount_in).await {
            Ok((tx_id, out_amount)) => {
                println!("\nSUCCESS! DEX Swap completed!");
                println!("Transaction ID: {}", tx_id);
                println!("Output Amount Received: {}", out_amount);
            }
            Err(e) => {
                println!("\nDEX Swap failed: {}", e);
            }
        }

        let _ = client.disconnect().await;
        println!("\nPress Enter to return...");
        self.wait_keypress();
    }

    async fn view_history(&self) {
        println!("\n--- [6] Transaction History ---");
        if self.ops.history.is_empty() {
            println!("No recent transactions found.");
        } else {
            for (idx, record) in self.ops.history.iter().enumerate() {
                println!("[{}] TxID:   {}", idx + 1, record.tx_id);
                println!("    Kind:   {:?}", record.kind);
                println!("    Status: {}", record.status);
                println!("    Time:   {}", record.timestamp);
                println!("----------------------------------------------------------------");
            }
        }
        println!("\nPress Enter to return...");
        self.wait_keypress();
    }

    async fn view_new_address(&mut self) {
        println!("\n--- [7] Generate New Address / Keypair ---");
        let new_key = WalletKeypair::generate();
        println!("New Address:   {}", new_key.address);
        println!("New SecretKey: {}", new_key.secret_hex());

        let save = Self::read_input("Save this keypair as active wallet? (y/N): ");
        if save.to_lowercase() == "y" {
            self.ops.keypair = new_key;
            let path = WalletKeypair::default_key_path();
            if let Err(e) = self.ops.keypair.save_to_file(&path) {
                println!("Warning: Failed to save to {}: {}", path.display(), e);
            } else {
                println!("Saved to {}", path.display());
            }
        }
        println!("\nPress Enter to return...");
        self.wait_keypress();
    }

    async fn view_load_address(&mut self) {
        println!("\n--- [8] Load Keypair from File or Hex ---");
        let input = Self::read_input("Enter Private Key Hex or File Path: ");
        if input.is_empty() {
            println!("Cancelled.");
            self.wait_keypress();
            return;
        }

        let key_result = if input.len() == 64 || input.len() == 66 {
            WalletKeypair::from_secret_hex(&input)
        } else {
            WalletKeypair::load_from_file(std::path::Path::new(&input))
        };

        match key_result {
            Ok(keypair) => {
                println!("Loaded Address: {}", keypair.address);
                self.ops.keypair = keypair;
            }
            Err(e) => {
                println!("Error loading key: {}", e);
            }
        }
        println!("\nPress Enter to return...");
        self.wait_keypress();
    }

    async fn view_setup_demo(&mut self) {
        println!("\n--- [9] Setup / Deploy GHOST Token & DEX ---");
        let client = match self.ops.connect_rpc().await {
            Ok(c) => c,
            Err(e) => {
                println!("RPC Error: {}", e);
                self.wait_keypress();
                return;
            }
        };

        let owner_u64 = WalletOps::holder_u64(&self.ops.keypair.address);
        println!("1. Deploying GHOST Token contract (supply: 1,000,000, owner: {})...", owner_u64);

        match self.ops.deploy_token(&client, 1_000_000, owner_u64).await {
            Ok(token_addr) => {
                println!("   GHOST Token Deployed! Address: {}", token_addr);
                self.ghost_token_contract = Some(token_addr.clone());

                println!("\n2. Deploying DEX AMM contract...");
                match self.ops.deploy_dex(&client, None).await {
                    Ok(dex_addr) => {
                        println!("   DEX Deployed! Address: {}", dex_addr);
                        self.dex_contract = Some(dex_addr.clone());

                        println!("\n3. Initializing DEX liquidity (10,000 ZYAN / 10,000 GHOST)...");
                        let contract_address = zyanya_rpc_core::RpcHash::from_str(&dex_addr).unwrap();
                        let params = vec![owner_u64, 10_000, 10_000];
                        if let Ok(res) = client.invoke_contract(contract_address, 1, params, 100_000, 1, 0).await {
                            println!("   Liquidity Added! Tx: {}, LP Minted: {:?}", res.transaction_id, res.return_value);
                        }
                    }
                    Err(e) => println!("DEX deployment failed: {}", e),
                }
            }
            Err(e) => println!("Token deployment failed: {}", e),
        }

        let _ = client.disconnect().await;
        println!("\nSetup complete! Press Enter to return...");
        self.wait_keypress();
    }
}
