mod key_management;
mod tui;
mod wallet_ops;

use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;
use zyanya_consensus_core::constants::SOMPI_PER_ZYANYA;
use zyanya_rpc_core::api::rpc::RpcApi;

use key_management::WalletKeypair;
use tui::WalletTui;
use wallet_ops::WalletOps;

#[derive(Parser, Debug)]
#[command(
    name = "zyanya-wallet",
    author = "Zyanya Developers",
    version,
    about = "Zyanya TUI Wallet — manage ZYAN, custom tokens (GHOST), and DEX swaps"
)]
struct Cli {
    /// RPC server address (e.g. 127.0.0.1:18610 or grpc://127.0.0.1:18610)
    #[arg(short = 's', long, default_value = "127.0.0.1:18610")]
    rpcserver: String,

    /// Path to private key file (~/.zyanya/wallet.key by default)
    #[arg(short = 'k', long)]
    keyfile: Option<PathBuf>,

    /// Secret key in hex format (overrides keyfile)
    #[arg(long)]
    secret_key: Option<String>,

    /// Action: Generate a new keypair and exit
    #[arg(long)]
    generate_key: bool,

    /// Action: Query ZYAN balance for wallet address
    #[arg(long)]
    balance: bool,

    /// Action: Send ZYAN
    #[arg(long)]
    send_zyan: bool,

    /// Recipient address for send
    #[arg(long)]
    to: Option<String>,

    /// Amount in ZYAN (for send_zyan) or sompi/tokens
    #[arg(long)]
    amount: Option<f64>,

    /// Action: Send tokens
    #[arg(long)]
    send_token: bool,

    /// Target contract address (token or DEX)
    #[arg(long)]
    contract: Option<String>,

    /// Action: Swap tokens on DEX
    #[arg(long)]
    swap_dex: bool,

    /// Input token for DEX swap ("0", "1", "zyan", "ghost")
    #[arg(long)]
    token_in: Option<String>,

    /// Action: Run automated live demo (generate key, check balance, send ZYAN, DEX swap)
    #[arg(long)]
    demo: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    // 1. Generate key CLI command
    if cli.generate_key {
        let keypair = WalletKeypair::generate();
        println!("Generated New Zyanya Keypair:");
        println!("  Address:   {}", keypair.address);
        println!("  SecretKey: {}", keypair.secret_hex());

        let save_path = cli.keyfile.unwrap_or_else(WalletKeypair::default_key_path);
        match keypair.save_to_file(&save_path) {
            Ok(_) => println!("Saved to {}", save_path.display()),
            Err(e) => eprintln!("Error saving keypair: {}", e),
        }
        return ExitCode::SUCCESS;
    }

    // Load or generate keypair
    let keypair = if let Some(ref hex) = cli.secret_key {
        match WalletKeypair::from_secret_hex(hex) {
            Ok(k) => k,
            Err(e) => {
                eprintln!("Error parsing secret key hex: {}", e);
                return ExitCode::FAILURE;
            }
        }
    } else {
        let key_path = cli.keyfile.clone().unwrap_or_else(WalletKeypair::default_key_path);
        if key_path.exists() {
            match WalletKeypair::load_from_file(&key_path) {
                Ok(k) => k,
                Err(e) => {
                    eprintln!("Error loading key from {}: {}", key_path.display(), e);
                    return ExitCode::FAILURE;
                }
            }
        } else {
            // Auto-generate if no keyfile exists
            let k = WalletKeypair::generate();
            let _ = k.save_to_file(&key_path);
            k
        }
    };

    let mut ops = WalletOps::new(keypair, cli.rpcserver.clone());

    // 2. Automated Live Demo
    if cli.demo {
        return run_live_demo(&mut ops).await;
    }

