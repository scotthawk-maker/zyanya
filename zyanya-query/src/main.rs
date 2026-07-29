use clap::{Parser, Subcommand};
use std::process::ExitCode;
use std::str::FromStr;
use zyanya_grpc_client::GrpcClient;
use zyanya_rpc_core::api::rpc::RpcApi;
use zyanya_rpc_core::RpcHash;

#[derive(Parser, Debug)]
#[command(
    name = "zyanya-query",
    author = "Zyanya Developers",
    version,
    about = "Simple non-interactive CLI for querying Zyanya node via gRPC"
)]
struct Cli {
    /// RPC server address (e.g. 127.0.0.1:18610 or grpc://127.0.0.1:18610)
    #[arg(short, long, default_value = "127.0.0.1:18610")]
    rpcserver: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get general node and chain state info
    #[command(name = "get-info", alias = "info")]
    GetInfo,

    /// Get block DAG info
    #[command(name = "get-dag-info", alias = "dag-info")]
    GetDagInfo,

    /// Get coin supply info
    #[command(name = "get-coin-supply", alias = "coin-supply")]
    GetCoinSupply,

    /// Get server info (version, network, sync state)
    #[command(name = "get-server-info", alias = "server-info")]
    GetServerInfo,

    /// Get known peer addresses
    #[command(name = "get-peer-addresses", alias = "peer-addresses")]
    GetPeerAddresses,

    /// Get connected peer info
    #[command(name = "get-connected-peer-info", alias = "connected-peers")]
    GetConnectedPeerInfo,

    /// Get sink (selected parent hash)
    #[command(name = "get-sink", alias = "sink")]
    GetSink,

    /// Get sink blue score
    #[command(name = "get-sink-blue-score", alias = "blue-score")]
    GetSinkBlueScore,

    /// Get block details by hash
    #[command(name = "get-block", alias = "block")]
    GetBlock {
        /// Hash of the block
        hash: String,
        /// Include transactions
        #[arg(long, default_value_t = false)]
        include_transactions: bool,
    },

    /// Get block headers
    #[command(name = "get-headers", alias = "headers")]
    GetHeaders {
        /// Start block hash
        #[arg(long)]
        start_hash: String,
        /// Number of headers to retrieve
        #[arg(long, default_value_t = 10)]
        limit: u64,
        /// Ascending order
        #[arg(long, default_value_t = true)]
        ascending: bool,
    },

    /// Compile a ZCL source file to bytecode hex
    #[command(name = "compile-contract", alias = "compile")]
    CompileContract {
        /// Path to .zcl source file
        #[arg(long, short)]
        source: String,
        /// Print generated assembly instead of bytecode hex
        #[arg(long, default_value_t = false)]
        asm: bool,
    },

    /// Deploy a smart contract bytecode
    #[command(name = "deploy-contract", alias = "deploy")]
    DeployContract {
        /// Compiled bytecode (hex encoded)
        #[arg(long)]
        bytecode: String,
        /// Maximum gas limit
        #[arg(long, default_value_t = 100000)]
        gas: u64,
        /// Gas price
        #[arg(long, default_value_t = 1)]
        gas_price: u64,
        /// Initial deposit amount
        #[arg(long, default_value_t = 0)]
        deposit: u64,
    },

    /// Invoke a smart contract
    #[command(name = "invoke-contract", alias = "invoke")]
    InvokeContract {
        /// Target contract address
        #[arg(long)]
        address: String,
        /// Calldata or parameters (hex encoded or integer)
        #[arg(long, default_value = "")]
        calldata: String,
        /// Entry point ID
        #[arg(long, default_value_t = 0)]
        entry_point: u16,
        /// Maximum gas limit
        #[arg(long, default_value_t = 100000)]
        gas: u64,
        /// Gas price
        #[arg(long, default_value_t = 1)]
        gas_price: u64,
        /// Deposit amount
        #[arg(long, default_value_t = 0)]
        deposit: u64,
    },

    /// Query a contract's persistent storage state
    #[command(name = "get-contract-state", alias = "contract-state")]
    GetContractState {
        /// Target contract address
        #[arg(long)]
        address: String,
        /// Storage key (hex or u64 integer)
        #[arg(long)]
        key: String,
    },

    /// Query a contract's deployed bytecode
    #[command(name = "get-contract-code", alias = "contract-code")]
    GetContractCode {
        /// Target contract address
        #[arg(long)]
        address: String,
    },

