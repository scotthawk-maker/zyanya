use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use zyanya_grpc_client::GrpcClient;
use zyanya_rpc_core::api::rpc::RpcApi;
use zyanya_rpc_core::RpcHash;
use zyanya_rpc_core::model::tx::RpcTransaction;
use zyanya_consensus_core::hashing::sighash::{calc_schnorr_signature_hash, SigHashReusedValuesUnsync};
use zyanya_consensus_core::hashing::sighash_type::SIG_HASH_ALL;
use zyanya_consensus_core::sign::verify;
use zyanya_consensus_core::tx::{ContractPayload, DeployContractPayload, SignableTransaction, Transaction, TransactionInput, TransactionOutput, UtxoEntry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedDeployTokenReq {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub supply: Option<u64>,
    pub slope: Option<u64>,
    pub description: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub website: Option<String>,
    pub icon_base64: Option<String>,
    pub gas: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignableTxData {
    pub tx: Transaction,
    pub entries: Vec<UtxoEntry>,
    pub contract_address: String,
    pub slope: u64,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub website: Option<String>,
    pub icon_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitSignedTxReq {
    pub unsigned_tx: String,
    pub signatures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenMetadata {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub website: Option<String>,
    pub icon_uri: Option<String>,
}

#[derive(Clone)]
pub struct RpcClientManager {
    rpc_url: String,
    client: Arc<RwLock<Option<GrpcClient>>>,
    pub metadata_store: Arc<tokio::sync::Mutex<std::collections::HashMap<String, TokenMetadata>>>,
    pub metadata_path: String,
    pub icons_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainDashboardInfo {
    pub block_count: u64,
    pub header_count: u64,
    pub difficulty: f64,
    pub network: String,
    pub is_synced: bool,
    pub server_version: String,
    pub virtual_daa_score: u64,
    pub past_median_time: u64,
    pub sink_hash: String,
    pub peer_count: usize,
    pub mempool_size: u64,
    pub coin_supply_zyan: f64,
    pub max_supply_zyan: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSummary {
    pub hash: String,
    pub blue_score: u64,
    pub daa_score: u64,
    pub timestamp: u64,
    pub tx_count: usize,
    pub selected_parent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseVestingOutput {
    pub index: usize,
    pub value_sompi: u64,
    pub value_zyan: f64,
    pub is_liquid: bool,
    pub lock_months: Option<usize>,
    pub address: String,
    pub script_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDetailView {
    pub hash: String,
    pub blue_score: u64,
    pub daa_score: u64,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
    pub version: u16,
    pub hash_merkle_root: String,
    pub accepted_id_merkle_root: String,
    pub utxo_commitment: String,
    pub pruning_point: String,
    pub selected_parent: String,
    pub parents: Vec<String>,
    pub children: Vec<String>,
    pub merge_set_blues: Vec<String>,
    pub merge_set_reds: Vec<String>,
    pub coinbase_vesting_outputs: Vec<CoinbaseVestingOutput>,
    pub transactions: Vec<TxSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxSummary {
    pub tx_id: String,
    pub hash: String,
    pub subnetwork_id: String,
    pub tx_type: String, // Transfer, DeployContract, InvokeContract, Coinbase
    pub lock_time: u64,
    pub gas: u64,
    pub mass: u64,
    pub input_count: usize,
    pub output_count: usize,
    pub total_output_zyan: f64,
    pub payload_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub address: String,
    pub bytecode_hex: String,
    pub bytecode_size: usize,
    pub deploy_tx_id: String,
    pub first_seen_block: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub contract_address: String,
    pub total_supply: u64,
    pub owner_address: u64,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSummary {
    pub address: String,
    pub bytecode_size: usize,
    pub deploy_tx_id: String,
    pub first_seen_block: String,
    pub contract_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSummary {
    pub contract_address: String,
    pub total_supply: u64,
    pub owner_address: u64,
    pub name: String,
    pub symbol: String,
    pub bytecode_size: usize,
    pub description: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub website: Option<String>,
    pub icon_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct DexSummary {
    pub address: String,
    pub reserveA: u64,
    pub reserveB: u64,
    pub totalLPSupply: u64,
    pub price: f64,
}

pub fn derive_contract_address(deploy_tx_id: &RpcHash, index: u32) -> RpcHash {
    use zyanya_hashes::{HasherBase, TransactionSigningHash};
    let mut hasher = TransactionSigningHash::new();
    hasher.update(deploy_tx_id.as_bytes());
    hasher.update(index.to_le_bytes());
    hasher.finalize()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagNode {
    pub hash: String,
    pub short_hash: String,
    pub blue_score: u64,
    pub daa_score: u64,
    pub parents: Vec<String>,
    pub selected_parent: String,
    pub is_chain_block: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagGraphData {
    pub nodes: Vec<DagNode>,
    pub sink: String,
}

impl RpcClientManager {
    pub fn new(rpc_url: String) -> Self {
        let metadata_path = std::env::var("ZYANYA_TOKEN_METADATA_PATH")
            .unwrap_or_else(|_| "token-metadata.json".to_string());
        let icons_dir = std::env::var("ZYANYA_TOKEN_ICONS_DIR")
            .unwrap_or_else(|_| "token-icons".to_string());

        if let Err(_) = std::fs::create_dir_all(&icons_dir) {
            let _ = std::fs::create_dir_all("/tmp/zyanya-token-icons");
        }

        let mut loaded_map = std::collections::HashMap::new();
        if let Ok(content) = std::fs::read_to_string(&metadata_path) {
            if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, TokenMetadata>>(&content) {
                loaded_map = map;
            }
        } else if let Ok(content) = std::fs::read_to_string("/tmp/zyanya-token-metadata.json") {
            if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, TokenMetadata>>(&content) {
                loaded_map = map;
            }
        }

        Self {
            rpc_url,
            client: Arc::new(RwLock::new(None)),
            metadata_store: Arc::new(tokio::sync::Mutex::new(loaded_map)),
            metadata_path,
            icons_dir,
        }
    }

    pub async fn get_token_metadata(&self, address: &str) -> Option<TokenMetadata> {
        let store = self.metadata_store.lock().await;
        store.get(address).or_else(|| store.get(&address.to_lowercase())).cloned()
    }

    pub async fn save_token_metadata(&self, address: &str, metadata: TokenMetadata) -> Result<(), String> {
        let mut store = self.metadata_store.lock().await;
        store.insert(address.to_string(), metadata.clone());
        store.insert(address.to_lowercase(), metadata);

        let json = serde_json::to_string_pretty(&*store)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

        if let Err(e) = std::fs::write(&self.metadata_path, &json) {
            let _ = std::fs::write("/tmp/zyanya-token-metadata.json", &json);
            log::warn!("Failed to write metadata to {}: {}, saved to /tmp", self.metadata_path, e);
        }
        Ok(())
    }

    pub fn save_token_icon(&self, address: &str, base64_data: &str) -> Result<String, String> {
        let decoded = decode_base64(base64_data)?;
        let filename = format!("{}.png", address);
        let mut path = std::path::Path::new(&self.icons_dir).join(&filename);

        if let Err(_) = std::fs::write(&path, &decoded) {
            let tmp_dir = std::path::Path::new("/tmp/zyanya-token-icons");
            let _ = std::fs::create_dir_all(tmp_dir);
            path = tmp_dir.join(&filename);
            std::fs::write(&path, &decoded).map_err(|e| format!("Failed to write icon to /tmp: {}", e))?;
        }

        Ok(format!("/token-icons/{}", filename))
    }

    pub async fn ensure_connected(&self) -> Result<GrpcClient, String> {
        {
            let lock = self.client.read().await;
            if let Some(ref c) = *lock {
                if c.is_connected() {
                    return Ok(c.clone());
                }
            }
        }

        let mut lock = self.client.write().await;
        let mut url = self.rpc_url.clone();
        if !url.starts_with("grpc://") && !url.starts_with("http://") {
            url = format!("grpc://{}", url);
        }

        match GrpcClient::connect(url.clone()).await {
            Ok(c) => {
                *lock = Some(c.clone());
                Ok(c)
            }
            Err(e) => Err(format!("Failed to connect to RPC at {}: {}", url, e)),
        }
    }

    pub async fn get_dashboard(&self) -> Result<ChainDashboardInfo, String> {
        let client = self.ensure_connected().await?;
        let info = client.get_info().await.map_err(|e| e.to_string())?;
        let dag_info = client.get_block_dag_info().await.map_err(|e| e.to_string())?;
        let coin_supply = client.get_coin_supply().await.ok();
        let peers = client.get_connected_peer_info().await.ok().map(|p| p.peer_info.len()).unwrap_or(0);

        let coin_supply_zyan = coin_supply.as_ref().map(|s| s.circulating_sompi as f64 / 100_000_000.0).unwrap_or(0.0);
        // The max supply is the total emission per the Zyanya schedule (~28.7B ZYAN),
        // NOT MAX_SOMPI (which is the max value of a single UTXO = 1.161B ZYAN).
        // See consensus/src/processes/coinbase.rs: total_supply_zyan approaches ~28.7B.
        let max_supply_zyan = 28_700_000_000.0;

        let sink_hash = dag_info.sink.to_string();

        Ok(ChainDashboardInfo {
            block_count: dag_info.block_count,
            header_count: dag_info.header_count,
            difficulty: dag_info.difficulty,
            network: dag_info.network.to_string(),
            is_synced: info.is_synced,
            server_version: info.server_version,
            virtual_daa_score: dag_info.virtual_daa_score,
            past_median_time: dag_info.past_median_time,
            sink_hash,
            peer_count: peers,
            mempool_size: info.mempool_size,
            coin_supply_zyan,
            max_supply_zyan,
        })
    }

    pub async fn get_recent_blocks(&self, limit: usize) -> Result<Vec<BlockSummary>, String> {
        let client = self.ensure_connected().await?;
        let dag_info = client.get_block_dag_info().await.map_err(|e| e.to_string())?;

        let mut current_hash = dag_info.sink;
        let mut summaries = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for _ in 0..limit {
            if visited.contains(&current_hash) {
                break;
            }
            visited.insert(current_hash);

            match client.get_block(current_hash, false).await {
                Ok(block) => {
                    let selected_parent = block.verbose_data.as_ref()
                        .map(|v| v.selected_parent_hash.to_string())
                        .unwrap_or_default();
                    let blue_score = block.header.blue_score;
                    let daa_score = block.header.daa_score;
                    let timestamp = block.header.timestamp;
                    let tx_count = block.verbose_data.as_ref()
                        .map(|v| v.transaction_ids.len())
                        .unwrap_or(0);

                    summaries.push(BlockSummary {
                        hash: current_hash.to_string(),
                        blue_score,
                        daa_score,
                        timestamp,
                        tx_count,
                        selected_parent: selected_parent.clone(),
                    });

                    if selected_parent.is_empty() || selected_parent == "0000000000000000000000000000000000000000000000000000000000000000" {
                        break;
                    }

                    if let Ok(next_hash) = RpcHash::from_str(&selected_parent) {
                        current_hash = next_hash;
                    } else {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        Ok(summaries)
    }

    pub async fn get_block_detail(&self, hash_str: &str) -> Result<BlockDetailView, String> {
        let client = self.ensure_connected().await?;
        let hash = RpcHash::from_str(hash_str).map_err(|e| format!("Invalid hash format: {}", e))?;
        let block = client.get_block(hash, true).await.map_err(|e| e.to_string())?;

        let verbose = block.verbose_data.ok_or_else(|| "Missing verbose data".to_string())?;

        let mut coinbase_vesting_outputs = Vec::new();
        let mut tx_summaries = Vec::new();

        for (idx, tx) in block.transactions.iter().enumerate() {
            let is_coinbase = idx == 0;
            let subnetwork_id = tx.subnetwork_id.to_string();

            let mut total_output_sompi: u64 = 0;
            for (out_idx, out) in tx.outputs.iter().enumerate() {
                total_output_sompi += out.value;

                if is_coinbase {
                    let script_hex = zyanya_utils::hex::ToHex::to_hex(&out.script_public_key.script());
                    let is_liquid = out_idx == 0;
                    let lock_months = if is_liquid { None } else { Some(out_idx) };
                    let addr = out.verbose_data.as_ref()
                        .map(|v| v.script_public_key_address.to_string())
                        .unwrap_or_else(|| {
                            if is_liquid {
                                "Liquid Output".to_string()
                            } else {
                                format!("Vested CSV Output #{}", out_idx)
                            }
                        });

                    coinbase_vesting_outputs.push(CoinbaseVestingOutput {
                        index: out_idx,
                        value_sompi: out.value,
                        value_zyan: out.value as f64 / 100_000_000.0,
                        is_liquid,
                        lock_months,
                        address: addr,
                        script_hex,
                    });
                }
            }

            let tx_id = tx.verbose_data.as_ref()
                .map(|v| v.transaction_id.to_string())
                .unwrap_or_else(|| format!("tx-{}", idx));

            let tx_type = if is_coinbase {
                "Coinbase".to_string()
            } else if subnetwork_id.ends_with("03") || subnetwork_id.contains("030000") {
                if tx.payload.is_empty() { "InvokeContract" } else { "DeployContract" }.to_string()
            } else {
                "Transfer".to_string()
            };

            let payload_hex = zyanya_utils::hex::ToHex::to_hex(&tx.payload);

            tx_summaries.push(TxSummary {
                tx_id,
                hash: tx.verbose_data.as_ref().map(|v| v.hash.to_string()).unwrap_or_default(),
                subnetwork_id,
                tx_type,
                lock_time: tx.lock_time,
                gas: tx.gas,
                mass: tx.mass,
                input_count: tx.inputs.len(),
                output_count: tx.outputs.len(),
                total_output_zyan: total_output_sompi as f64 / 100_000_000.0,
                payload_hex,
            });
        }

        let parents = block.header.parents_by_level.get(0)
            .map(|p| p.iter().map(|h| h.to_string()).collect())
            .unwrap_or_default();

        Ok(BlockDetailView {
            hash: verbose.hash.to_string(),
            blue_score: verbose.blue_score,
            daa_score: block.header.daa_score,
            timestamp: block.header.timestamp,
            bits: block.header.bits,
            nonce: block.header.nonce,
            version: block.header.version,
            hash_merkle_root: block.header.hash_merkle_root.to_string(),
            accepted_id_merkle_root: block.header.accepted_id_merkle_root.to_string(),
            utxo_commitment: block.header.utxo_commitment.to_string(),
            pruning_point: block.header.pruning_point.to_string(),
            selected_parent: verbose.selected_parent_hash.to_string(),
            parents,
            children: verbose.children_hashes.iter().map(|h| h.to_string()).collect(),
            merge_set_blues: verbose.merge_set_blues_hashes.iter().map(|h| h.to_string()).collect(),
            merge_set_reds: verbose.merge_set_reds_hashes.iter().map(|h| h.to_string()).collect(),
            coinbase_vesting_outputs,
            transactions: tx_summaries,
        })
    }

    pub async fn get_contract_code(&self, address_str: &str) -> Result<ContractInfo, String> {
        let client = self.ensure_connected().await?;
        let addr = RpcHash::from_str(address_str).map_err(|e| format!("Invalid contract address: {}", e))?;
        let res = client.get_contract_code(addr).await.map_err(|e| e.to_string())?;

        let hex = zyanya_utils::hex::ToHex::to_hex(&res.bytecode);
        let size = res.bytecode.len();

        Ok(ContractInfo {
            address: address_str.to_string(),
            bytecode_hex: hex,
            bytecode_size: size,
            deploy_tx_id: "On-chain deployed".to_string(),
            first_seen_block: "Active".to_string(),
        })
    }

    pub async fn get_contract_state_key(&self, address_str: &str, key: u64) -> Result<u64, String> {
        let client = self.ensure_connected().await?;
        let addr = RpcHash::from_str(address_str).map_err(|e| format!("Invalid contract address: {}", e))?;
        let res = client.get_contract_state(addr, key).await.map_err(|e| e.to_string())?;
        Ok(res.value)
    }

    pub async fn get_dag_graph(&self, limit: usize) -> Result<DagGraphData, String> {
        let client = self.ensure_connected().await?;
        let dag_info = client.get_block_dag_info().await.map_err(|e| e.to_string())?;

        let mut current_hash = dag_info.sink;
        let mut nodes = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for _ in 0..limit {
            if visited.contains(&current_hash) {
                break;
            }
            visited.insert(current_hash);

            match client.get_block(current_hash, false).await {
                Ok(block) => {
                    let selected_parent = block.verbose_data.as_ref()
                        .map(|v| v.selected_parent_hash.to_string())
                        .unwrap_or_default();
                    let is_chain = block.verbose_data.as_ref()
                        .map(|v| v.is_chain_block)
                        .unwrap_or(true);

                    let hash_s = current_hash.to_string();
                    let short_hash = if hash_s.len() > 12 {
                        format!("{}..{}", &hash_s[..6], &hash_s[hash_s.len()-4..])
                    } else {
                        hash_s.clone()
                    };

                    let parents = block.header.parents_by_level.get(0)
                        .map(|p| p.iter().map(|h| h.to_string()).collect())
                        .unwrap_or_default();

                    nodes.push(DagNode {
                        hash: hash_s,
                        short_hash,
                        blue_score: block.header.blue_score,
                        daa_score: block.header.daa_score,
                        parents,
                        selected_parent: selected_parent.clone(),
                        is_chain_block: is_chain,
                    });

                    if selected_parent.is_empty() || selected_parent == "0000000000000000000000000000000000000000000000000000000000000000" {
                        break;
                    }

                    if let Ok(next_hash) = RpcHash::from_str(&selected_parent) {
                        current_hash = next_hash;
                    } else {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        Ok(DagGraphData {
            nodes,
            sink: dag_info.sink.to_string(),
        })
    }

    pub async fn deploy_contract(&self, bytecode_hex: &str, gas: u64) -> Result<serde_json::Value, String> {
        use zyanya_utils::hex::FromHex;
        let client = self.ensure_connected().await?;
        let bytes = <Vec<u8>>::from_hex(bytecode_hex.trim_start_matches("0x"))
            .map_err(|e| format!("Invalid bytecode hex: {}", e))?;
        let res = client.deploy_contract(bytes, gas, 1, 0).await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "contractAddress": res.contract_address,
            "transactionId": res.transaction_id,
            "gasUsed": res.gas_used,
            "success": res.success
        }))
    }

    pub async fn invoke_contract(&self, address: &str, entry_point: u16, calldata: &str, gas: u64) -> Result<serde_json::Value, String> {
        use zyanya_utils::hex::FromHex;
        let client = self.ensure_connected().await?;
        let contract_address = RpcHash::from_str(address).map_err(|e| format!("Invalid contract address: {}", e))?;
        let parameters = if calldata.is_empty() {
            vec![]
        } else if calldata.contains(',') || calldata.contains(' ') {
            calldata.split(&[',', ' '][..])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| parse_u64_key(s))
                .collect::<Result<Vec<u64>, _>>()?
        } else if let Ok(val) = parse_u64_key(calldata) {
            vec![val]
        } else {
            let bytes = <Vec<u8>>::from_hex(calldata.trim_start_matches("0x"))
                .map_err(|e| format!("Invalid calldata hex: {}", e))?;
            bytes.iter().map(|&b| b as u64).collect()
        };
        let res = client.invoke_contract(contract_address, entry_point, parameters, gas, 1, 0).await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "returnValue": res.return_value,
            "transactionId": res.transaction_id,
            "gasUsed": res.gas_used,
            "success": res.success
        }))
    }

    pub async fn deploy_bonding_curve_token(
        &self,
        name: &str,
        symbol: &str,
        supply: u64,
        owner: &str,
        slope: u64,
        gas: u64,
        description: Option<String>,
        twitter: Option<String>,
        telegram: Option<String>,
        website: Option<String>,
        icon_base64: Option<String>,
    ) -> Result<serde_json::Value, String> {
        let client = self.ensure_connected().await?;
        let owner_u64 = parse_u64_key(owner)?;

        let bytecode = zyanya_vm::bonding_curve_token::bonding_curve_bytecode();
        let res = client.deploy_contract(bytecode, gas, 1, 0).await.map_err(|e| e.to_string())?;
        let contract_address = res.contract_address.to_string();

        let contract_hash = RpcHash::from_str(&contract_address).map_err(|e| e.to_string())?;
        let _init_res = client.invoke_contract(contract_hash, 0, vec![slope], gas, 1, 0).await.map_err(|e| format!("Init curve failed: {}", e))?;

        let icon_uri = if let Some(ref base64_str) = icon_base64 {
            if !base64_str.trim().is_empty() {
                match self.save_token_icon(&contract_address, base64_str) {
                    Ok(uri) => Some(uri),
                    Err(e) => {
                        log::warn!("Failed to save icon: {}", e);
                        Some(format!("/token-icons/{}.png", contract_address))
                    }
                }
            } else {
                Some(format!("/token-icons/{}.png", contract_address))
            }
        } else {
            Some(format!("/token-icons/{}.png", contract_address))
        };

        let metadata = TokenMetadata {
            name: Some(name.to_string()),
            symbol: Some(symbol.to_string()),
            description: description.clone(),
            twitter: twitter.clone(),
            telegram: telegram.clone(),
            website: website.clone(),
            icon_uri: icon_uri.clone(),
        };
        self.save_token_metadata(&contract_address, metadata).await?;

        Ok(serde_json::json!({
            "contract_address": contract_address,
            "contractAddress": contract_address,
            "transactionId": res.transaction_id,
            "gasUsed": res.gas_used,
            "success": res.success,
            "name": name,
            "symbol": symbol,
            "description": description,
            "socials": {
                "twitter": twitter,
                "telegram": telegram,
                "website": website
            },
            "icon_uri": icon_uri,
            "slope": slope,
            "supply": supply,
            "owner": owner_u64
        }))
    }

    /*
     =========================================================================================
       STEP 1 SIGNING SPIKE & ARCHITECTURE FINDINGS:
       ---------------------------------------------------------------------------------------
       1. Sighash Computation:
          Zyanya uses BIP 340 Schnorr sighashes computed via:
            `calc_schnorr_signature_hash(&signable_tx.as_verifiable(), input_idx, SIG_HASH_ALL, &reused_values)`
          where `signable_tx` is a `SignableTransaction` wrapping `Transaction` and UTXO `entries`.

       2. Signature Format:
          The browser signs the 32-byte sighash using BIP 340 Schnorr (@noble/curves or secp256k1).
          The resulting 64-byte signature is packed into `signature_script`:
            `signature_script = [0x41 (65)] + sig_64_bytes + [SIG_HASH_ALL (0x01)]` (66 bytes total).

       3. Node Submission:
          The assembled `Transaction` is converted into an `RpcTransaction`:
            `let rpc_tx = RpcTransaction::from(&tx);`
          and submitted to the node via `client.submit_transaction(rpc_tx, false)`.
     =========================================================================================
    */

    pub async fn build_unsigned_deploy_token_tx(
        &self,
        req: UnsignedDeployTokenReq,
    ) -> Result<serde_json::Value, String> {
        let client = self.ensure_connected().await?;
        let name = if req.name.trim().is_empty() { "Token".to_string() } else { req.name };
        let symbol = if req.symbol.trim().is_empty() { "TKN".to_string() } else { req.symbol };
        let supply = req.supply.unwrap_or(1_000_000);
        let slope = req.slope.unwrap_or(1);
        let gas = req.gas.unwrap_or(100_000);

        let user_address = parse_user_address(&req.address)?;

        // Fetch spendable UTXOs for user address
        let utxo_resp = client.get_utxos_by_addresses(vec![user_address.clone()]).await
            .unwrap_or_default();

        let virtual_daa_score = client.get_server_info().await
            .map(|s| s.virtual_daa_score)
            .unwrap_or(0);

        let mut selected_utxos = Vec::new();
        let mut total_in = 0u64;
        let fee = gas.saturating_mul(1);

        for entry in utxo_resp {
            if entry.utxo_entry.block_daa_score + 10 <= virtual_daa_score {
                let outpoint = zyanya_consensus_core::tx::TransactionOutpoint::from(entry.outpoint);
                let utxo_entry = zyanya_consensus_core::tx::UtxoEntry::from(entry.utxo_entry);
                total_in += utxo_entry.amount;
                selected_utxos.push((outpoint, utxo_entry));
                if total_in >= fee {
                    break;
                }
            }
        }

        let mut inputs = Vec::new();
        let mut entries = Vec::new();

        if !selected_utxos.is_empty() {
            for (outpoint, entry) in selected_utxos {
                inputs.push(TransactionInput {
                    previous_outpoint: outpoint,
                    signature_script: vec![],
                    sequence: 0,
                    sig_op_count: 1,
                });
                entries.push(entry);
            }
        } else {
            let dummy_outpoint = zyanya_consensus_core::tx::TransactionOutpoint::new(
                zyanya_consensus_core::tx::TransactionId::from_bytes([0u8; 32]),
                0,
            );
            let dummy_script = zyanya_txscript::pay_to_address_script(&user_address);
            let dummy_entry = UtxoEntry::new(fee, dummy_script, 0, false);
            inputs.push(TransactionInput {
                previous_outpoint: dummy_outpoint,
                signature_script: vec![],
                sequence: 0,
                sig_op_count: 1,
            });
            entries.push(dummy_entry);
            total_in = fee;
        }

        let change = total_in.saturating_sub(fee);
        let mut outputs = Vec::new();
        if change > 0 {
            let script_pub_key = zyanya_txscript::pay_to_address_script(&user_address);
            outputs.push(TransactionOutput {
                value: change,
                script_public_key: script_pub_key,
            });
        }

        let bytecode = zyanya_vm::bonding_curve_token::bonding_curve_bytecode();
        let payload = ContractPayload::Deploy(DeployContractPayload {
            bytecode,
            max_gas: gas,
            gas_price: 1,
            deposit_amount: 0,
        });
        let payload_bytes = payload.to_bytes().map_err(|e| e.to_string())?;

        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let unsigned_tx = Transaction::new(
            1,
            inputs,
            outputs,
            nonce,
            zyanya_consensus_core::subnets::SUBNETWORK_ID_SMART_CONTRACT,
            gas,
            payload_bytes,
        );

        let contract_address = derive_contract_address(&unsigned_tx.id(), 0).to_string();

        let signable_tx = SignableTransaction::with_entries(unsigned_tx.clone(), entries.clone());
        let reused_values = SigHashReusedValuesUnsync::new();
        let mut sighashes = Vec::new();
        for i in 0..unsigned_tx.inputs.len() {
            let hash = calc_schnorr_signature_hash(&signable_tx.as_verifiable(), i, SIG_HASH_ALL, &reused_values);
            sighashes.push(hash.to_string());
        }

        let icon_uri = if let Some(ref base64_str) = req.icon_base64 {
            if !base64_str.trim().is_empty() {
                match self.save_token_icon(&contract_address, base64_str) {
                    Ok(uri) => Some(uri),
                    Err(_) => Some(format!("/token-icons/{}.png", contract_address)),
                }
            } else {
                Some(format!("/token-icons/{}.png", contract_address))
            }
        } else {
            Some(format!("/token-icons/{}.png", contract_address))
        };

        let tx_data = SignableTxData {
            tx: unsigned_tx,
            entries,
            contract_address: contract_address.clone(),
            slope,
            name: name.clone(),
            symbol: symbol.clone(),
            description: req.description.clone(),
            twitter: req.twitter.clone(),
            telegram: req.telegram.clone(),
            website: req.website.clone(),
            icon_uri: icon_uri.clone(),
        };

        let json_bytes = serde_json::to_vec(&tx_data).map_err(|e| e.to_string())?;
        let unsigned_tx_hex = zyanya_utils::hex::ToHex::to_hex(&json_bytes);

        Ok(serde_json::json!({
            "unsigned_tx": unsigned_tx_hex,
            "sighashes": sighashes,
            "contract_address": contract_address,
            "summary": {
                "name": name,
                "symbol": symbol,
                "supply": supply,
                "slope": slope,
                "fee_zyan": fee as f64 / 100_000_000.0,
                "user_address": user_address.to_string(),
                "input_count": tx_data.tx.inputs.len(),
            }
        }))
    }

    pub async fn submit_signed_tx(
        &self,
        req: SubmitSignedTxReq,
    ) -> Result<serde_json::Value, String> {
        use zyanya_utils::hex::FromHex;
        let client = self.ensure_connected().await?;

        let json_bytes = <Vec<u8>>::from_hex(req.unsigned_tx.trim_start_matches("0x"))
            .map_err(|e| format!("Invalid unsigned_tx hex: {}", e))?;
        let data: SignableTxData = serde_json::from_slice(&json_bytes)
            .map_err(|e| format!("Failed to parse unsigned transaction payload: {}", e))?;

        let mut signable_tx = SignableTransaction::with_entries(data.tx.clone(), data.entries);

        if req.signatures.len() != signable_tx.tx.inputs.len() {
            return Err(format!(
                "Signature count mismatch: expected {}, got {}",
                signable_tx.tx.inputs.len(),
                req.signatures.len()
            ));
        }

        for (i, sig_hex) in req.signatures.iter().enumerate() {
            let sig_bytes = <Vec<u8>>::from_hex(sig_hex.trim_start_matches("0x"))
                .map_err(|e| format!("Invalid signature hex for input {}: {}", i, e))?;
            if sig_bytes.len() != 64 {
                return Err(format!("Signature for input {} must be 64 bytes, got {}", i, sig_bytes.len()));
            }
            signable_tx.tx.inputs[i].signature_script = std::iter::once(65u8)
                .chain(sig_bytes)
                .chain([SIG_HASH_ALL.to_u8()])
                .collect();
        }

        let _ = verify(&signable_tx.as_verifiable());

        let rpc_tx = RpcTransaction::from(&signable_tx.tx);
        let tx_id = client.submit_transaction(rpc_tx, false).await
            .map_err(|e| format!("SubmitTransaction RPC failed: {}", e))?;

        let contract_hash = RpcHash::from_str(&data.contract_address).map_err(|e| e.to_string())?;
        let _init_res = client.invoke_contract(contract_hash, 0, vec![data.slope], 100_000, 1, 0).await.ok();

        let metadata = TokenMetadata {
            name: Some(data.name.clone()),
            symbol: Some(data.symbol.clone()),
            description: data.description.clone(),
            twitter: data.twitter.clone(),
            telegram: data.telegram.clone(),
            website: data.website.clone(),
            icon_uri: data.icon_uri.clone(),
        };
        self.save_token_metadata(&data.contract_address, metadata).await?;

        Ok(serde_json::json!({
            "success": true,
            "transactionId": tx_id.to_string(),
            "transaction_id": tx_id.to_string(),
            "contractAddress": data.contract_address,
            "contract_address": data.contract_address,
            "name": data.name,
            "symbol": data.symbol,
        }))
    }

    pub async fn call_contract(&self, address: &str, calldata: &str, entry_point: u16, gas: u64) -> Result<serde_json::Value, String> {
        use zyanya_utils::hex::FromHex;
        let client = self.ensure_connected().await?;
        let contract_address = RpcHash::from_str(address).map_err(|e| format!("Invalid contract address: {}", e))?;
        let mut bytes = if calldata.is_empty() {
            vec![]
        } else if let Ok(val) = calldata.parse::<u64>() {
            val.to_le_bytes().to_vec()
        } else {
            <Vec<u8>>::from_hex(calldata.trim_start_matches("0x"))
                .map_err(|e| format!("Invalid calldata hex: {}", e))?
        };
        bytes.extend_from_slice(&(entry_point as u64).to_le_bytes());
        let res = client.call_contract(contract_address, bytes, gas).await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "returnValue": res.return_value,
            "executionSuccess": res.success,
            "gasUsed": res.gas_used
        }))
    }

    pub async fn deploy_token(&self, name: &str, supply: u64, owner: &str, gas: u64) -> Result<serde_json::Value, String> {
        let client = self.ensure_connected().await?;
        let owner_u64 = parse_u64_key(owner)?;
        let bytecode = zyanya_vm::token_contract_bytecode(supply, owner_u64).map_err(|e| e.to_string())?;
        let res = client.deploy_contract(bytecode, gas, 1, 0).await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "contractAddress": res.contract_address,
            "transactionId": res.transaction_id,
            "gasUsed": res.gas_used,
            "success": res.success,
            "name": name,
            "supply": supply,
            "owner": owner_u64
        }))
    }

    pub async fn get_token_balance(&self, token: &str, holder: &str) -> Result<serde_json::Value, String> {
        let client = self.ensure_connected().await?;
        let contract_address = RpcHash::from_str(token).map_err(|e| format!("Invalid token address: {}", e))?;
        let holder_u64 = parse_u64_key(holder)?;
        let state_res = client.get_contract_state(contract_address, holder_u64).await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "token": token,
            "holder": holder_u64,
            "balance": state_res.value
        }))
    }

    pub async fn token_transfer(&self, token: &str, from: &str, to: &str, amount: u64, gas: u64) -> Result<serde_json::Value, String> {
        let client = self.ensure_connected().await?;
        let contract_address = RpcHash::from_str(token).map_err(|e| format!("Invalid token address: {}", e))?;
        let from_u64 = parse_u64_key(from)?;
        let to_u64 = parse_u64_key(to)?;
        let parameters = vec![from_u64, to_u64, amount];
        let res = client.invoke_contract(contract_address, 0, parameters, gas, 1, 0).await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "token": token,
            "from": from_u64,
            "to": to_u64,
            "amount": amount,
            "transactionId": res.transaction_id,
            "gasUsed": res.gas_used,
            "success": res.success,
            "returnValue": res.return_value
        }))
    }

    pub async fn swap_on_dex(&self, dex: &str, token_in: &str, amount_in: u64, gas: u64) -> Result<serde_json::Value, String> {
        let client = self.ensure_connected().await?;
        let contract_address = RpcHash::from_str(dex).map_err(|e| format!("Invalid DEX address: {}", e))?;
        let token_in_val: u64 = match token_in.to_lowercase().as_str() {
            "a" | "0" | "zyan" => 0,
            "b" | "1" | "ghost" => 1,
            _ => token_in.parse::<u64>().unwrap_or(0),
        };
        let parameters = vec![token_in_val, amount_in];
        let res = client.invoke_contract(contract_address, 2, parameters, gas, 1, 0).await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "dex": dex,
            "tokenIn": token_in,
            "tokenInValue": token_in_val,
            "amountIn": amount_in,
            "amountOut": res.return_value,
            "transactionId": res.transaction_id,
            "gasUsed": res.gas_used,
            "success": res.success
        }))
    }

    pub async fn get_dex_reserves(&self, dex: &str) -> Result<serde_json::Value, String> {
        let client = self.ensure_connected().await?;
        let contract_address = RpcHash::from_str(dex).map_err(|e| format!("Invalid DEX address: {}", e))?;
        let res_a = client.get_contract_state(contract_address, 0).await.map_err(|e| e.to_string())?;
        let res_b = client.get_contract_state(contract_address, 1).await.map_err(|e| e.to_string())?;
        let total_lp = client.get_contract_state(contract_address, 2).await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "dex": dex,
            "reserveA": res_a.value,
            "reserveB": res_b.value,
            "totalLPSupply": total_lp.value
        }))
    }

    pub fn compile_contract(&self, source: &str) -> Result<serde_json::Value, String> {
        use zyanya_utils::hex::ToHex;
        let bytecode = zyanya_vm::Compiler::compile(source).map_err(|e| e.to_string())?;
        let hex_str = bytecode.to_hex();
        Ok(serde_json::json!({
            "bytecode": hex_str,
            "size_bytes": bytecode.len()
        }))
    }

    pub async fn get_contracts(&self) -> Result<Vec<ContractSummary>, String> {
        let client = self.ensure_connected().await?;
        let dag_info = client.get_block_dag_info().await.map_err(|e| e.to_string())?;

        let mut known_addresses: std::collections::HashSet<String> = std::collections::HashSet::new();
        // Seed known contracts on chain (DEX + GHOST token)
        known_addresses.insert("3d208f19ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf483".to_string());
        known_addresses.insert("cef968ca5d9ea40d306224efb988b2b408d3c751f8b8baea10c1e7caafb4fe40".to_string());

        let mut current_hash = dag_info.sink;
        let mut visited = std::collections::HashSet::new();

        for _ in 0..500 {
            if visited.contains(&current_hash) {
                break;
            }
            visited.insert(current_hash);

            if let Ok(block) = client.get_block(current_hash, true).await {
                for tx in &block.transactions {
                    let subnetwork_id = tx.subnetwork_id.to_string();
                    let is_contract_subnetwork = subnetwork_id.ends_with("03") || subnetwork_id.contains("030000");
                    if is_contract_subnetwork && !tx.payload.is_empty() {
                        let tx_id_str = tx.verbose_data.as_ref()
                            .map(|v| v.transaction_id.to_string())
                            .unwrap_or_default();
                        if let Ok(tx_hash) = RpcHash::from_str(&tx_id_str) {
                            let derived_addr = derive_contract_address(&tx_hash, 0);
                            known_addresses.insert(derived_addr.to_string());
                        }
                    }
                }

                let selected_parent = block.verbose_data.as_ref()
                    .map(|v| v.selected_parent_hash.to_string())
                    .unwrap_or_default();
                if selected_parent.is_empty() || selected_parent == "0000000000000000000000000000000000000000000000000000000000000000" {
                    break;
                }
                if let Ok(next_hash) = RpcHash::from_str(&selected_parent) {
                    current_hash = next_hash;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let mut contracts = Vec::new();
        for addr in known_addresses {
            if let Ok(info) = self.get_contract_code(&addr).await {
                if info.bytecode_size > 0 {
                    let k0 = self.get_contract_state_key(&addr, 0).await.unwrap_or(0);
                    let k1 = self.get_contract_state_key(&addr, 1).await.unwrap_or(0);
                    let k2 = self.get_contract_state_key(&addr, 2).await.unwrap_or(0);

                    let contract_type = if k0 > 0 && k1 > 0 && k2 > 0 {
                        "DEX".to_string()
                    } else if k0 > 0 {
                        "Token".to_string()
                    } else {
                        "Contract".to_string()
                    };

                    contracts.push(ContractSummary {
                        address: addr,
                        bytecode_size: info.bytecode_size,
                        deploy_tx_id: info.deploy_tx_id,
                        first_seen_block: info.first_seen_block,
                        contract_type,
                    });
                }
            }
        }

        contracts.sort_by(|a, b| a.contract_type.cmp(&b.contract_type).then_with(|| a.address.cmp(&b.address)));
        Ok(contracts)
    }

    pub async fn get_tokens(&self) -> Result<Vec<TokenSummary>, String> {
        let contracts = self.get_contracts().await?;
        let mut tokens = Vec::new();
        let store = self.metadata_store.lock().await;

        for c in contracts {
            if c.contract_type == "Token" {
                let k0 = self.get_contract_state_key(&c.address, 0).await.unwrap_or(0);
                let k1 = self.get_contract_state_key(&c.address, 1).await.unwrap_or(0);

                let meta = store.get(&c.address).or_else(|| store.get(&c.address.to_lowercase()));
                let name = meta.and_then(|m| m.name.clone()).unwrap_or_else(|| "GHOST Token".to_string());
                let symbol = meta.and_then(|m| m.symbol.clone()).unwrap_or_else(|| "GHOST".to_string());
                let description = meta.and_then(|m| m.description.clone());
                let twitter = meta.and_then(|m| m.twitter.clone());
                let telegram = meta.and_then(|m| m.telegram.clone());
                let website = meta.and_then(|m| m.website.clone());
                let icon_uri = meta.and_then(|m| m.icon_uri.clone());

                tokens.push(TokenSummary {
                    contract_address: c.address.clone(),
                    total_supply: k0,
                    owner_address: k1,
                    name,
                    symbol,
                    bytecode_size: c.bytecode_size,
                    description,
                    twitter,
                    telegram,
                    website,
                    icon_uri,
                });
            }
        }

        Ok(tokens)
    }

    pub async fn get_dexes(&self) -> Result<Vec<DexSummary>, String> {
        let contracts = self.get_contracts().await?;
        let mut dexes = Vec::new();

        for c in contracts {
            if c.contract_type == "DEX" {
                let k0 = self.get_contract_state_key(&c.address, 0).await.unwrap_or(0);
                let k1 = self.get_contract_state_key(&c.address, 1).await.unwrap_or(0);
                let k2 = self.get_contract_state_key(&c.address, 2).await.unwrap_or(0);

                let price = if k0 > 0 { k1 as f64 / k0 as f64 } else { 0.0 };
                dexes.push(DexSummary {
                    address: c.address.clone(),
                    reserveA: k0,
                    reserveB: k1,
                    totalLPSupply: k2,
                    price,
                });
            }
        }

        Ok(dexes)
    }
}

fn parse_u64_key(s: &str) -> Result<u64, String> {
    let clean = s.trim();
    if let Some(stripped) = clean.strip_prefix("0x").or_else(|| clean.strip_prefix("0X")) {
        u64::from_str_radix(stripped, 16).map_err(|e| format!("Invalid hex key: {}", e))
    } else {
        clean.parse::<u64>().map_err(|e| format!("Invalid numeric key: {}", e))
    }
}

pub fn decode_base64(s: &str) -> Result<Vec<u8>, String> {
    let clean = if let Some(pos) = s.find(',') {
        &s[pos + 1..]
    } else {
        s
    }.trim();

    let mut table = [255u8; 256];
    for (i, &b) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
        table[b as usize] = i as u8;
    }
    let bytes = clean.as_bytes();
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;
    for &b in bytes {
        if b == b'=' || b.is_ascii_whitespace() { continue; }
        let val = table[b as usize];
        if val == 255 {
            continue;
        }
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Ok(out)
}

fn parse_user_address(address_str: &str) -> Result<zyanya_addresses::Address, String> {
    use zyanya_utils::hex::FromHex;
    let clean = address_str.trim();
    if let Ok(addr) = zyanya_addresses::Address::try_from(clean) {
        return Ok(addr);
    }
    if clean.len() == 64 {
        if let Ok(bytes) = <Vec<u8>>::from_hex(clean) {
            if bytes.len() == 32 {
                return Ok(zyanya_addresses::Address::new(
                    zyanya_addresses::Prefix::Testnet,
                    zyanya_addresses::Version::PubKey,
                    &bytes,
                ));
            }
        }
    }
    Err(format!("Invalid Zyanya address or public key format: {}", address_str))
}

