use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use zyanya_grpc_client::GrpcClient;
use zyanya_rpc_core::api::rpc::RpcApi;
use zyanya_rpc_core::RpcHash;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct RpcClientManager {
    rpc_url: String,
    client: Arc<RwLock<Option<GrpcClient>>>,
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
        Self {
            rpc_url,
            client: Arc::new(RwLock::new(None)),
        }
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
        let max_supply_zyan = coin_supply.as_ref().map(|s| s.max_sompi as f64 / 100_000_000.0).unwrap_or(21_000_000.0);

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
}