    /// Read-only call to execute a contract without submitting a transaction
    #[command(name = "call-contract", alias = "call")]
    CallContract {
        /// Target contract address
        #[arg(long)]
        address: String,
        /// Calldata (hex encoded or u64 integer)
        #[arg(long, default_value = "")]
        calldata: String,
        /// Entry point ID
        #[arg(long, default_value_t = 0)]
        entry_point: u16,
        /// Maximum gas limit
        #[arg(long, default_value_t = 100000)]
        gas: u64,
    },

    /// Deploy a reference token contract (ERC-20 style)
    #[command(name = "deploy-token", alias = "deploy-tok")]
    DeployToken {
        /// Initial total supply
        #[arg(long, default_value_t = 1000000)]
        supply: u64,
        /// Initial owner address or storage key
        #[arg(long, default_value = "1")]
        owner: String,
        /// Maximum gas limit
        #[arg(long, default_value_t = 100000)]
        gas: u64,
        /// Gas price
        #[arg(long, default_value_t = 1)]
        gas_price: u64,
        /// Initial deposit amount
        #[arg(long, default_value_t = 0)]
        deposit: u64,
    },

    /// Query a holder's token balance
    #[command(name = "token-balance", alias = "tok-bal")]
    TokenBalance {
        /// Target token contract address
        #[arg(long)]
        token: String,
        /// Holder address or storage key
        #[arg(long, default_value = "1")]
        holder: String,
    },

    /// Query token's total supply
    #[command(name = "token-supply", alias = "tok-supply")]
    TokenSupply {
        /// Target token contract address
        #[arg(long)]
        token: String,
    },

    /// Transfer tokens from sender to recipient
    #[command(name = "token-transfer", alias = "tok-transfer")]
    TokenTransfer {
        /// Target token contract address
        #[arg(long)]
        token: String,
        /// Sender address or key
        #[arg(long, default_value = "1")]
        from: String,
        /// Recipient address or key
        #[arg(long)]
        to: String,
        /// Transfer amount
        #[arg(long)]
        amount: u64,
        /// Maximum gas limit
        #[arg(long, default_value_t = 100000)]
        gas: u64,
        /// Gas price
        #[arg(long, default_value_t = 1)]
        gas_price: u64,
        /// Deposit amount
        #[arg(long, default_value_t = 0)]
        deposit: u64,
    },

    /// Mint new tokens to recipient
    #[command(name = "mint-token", alias = "mint-tok")]
    MintToken {
        /// Target token contract address
        #[arg(long)]
        token: String,
        /// Recipient address or key
        #[arg(long)]
        to: String,
        /// Mint amount
        #[arg(long)]
        amount: u64,
        /// Maximum gas limit
        #[arg(long, default_value_t = 100000)]
        gas: u64,
        /// Gas price
        #[arg(long, default_value_t = 1)]
        gas_price: u64,
        /// Deposit amount
        /// Deposit amount
        #[arg(long, default_value_t = 0)]
        deposit: u64,
    },

    /// Create a new DEX contract for token pair A and B
    #[command(name = "dex-create")]
    DexCreate {
        /// Token A address (or name)
        #[arg(long, default_value = "")]
        token_a: String,
        /// Token B address (or name)
        #[arg(long, default_value = "")]
        token_b: String,
        /// Path to ZCL source file
        #[arg(long, default_value = "dex.zcl")]
        source: String,
        /// Maximum gas limit
        #[arg(long, default_value_t = 100000)]
        gas: u64,
        /// Gas price
        #[arg(long, default_value_t = 1)]
        gas_price: u64,
        /// Deposit amount
        #[arg(long, default_value_t = 0)]
        deposit: u64,
    },

    /// Add liquidity to DEX pool
    #[command(name = "dex-add-liquidity")]
    DexAddLiquidity {
        /// Target DEX contract address
        #[arg(long)]
        dex: String,
        /// Amount of Token A to deposit
        #[arg(long)]
        amount_a: u64,
        /// Amount of Token B to deposit
        #[arg(long)]
        amount_b: u64,
        /// Caller ID / storage key
        #[arg(long, default_value = "10")]
        caller: String,
        /// Maximum gas limit
        #[arg(long, default_value_t = 100000)]
        gas: u64,
        /// Gas price
        #[arg(long, default_value_t = 1)]
        gas_price: u64,
        /// Deposit amount
        #[arg(long, default_value_t = 0)]
        deposit: u64,
    },