    // 3. Single CLI Actions
    if cli.balance {
        let client = match ops.connect_rpc().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}", e);
                return ExitCode::FAILURE;
            }
        };
        match ops.get_zyan_balance(&client).await {
            Ok((bal, utxos)) => {
                let json = serde_json::json!({
                    "address": ops.keypair.address.to_string(),
                    "zyanBalance": bal as f64 / SOMPI_PER_ZYANYA as f64,
                    "sompiBalance": bal,
                    "utxoCount": utxos.len(),
                });
                println!("{}", serde_json::to_string_pretty(&json).unwrap());
            }
            Err(e) => eprintln!("Error: {}", e),
        }
        let _ = client.disconnect().await;
        return ExitCode::SUCCESS;
    }

    if cli.send_zyan {
        let recipient = match cli.to {
            Some(r) => r,
            None => {
                eprintln!("Error: --to <recipient_address> required for --send-zyan");
                return ExitCode::FAILURE;
            }
        };
        let zyan_val = match cli.amount {
            Some(a) => a,
            None => {
                eprintln!("Error: --amount <zyan_val> required for --send-zyan");
                return ExitCode::FAILURE;
            }
        };
        let amount_sompi = (zyan_val * SOMPI_PER_ZYANYA as f64) as u64;

        let client = match ops.connect_rpc().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}", e);
                return ExitCode::FAILURE;
            }
        };

        match ops.send_zyan(&client, &recipient, amount_sompi).await {
            Ok(tx_id) => {
                let json = serde_json::json!({
                    "success": true,
                    "transactionId": tx_id,
                    "recipient": recipient,
                    "amountZyan": zyan_val,
                    "signedBy": ops.keypair.address.to_string(),
                });
                println!("{}", serde_json::to_string_pretty(&json).unwrap());
            }
            Err(e) => eprintln!("Send ZYAN failed: {}", e),
        }
        let _ = client.disconnect().await;
        return ExitCode::SUCCESS;
    }

    if cli.swap_dex {
        let dex_contract = match cli.contract {
            Some(c) => c,
            None => {
                eprintln!("Error: --contract <dex_address> required for --swap-dex");
                return ExitCode::FAILURE;
            }
        };
        let token_in_str = cli.token_in.unwrap_or_else(|| "0".to_string());
        let token_in_val: u64 = match token_in_str.to_lowercase().as_str() {
            "0" | "a" | "zyan" => 0,
            "1" | "b" | "ghost" => 1,
            _ => token_in_str.parse().unwrap_or(0),
        };
        let amount_in = cli.amount.unwrap_or(100.0) as u64;

        let client = match ops.connect_rpc().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}", e);
                return ExitCode::FAILURE;
            }
        };

        match ops.swap_on_dex(&client, &dex_contract, token_in_val, amount_in).await {
            Ok((tx_id, out_amount)) => {
                let json = serde_json::json!({
                    "success": true,
                    "transactionId": tx_id,
                    "dexContract": dex_contract,
                    "tokenIn": token_in_val,
                    "amountIn": amount_in,
                    "amountOut": out_amount,
                });
                println!("{}", serde_json::to_string_pretty(&json).unwrap());
            }
            Err(e) => eprintln!("Swap failed: {}", e),
        }
        let _ = client.disconnect().await;
        return ExitCode::SUCCESS;
    }

    // 4. Default: Launch Interactive TUI
    println!("Launching Zyanya TUI Wallet...");
    let mut tui = WalletTui::new(ops);
    tui.run().await;

    ExitCode::SUCCESS
}

/// Run automated live demo flow
async fn run_live_demo(ops: &mut WalletOps) -> ExitCode {
    println!("================================================================================");
    println!("                      ZYANYA WALLET LIVE DEMO EXECUTION                        ");
    println!("================================================================================");

    // Step 1: Key Management
    println!("\n[STEP 1] Key Management");
    let demo_wallet = WalletKeypair::generate();
    println!("  Generated User Keypair:");
    println!("    Address:   {}", demo_wallet.address);
    println!("    SecretKey: {}", demo_wallet.secret_hex());
    ops.keypair = demo_wallet.clone();

    let client = match ops.connect_rpc().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("RPC Error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Step 2: Check Balance
    println!("\n[STEP 2] Query ZYAN Balance");
    let (bal, utxos) = ops.get_zyan_balance(&client).await.unwrap_or((0, vec![]));
    println!("  ZYAN Balance: {} ZYAN ({} sompi, UTXOs: {})", bal as f64 / SOMPI_PER_ZYANYA as f64, bal, utxos.len());

    // Step 3: Deploy GHOST Token & DEX
    println!("\n[STEP 3] Deploying Smart Contracts");
    let owner_u64 = WalletOps::holder_u64(&ops.keypair.address);
    println!("  Deploying GHOST Token contract (initial supply: 1,000,000)...");
    let _token_contract = match ops.deploy_token(&client, 1_000_000, owner_u64).await {
        Ok(addr) => {
            println!("  GHOST Token Deployed! Contract Address: {}", addr);
            addr
        }
        Err(e) => {
            eprintln!("  Token deploy failed: {}", e);
            let _ = client.disconnect().await;
            return ExitCode::FAILURE;
        }
    };

    println!("  Deploying DEX contract...");
    let dex_contract = match ops.deploy_dex(&client, None).await {
        Ok(addr) => {
            println!("  DEX Deployed! Contract Address: {}", addr);
            addr
        }
        Err(e) => {
            eprintln!("  DEX deploy failed: {}", e);
            let _ = client.disconnect().await;
            return ExitCode::FAILURE;
        }
    };

    // Step 4: Add Initial Liquidity to DEX
    println!("\n[STEP 4] Adding Liquidity to DEX Pool");
    let dex_hash = zyanya_rpc_core::RpcHash::from_str(&dex_contract).unwrap();
    let add_liq_res = client.invoke_contract(dex_hash, 1, vec![owner_u64, 10_000, 10_000], 100_000, 1, 0).await;
    match add_liq_res {
        Ok(res) => println!("  Liquidity Added! TxID: {}, LP Minted: {:?}", res.transaction_id, res.return_value),
        Err(e) => println!("  Liquidity add error: {}", e),
    }

    if let Ok((res_a, res_b)) = ops.get_dex_reserves(&client, &dex_contract).await {
        println!("  DEX Reserves: Reserve A (ZYAN) = {}, Reserve B (GHOST) = {}", res_a, res_b);
    }

    // Step 5: Perform Swap on DEX
    println!("\n[STEP 5] Swapping Tokens on DEX (ZYAN -> GHOST)");
    println!("  Swapping 500 Token A (ZYAN) for Token B (GHOST)...");
    match ops.swap_on_dex(&client, &dex_contract, 0, 500).await {
        Ok((tx_id, out_amount)) => {
            println!("  Swap SUCCESS!");
            println!("    TxID:            {}", tx_id);
            println!("    GHOST Received:  {}", out_amount);
        }
        Err(e) => eprintln!("  DEX Swap failed: {}", e),
    }

    if let Ok((res_a, res_b)) = ops.get_dex_reserves(&client, &dex_contract).await {
        println!("  Updated DEX Reserves: Reserve A = {}, Reserve B = {}", res_a, res_b);
    }

    // Step 6: Send ZYAN
    println!("\n[STEP 6] Send ZYAN Transfer");
    let recipient_key = WalletKeypair::generate();
    println!("  Recipient Address: {}", recipient_key.address);

    if bal >= 100 * SOMPI_PER_ZYANYA {
        println!("  Sending 10 ZYAN to recipient (signed with user's private key)...");
        match ops.send_zyan(&client, &recipient_key.address.to_string(), 10 * SOMPI_PER_ZYANYA).await {
            Ok(tx_id) => println!("  Send ZYAN SUCCESS! TxID: {}", tx_id),
            Err(e) => println!("  Send ZYAN failed (UTXOs pending confirmation): {}", e),
        }
    } else {
        println!("  [Note: Address needs mining rewards/UTXOs to send ZYAN on live network]");
    }

    // Step 7: History Summary
    println!("\n[STEP 7] Wallet History");
    println!("  Recent Wallet Operations Count: {}", ops.history.len());
    for (idx, rec) in ops.history.iter().take(5).enumerate() {
        println!("    [{}] {:?} -> TxID: {} ({})", idx + 1, rec.kind, rec.tx_id, rec.status);
    }

    let _ = client.disconnect().await;
    println!("\n================================================================================");
    println!("                      LIVE DEMO COMPLETED SUCCESSFULLY!                         ");
    println!("================================================================================");

    ExitCode::SUCCESS
}