    /// Swap tokens in DEX pool
    #[command(name = "dex-swap")]
    DexSwap {
        /// Target DEX contract address
        #[arg(long)]
        dex: String,
        /// Token to swap in: 'a', 'b', '0', or '1'
        #[arg(long)]
        token_in: String,
        /// Amount of input token to swap
        #[arg(long)]
        amount_in: u64,
        /// Maximum gas limit
        #[arg(long, default_value_t = 100000)]
        gas: u64,
        /// Gas price
        #[arg(long, default_value_t = 1)]
        gas_price: u64,
        /// Deposit amount
        #[arg(long, default_value_t = 0)]
        deposit: u64,
    },

    /// Remove liquidity from DEX pool
    #[command(name = "dex-remove-liquidity")]
    DexRemoveLiquidity {
        /// Target DEX contract address
        #[arg(long)]
        dex: String,
        /// LP token amount to burn
        #[arg(long)]
        lp_amount: u64,
        /// Caller ID / storage key
        #[arg(long, default_value = "10")]
        caller: String,
        /// Maximum gas limit
        #[arg(long, default_value_t = 100000)]
        gas: u64,
        /// Gas price
        #[arg(long, default_value_t = 1)]
        gas_price: u64,
        /// Deposit amount
        #[arg(long, default_value_t = 0)]
        deposit: u64,
    },

    /// Query DEX reserves
    #[command(name = "dex-reserves")]
    DexReserves {
        /// Target DEX contract address
        #[arg(long)]
        dex: String,
    },

    /// Query DEX price ratio
    #[command(name = "dex-price")]
    DexPrice {
        /// Target DEX contract address
        #[arg(long)]
        dex: String,
    },
}

const DEFAULT_DEX_ZCL: &str = include_str!("../../dex.zcl");


#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Commands::CompileContract { ref source, asm } = cli.command {
        let content = match std::fs::read_to_string(source) {
            Ok(c) => c,
            Err(err) => {
                eprintln!("Error reading source file '{}': {}", source, err);
                return ExitCode::FAILURE;
            }
        };

        if asm {
            match zyanya_vm::Compiler::compile_to_assembly(&content) {
                Ok(asm_text) => {
                    println!("{}", asm_text);
                    return ExitCode::SUCCESS;
                }
                Err(err) => {
                    eprintln!("Compilation failed: {}", err);
                    return ExitCode::FAILURE;
                }
            }
        } else {
            match zyanya_vm::Compiler::compile(&content) {
                Ok(bytecode) => {
                    use zyanya_utils::hex::ToHex;
                    let hex_str = bytecode.to_hex();
                    let json = serde_json::json!({
                        "source": source,
                        "bytecode": hex_str,
                        "size_bytes": bytecode.len()
                    });
                    println!("{}", serde_json::to_string_pretty(&json).unwrap());
                    return ExitCode::SUCCESS;
                }
                Err(err) => {
                    eprintln!("Compilation failed: {}", err);
                    return ExitCode::FAILURE;
                }
            }
        }
    }

    let mut rpc_url = cli.rpcserver.clone();
    if !rpc_url.starts_with("grpc://") && !rpc_url.starts_with("http://") {
        rpc_url = format!("grpc://{}", rpc_url);
    }

    let client = match GrpcClient::connect(rpc_url.clone()).await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Error connecting to Zyanya RPC server at {}: {}", rpc_url, err);
            return ExitCode::FAILURE;
        }
    };

    let result = run_query(&client, cli.command).await;

    // Clean disconnect
    let _ = client.disconnect().await;

    match result {
        Ok(json_output) => {
            println!("{}", json_output);
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("RPC Query failed: {}", err);
            ExitCode::FAILURE
        }
    }
}

fn parse_u64_key(s: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let clean = s.trim();
    if let Some(stripped) = clean.strip_prefix("0x").or_else(|| clean.strip_prefix("0X")) {
        Ok(u64::from_str_radix(stripped, 16)?)
    } else {
        Ok(clean.parse::<u64>()?)
    }
}

async fn run_query(client: &GrpcClient, command: Commands) -> Result<String, Box<dyn std::error::Error>> {
    use zyanya_utils::hex::FromHex;

    let json_val = match command {
        Commands::CompileContract { .. } => unreachable!("CompileContract is handled offline"),
        Commands::GetInfo => {
            let res = client.get_info().await?;
            serde_json::to_value(&res)?
        }
        Commands::GetDagInfo => {
            let res = client.get_block_dag_info().await?;
            serde_json::to_value(&res)?
        }
        Commands::GetCoinSupply => {
            let res = client.get_coin_supply().await?;
            serde_json::to_value(&res)?
        }
        Commands::GetServerInfo => {
            let res = client.get_server_info().await?;
            serde_json::to_value(&res)?
        }
        Commands::GetPeerAddresses => {
            let res = client.get_peer_addresses().await?;
            serde_json::to_value(&res)?
        }
        Commands::GetConnectedPeerInfo => {
            let res = client.get_connected_peer_info().await?;
            serde_json::to_value(&res)?
        }
        Commands::GetSink => {
            let res = client.get_sink().await?;
            serde_json::to_value(&res)?
        }
        Commands::GetSinkBlueScore => {
            let res = client.get_sink_blue_score().await?;
            serde_json::json!({ "blueScore": res })
        }
        Commands::GetBlock { hash, include_transactions } => {
            let block_hash = RpcHash::from_str(&hash)?;
            let res = client.get_block(block_hash, include_transactions).await?;
            serde_json::to_value(&res)?
        }
        Commands::GetHeaders { start_hash, limit, ascending } => {
            let header_hash = RpcHash::from_str(&start_hash)?;
            let res = client.get_headers(header_hash, limit, ascending).await?;
            serde_json::to_value(&res)?
        }
        Commands::DeployContract { bytecode, gas, gas_price, deposit } => {
            let bytes = <Vec<u8>>::from_hex(bytecode.trim_start_matches("0x"))?;
            let res = client.deploy_contract(bytes, gas, gas_price, deposit).await?;
            serde_json::to_value(&res)?
        }
        Commands::InvokeContract { address, calldata, entry_point, gas, gas_price, deposit } => {
            let contract_address = RpcHash::from_str(&address)?;
            let parameters = if calldata.is_empty() {
                vec![]
            } else if let Ok(val) = calldata.parse::<u64>() {
                vec![val]
            } else {
                let bytes = <Vec<u8>>::from_hex(calldata.trim_start_matches("0x"))?;
                bytes.iter().map(|&b| b as u64).collect()
            };
            let res = client.invoke_contract(contract_address, entry_point, parameters, gas, gas_price, deposit).await?;
            serde_json::to_value(&res)?
        }
        Commands::GetContractState { address, key } => {
            let contract_address = RpcHash::from_str(&address)?;
            let key_val = parse_u64_key(&key)?;
            let res = client.get_contract_state(contract_address, key_val).await?;
            serde_json::to_value(&res)?
        }
        Commands::GetContractCode { address } => {
            let contract_address = RpcHash::from_str(&address)?;
            let res = client.get_contract_code(contract_address).await?;
            serde_json::to_value(&res)?
        }
        Commands::CallContract { address, calldata, entry_point, gas } => {
            let contract_address = RpcHash::from_str(&address)?;
            let mut bytes = if calldata.is_empty() {
                vec![]
            } else if let Ok(val) = calldata.parse::<u64>() {
                val.to_le_bytes().to_vec()
            } else {
                <Vec<u8>>::from_hex(calldata.trim_start_matches("0x"))?
            };
            bytes.extend_from_slice(&(entry_point as u64).to_le_bytes());
            let res = client.call_contract(contract_address, bytes, gas).await?;
            serde_json::to_value(&res)?
        }
        Commands::DeployToken { supply, owner, gas, gas_price, deposit } => {
            let owner_u64 = parse_u64_key(&owner)?;
            let bytecode = zyanya_vm::token_contract_bytecode(supply, owner_u64)?;
            let res = client.deploy_contract(bytecode, gas, gas_price, deposit).await?;
            serde_json::to_value(&res)?
        }
        Commands::TokenBalance { token, holder } => {
            let contract_address = RpcHash::from_str(&token)?;
            let holder_u64 = parse_u64_key(&holder)?;
            let state_res = client.get_contract_state(contract_address, holder_u64).await?;
            serde_json::json!({
                "token": token,
                "holder": holder_u64,
                "balance": state_res.value,
            })
        }
        Commands::TokenSupply { token } => {
            let contract_address = RpcHash::from_str(&token)?;
            let state_res = client.get_contract_state(contract_address, 0).await?;
            serde_json::json!({
                "token": token,
                "totalSupply": state_res.value,
            })
        }
        Commands::TokenTransfer { token, from, to, amount, gas, gas_price, deposit } => {
            let contract_address = RpcHash::from_str(&token)?;
            let from_u64 = parse_u64_key(&from)?;
            let to_u64 = parse_u64_key(&to)?;
            let parameters = vec![from_u64, to_u64, amount];
            let res = client.invoke_contract(contract_address, 0, parameters, gas, gas_price, deposit).await?;
            serde_json::to_value(&res)?
        }
        Commands::MintToken { token, to, amount, gas, gas_price, deposit } => {
            let contract_address = RpcHash::from_str(&token)?;
            let to_u64 = parse_u64_key(&to)?;
            let parameters = vec![to_u64, amount];
            let res = client.invoke_contract(contract_address, 3, parameters, gas, gas_price, deposit).await?;
            serde_json::to_value(&res)?
        }
        Commands::DexCreate { token_a, token_b, source, gas, gas_price, deposit } => {
            let content = match std::fs::read_to_string(&source) {
                Ok(c) => c,
                Err(_) => DEFAULT_DEX_ZCL.to_string(),
            };
            let bytecode = zyanya_vm::Compiler::compile(&content)?;
            let res = client.deploy_contract(bytecode, gas, gas_price, deposit).await?;
            serde_json::json!({
                "contractAddress": res.contract_address,
                "transactionId": res.transaction_id,
                "tokenA": token_a,
                "tokenB": token_b,
                "gasUsed": res.gas_used,
                "success": res.success
            })
        }
        Commands::DexAddLiquidity { dex, amount_a, amount_b, caller, gas, gas_price, deposit } => {
            let contract_address = RpcHash::from_str(&dex)?;
            let caller_u64 = parse_u64_key(&caller)?;
            let parameters = vec![caller_u64, amount_a, amount_b];
            let res = client.invoke_contract(contract_address, 1, parameters, gas, gas_price, deposit).await?;
            serde_json::to_value(&res)?
        }
        Commands::DexSwap { dex, token_in, amount_in, gas, gas_price, deposit } => {
            let contract_address = RpcHash::from_str(&dex)?;
            let token_in_val: u64 = match token_in.to_lowercase().as_str() {
                "a" | "0" | "zyan" => 0,
                "b" | "1" | "ghost" => 1,
                _ => token_in.parse::<u64>().unwrap_or(0),
            };
            let parameters = vec![token_in_val, amount_in];
            let res = client.invoke_contract(contract_address, 2, parameters, gas, gas_price, deposit).await?;
            serde_json::to_value(&res)?
        }
        Commands::DexRemoveLiquidity { dex, lp_amount, caller, gas, gas_price, deposit } => {
            let contract_address = RpcHash::from_str(&dex)?;
            let caller_u64 = parse_u64_key(&caller)?;
            let parameters = vec![caller_u64, lp_amount];
            let res = client.invoke_contract(contract_address, 3, parameters, gas, gas_price, deposit).await?;
            serde_json::to_value(&res)?
        }
        Commands::DexReserves { dex } => {
            let contract_address = RpcHash::from_str(&dex)?;
            let res_a = client.get_contract_state(contract_address, 0).await?;
            let res_b = client.get_contract_state(contract_address, 1).await?;
            let total_lp = client.get_contract_state(contract_address, 2).await?;
            serde_json::json!({
                "dex": dex,
                "reserveA": res_a.value,
                "reserveB": res_b.value,
                "totalLPSupply": total_lp.value
            })
        }
        Commands::DexPrice { dex } => {
            let contract_address = RpcHash::from_str(&dex)?;
            let res_a = client.get_contract_state(contract_address, 0).await?;
            let res_b = client.get_contract_state(contract_address, 1).await?;
            let r_a = res_a.value as f64;
            let r_b = res_b.value as f64;
            let price_a_per_b = if r_b > 0.0 { r_a / r_b } else { 0.0 };
            let price_b_per_a = if r_a > 0.0 { r_b / r_a } else { 0.0 };
            serde_json::json!({
                "dex": dex,
                "reserveA": res_a.value,
                "reserveB": res_b.value,
                "priceRatioAperB": price_a_per_b,
                "priceRatioBperA": price_b_per_a
            })
        }
    };

    Ok(serde_json::to_string_pretty(&json_val)?)
}
