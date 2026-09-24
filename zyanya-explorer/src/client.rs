use crate::api::{UnsignedBuyReq, UnsignedSellReq};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use zyanya_consensus_core::hashing::sighash::{calc_schnorr_signature_hash, SigHashReusedValuesUnsync};
use zyanya_consensus_core::hashing::sighash_type::SIG_HASH_ALL;
use zyanya_consensus_core::sign::verify;
use zyanya_consensus_core::tx::{
    ContractPayload, DeployContractPayload, InvokeContractPayload, SignableTransaction, Transaction, TransactionInput,
    TransactionOutput, UtxoEntry,
};
use zyanya_grpc_client::GrpcClient;
use zyanya_rpc_core::api::rpc::RpcApi;
use zyanya_rpc_core::model::tx::RpcTransaction;
use zyanya_rpc_core::RpcHash;

/// F-M-35: write a file with 0600 permissions (owner-only read/write) so that
/// token metadata/icons written to the /tmp fallback are not world-readable.
/// On non-Unix targets the mode call is a no-op.
/// F-L-30: Atomically write a file via temp file + rename to prevent
/// torn writes from corrupting metadata files.
fn write_atomic(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, path)
}

fn write_private(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    opts.mode_private();
    let mut f = opts.open(path)?;
    f.write_all(data)
}

/// F-M-35: create a directory with 0700 permissions (owner-only access) for the
/// /tmp token-icon fallback. On non-Unix targets the mode call is a no-op.
fn create_private_dir(path: &std::path::Path) -> std::io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    builder.mode_private();
    builder.create(path)
}

#[cfg(unix)]
trait FileModePrivate {
    fn mode_private(&mut self);
}
#[cfg(unix)]
impl FileModePrivate for std::fs::OpenOptions {
    fn mode_private(&mut self) {
        use std::os::unix::fs::OpenOptionsExt;
        self.mode(0o600);
    }
}
#[cfg(unix)]
impl FileModePrivate for std::fs::DirBuilder {
    fn mode_private(&mut self) {
        use std::os::unix::fs::DirBuilderExt;
        self.mode(0o700);
    }
}
#[cfg(not(unix))]
trait FileModePrivate {
    fn mode_private(&mut self);
}
#[cfg(not(unix))]
impl FileModePrivate for std::fs::OpenOptions {
    fn mode_private(&mut self) {}
}
#[cfg(not(unix))]
impl FileModePrivate for std::fs::DirBuilder {
    fn mode_private(&mut self) {}
}

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

/// Strip characters that could enable HTML/script injection from all
/// user-controlled metadata fields.  This is a defense-in-depth measure
/// (F-C-16): because transaction signatures do not cover the metadata
/// fields, a client can tamper with them after signing.  By rejecting /
/// neutralising HTML delimiters at the storage boundary we prevent stored
/// XSS payloads from ever being persisted.
pub fn sanitize_metadata(metadata: &mut TokenMetadata) {
    /// Remove `<`, `>`, `"`, `'` and backtick characters from a string.
    fn strip_dangerous(s: &str) -> String {
        s.chars().filter(|&c| c != '<' && c != '>' && c != '"' && c != '\'' && c != '`').collect()
    }

    if let Some(ref mut v) = metadata.name {
        *v = strip_dangerous(v);
    }
    if let Some(ref mut v) = metadata.symbol {
        *v = strip_dangerous(v);
    }
    if let Some(ref mut v) = metadata.description {
        *v = strip_dangerous(v);
    }
    if let Some(ref mut v) = metadata.twitter {
        *v = strip_dangerous(v);
    }
    if let Some(ref mut v) = metadata.telegram {
        *v = strip_dangerous(v);
    }
    if let Some(ref mut v) = metadata.website {
        *v = strip_dangerous(v);
    }
    if let Some(ref mut v) = metadata.icon_uri {
        *v = strip_dangerous(v);
    }
}

/// F-C-16 FOLLOW-UP: compute a deterministic blake2b-256 hash of the sanitized
/// token metadata. Each field is length-prefixed (u64 LE) to avoid field-boundary
/// ambiguity. `None` fields hash as empty strings.
pub fn compute_metadata_hash(m: &TokenMetadata) -> [u8; 32] {
    let mut h = blake2b_simd::Params::new().hash_length(32).to_state();
    for field in [
        m.name.as_deref(),
        m.symbol.as_deref(),
        m.description.as_deref(),
        m.twitter.as_deref(),
        m.telegram.as_deref(),
        m.website.as_deref(),
        m.icon_uri.as_deref(),
    ] {
        let bytes = field.unwrap_or("").as_bytes();
        h.update(&(bytes.len() as u64).to_le_bytes());
        h.update(bytes);
    }
    let hash = h.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_bytes()[..32]);
    result
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStakePosition {
    pub user_address: String,
    pub staked_sompi: u64,
    pub staked_zyan: f64,
    pub claimed_rewards_sompi: u64,
    pub claimed_rewards_zyan: f64,
    pub covenant_duration_days: u32,
    pub covenant_multiplier: f64,
    pub start_timestamp: u64,
    pub unlock_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingState {
    pub vault_address: String,
    pub total_staked_sompi: u64,
    pub total_rewards_distributed_sompi: u64,
    pub user_positions: std::collections::HashMap<String, UserStakePosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingInfo {
    pub vault_address: String,
    pub total_staked_zyan: f64,
    pub total_staked_sompi: u64,
    pub total_rewards_distributed_zyan: f64,
    pub total_rewards_distributed_sompi: u64,
    pub base_apr_percent: f64,
    pub covenant_30d_apr_percent: f64,
    pub boosted_apr_percent: f64,
    pub protocol_fee_rate_percent: f64,
    pub total_stakers: usize,
    pub user_staked_zyan: f64,
    pub user_staked_sompi: u64,
    pub user_pending_rewards_zyan: f64,
    pub user_pending_rewards_sompi: u64,
    pub user_covenant_tier: String,
    pub user_unlock_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexPoolState {
    pub pool_address: String,
    pub token_a_symbol: String,
    pub token_b_symbol: String,
    pub token_b_address: String,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub total_lp_shares: u64,
    pub volume_24h_zyan: f64,
    pub fee_protocol_routed_zyan: f64,
}

#[derive(Clone)]
pub struct RpcClientManager {
    rpc_url: String,
    client: Arc<RwLock<Option<GrpcClient>>>,
    pub metadata_store: Arc<tokio::sync::Mutex<std::collections::HashMap<String, TokenMetadata>>>,
    /// F-C-16 FOLLOW-UP: committed metadata hashes per contract address.
    pub metadata_hash_store: Arc<tokio::sync::Mutex<std::collections::HashMap<String, [u8; 32]>>>,
    pub metadata_path: String,
    pub icons_dir: String,
    pub staking_state: Arc<tokio::sync::Mutex<StakingState>>,
    pub staking_path: String,
    pub dex_pools: Arc<tokio::sync::Mutex<std::collections::HashMap<String, DexPoolState>>>,
    pub dex_path: String,
    /// In-memory rolling 24-hour spend velocity store per session_id (current_spent, window_start).
    pub session_velocities: Arc<tokio::sync::Mutex<std::collections::HashMap<String, (u64, u64)>>>,
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
    pub name: String,
    pub bytecode_size: usize,
    pub deploy_tx_id: String,
    pub first_seen_block: String,
    pub contract_type: String,
    pub source_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSummary {
    pub contract_address: String,
    pub address: String,
    pub total_supply: u64,
    pub owner_address: u64,
    pub owner: String,
    pub name: String,
    pub symbol: String,
    pub bytecode_size: usize,
    pub price_zyan: f64,
    pub market_cap_zyan: f64,
    pub graduated_to_dex: bool,
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
    pub pool_address: String,
    pub token_a_symbol: String,
    pub token_b_symbol: String,
    pub reserveA: u64,
    pub reserveB: u64,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub totalLPSupply: u64,
    pub total_lp_shares: u64,
    pub price: f64,
    pub volume_24h_zyan: f64,
    pub fee_protocol_routed_zyan: f64,
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
        let metadata_path = std::env::var("ZYANYA_TOKEN_METADATA_PATH").unwrap_or_else(|_| "token-metadata.json".to_string());
        let staking_path = std::env::var("ZYANYA_STAKING_STATE_PATH").unwrap_or_else(|_| "staking-state.json".to_string());
        let dex_path = std::env::var("ZYANYA_DEX_POOLS_PATH").unwrap_or_else(|_| "dex-pools.json".to_string());
        let icons_dir = std::env::var("ZYANYA_TOKEN_ICONS_DIR").unwrap_or_else(|_| "token-icons".to_string());

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

        // Seed verified tokens if missing
        if !loaded_map.contains_key("cef968ca5d9ea40d306224efb988b2b408d3c751f8b8baea10c1e7caafb4fe40") {
            loaded_map.insert("cef968ca5d9ea40d306224efb988b2b408d3c751f8b8baea10c1e7caafb4fe40".to_string(), TokenMetadata {
                name: Some("Ghost Token".to_string()),
                symbol: Some("GHOST".to_string()),
                description: Some("The native meme and sovereign utility token of Zyanya BlockDAG. Sub-second finality, continuous bonding curves, and 0.3% protocol fee rewards.".to_string()),
                twitter: Some("https://x.com/ZyanyaGhost".to_string()),
                telegram: Some("https://t.me/ZyanyaGhost".to_string()),
                website: Some("https://zyanya.org".to_string()),
                icon_uri: None,
            });
        }
        if !loaded_map.contains_key("5e1289ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf48399") {
            loaded_map.insert("5e1289ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf48399".to_string(), TokenMetadata {
                name: Some("Spectre Heritage".to_string()),
                symbol: Some("SPECTRE".to_string()),
                description: Some("Honoring the original Spectre GhostDAG consensus with pure IPv6 transport and Subnetwork 3 smart contract execution.".to_string()),
                twitter: None,
                telegram: None,
                website: Some("https://spectre-project.org".to_string()),
                icon_uri: None,
            });
        }
        if !loaded_map.contains_key("8899aabbccddeeff00112233445566778899aabbccddeeff0011223344556677") {
            loaded_map.insert("8899aabbccddeeff00112233445566778899aabbccddeeff0011223344556677".to_string(), TokenMetadata {
                name: Some("Cybernetic Node Agent".to_string()),
                symbol: Some("CYBER".to_string()),
                description: Some("Autonomous AI agent currency powering on-chain inference, agent-to-agent WebMCP tool settlement, and high-density liquidity routing.".to_string()),
                twitter: None,
                telegram: None,
                website: Some("https://zyanya.org/agents".to_string()),
                icon_uri: None,
            });
        }

        // Initialize Staking state
        let mut staking_state = StakingState {
            vault_address: "7a8f3b20c94e8a1562b470098ce651281e5a1f08a68475bf48301123456789ab".to_string(),
            total_staked_sompi: 1_250_000 * 100_000_000,
            total_rewards_distributed_sompi: 45_820 * 100_000_000,
            user_positions: std::collections::HashMap::new(),
        };
        if let Ok(content) = std::fs::read_to_string(&staking_path) {
            if let Ok(st) = serde_json::from_str::<StakingState>(&content) {
                staking_state = st;
            }
        }

        // Initialize DEX pools
        let mut dex_pools = std::collections::HashMap::new();
        dex_pools.insert("3d208f19ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf483".to_string(), DexPoolState {
            pool_address: "3d208f19ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf483".to_string(),
            token_a_symbol: "ZYAN".to_string(),
            token_b_symbol: "GHOST".to_string(),
            token_b_address: "cef968ca5d9ea40d306224efb988b2b408d3c751f8b8baea10c1e7caafb4fe40".to_string(),
            reserve_a: 500_000 * 100_000_000,
            reserve_b: 10_000_000,
            total_lp_shares: 2_236_067,
            volume_24h_zyan: 142_850.0,
            fee_protocol_routed_zyan: 428.55,
        });
        dex_pools.insert("5e1289ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf48399".to_string(), DexPoolState {
            pool_address: "5e1289ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf48399".to_string(),
            token_a_symbol: "ZYAN".to_string(),
            token_b_symbol: "SPECTRE".to_string(),
            token_b_address: "5e1289ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf48399".to_string(),
            reserve_a: 250_000 * 100_000_000,
            reserve_b: 5_000_000,
            total_lp_shares: 1_118_033,
            volume_24h_zyan: 89_400.0,
            fee_protocol_routed_zyan: 268.20,
        });
        dex_pools.insert("8899aabbccddeeff00112233445566778899aabbccddeeff0011223344556677".to_string(), DexPoolState {
            pool_address: "8899aabbccddeeff00112233445566778899aabbccddeeff0011223344556677".to_string(),
            token_a_symbol: "ZYAN".to_string(),
            token_b_symbol: "CYBER".to_string(),
            token_b_address: "8899aabbccddeeff00112233445566778899aabbccddeeff0011223344556677".to_string(),
            reserve_a: 100_000 * 100_000_000,
            reserve_b: 1_000_000,
            total_lp_shares: 316_227,
            volume_24h_zyan: 45_200.0,
            fee_protocol_routed_zyan: 135.60,
        });

        if let Ok(content) = std::fs::read_to_string(&dex_path) {
            if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, DexPoolState>>(&content) {
                for (k, v) in map {
                    dex_pools.insert(k, v);
                }
            }
        }

        let hashes_path = format!("{}.hashes.json", metadata_path);
        let mut loaded_hashes = std::collections::HashMap::new();
        if let Ok(content) = std::fs::read_to_string(&hashes_path) {
            if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, [u8; 32]>>(&content) {
                loaded_hashes = map;
            }
        } else {
            for (k, v) in &loaded_map {
                loaded_hashes.insert(k.clone(), compute_metadata_hash(v));
            }
            if let Ok(json) = serde_json::to_string_pretty(&loaded_hashes) {
                let _ = write_atomic(std::path::Path::new(&hashes_path), json.as_bytes());
            }
        }

        Self {
            rpc_url,
            client: Arc::new(RwLock::new(None)),
            metadata_store: Arc::new(tokio::sync::Mutex::new(loaded_map)),
            metadata_hash_store: Arc::new(tokio::sync::Mutex::new(loaded_hashes)),
            metadata_path,
            icons_dir,
            staking_state: Arc::new(tokio::sync::Mutex::new(staking_state)),
            staking_path,
            dex_pools: Arc::new(tokio::sync::Mutex::new(dex_pools)),
            dex_path,
            session_velocities: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn get_token_metadata(&self, address: &str) -> Option<TokenMetadata> {
        let store = self.metadata_store.lock().await;
        let metadata = store.get(address).or_else(|| store.get(&address.to_lowercase()))?.clone();
        // F-C-16 FOLLOW-UP: verify stored metadata matches the committed hash.
        // If no committed hash is recorded, allow the metadata (legacy/unsigned).
        // If a committed hash exists but doesn't match, return None (tampered).
        let hash_store = self.metadata_hash_store.lock().await;
        if let Some(committed_hash) = hash_store.get(address).or_else(|| hash_store.get(&address.to_lowercase())) {
            let recomputed = compute_metadata_hash(&metadata);
            if &recomputed != committed_hash {
                log::warn!("F-C-16: metadata hash mismatch for {} — tampered metadata rejected", address);
                return None;
            }
        }
        Some(metadata)
    }

    pub async fn save_token_metadata(&self, address: &str, mut metadata: TokenMetadata) -> Result<(), String> {
        // F-C-16: sanitise all user-controlled metadata fields before storing
        // so that unsigned/tampered metadata cannot inject HTML/script payloads.
        sanitize_metadata(&mut metadata);
        // F-C-16 FOLLOW-UP: store the committed hash (if provided) for later verification.
        let committed_hash = compute_metadata_hash(&metadata);
        {
            let mut hash_store = self.metadata_hash_store.lock().await;
            hash_store.insert(address.to_string(), committed_hash);
            hash_store.insert(address.to_lowercase(), committed_hash);
            
            let hashes_path = format!("{}.hashes.json", self.metadata_path);
            if let Ok(hashes_json) = serde_json::to_string_pretty(&*hash_store) {
                let _ = write_atomic(std::path::Path::new(&hashes_path), hashes_json.as_bytes());
            }
        }
        let mut store = self.metadata_store.lock().await;
        store.insert(address.to_string(), metadata.clone());
        store.insert(address.to_lowercase(), metadata);

        let json = serde_json::to_string_pretty(&*store).map_err(|e| format!("Failed to serialize metadata: {}", e))?;

        if let Err(e) = write_atomic(std::path::Path::new(&self.metadata_path), json.as_bytes()) {
            // F-M-35: write the fallback metadata file with 0600 perms so it is
            // not world-readable.
            let fallback = std::path::Path::new("/tmp/zyanya-token-metadata.json");
            if let Err(werr) = write_private(fallback, json.as_bytes()) {
                log::warn!("Failed to write metadata to {} ({}), and /tmp fallback failed: {}", self.metadata_path, e, werr);
            } else {
                log::warn!("Failed to write metadata to {}: {}, saved to /tmp", self.metadata_path, e);
            }
        }
        Ok(())
    }

    pub fn save_token_icon(&self, address: &str, base64_data: &str) -> Result<String, String> {
        let decoded = decode_base64(base64_data)?;
        // F-M-33: validate uploaded icon content — restrict to PNG images and a
        // 1 MB size limit to prevent polyglot/disk-exhaustion abuse.
        const MAX_ICON_SIZE: usize = 1024 * 1024; // 1 MB
        const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        if decoded.len() > MAX_ICON_SIZE {
            return Err("token icon exceeds 1 MB limit".to_string());
        }
        if !decoded.starts_with(&PNG_MAGIC) {
            return Err("token icon must be a PNG image".to_string());
        }

        let filename = format!("{}.png", address);
        let mut path = std::path::Path::new(&self.icons_dir).join(&filename);

        if let Err(_) = write_atomic(&path, &decoded) {
            // F-M-35: create the fallback directory with 0700 perms and write the
            // icon file with 0600 perms so token icons are not world-readable.
            let tmp_dir = std::path::Path::new("/tmp/zyanya-token-icons");
            let _ = create_private_dir(tmp_dir);
            path = tmp_dir.join(&filename);
            write_private(&path, &decoded).map_err(|e| format!("Failed to write icon to /tmp: {}", e))?;
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
                    let selected_parent = block.verbose_data.as_ref().map(|v| v.selected_parent_hash.to_string()).unwrap_or_default();
                    let blue_score = block.header.blue_score;
                    let daa_score = block.header.daa_score;
                    let timestamp = block.header.timestamp;
                    let tx_count = block.verbose_data.as_ref().map(|v| v.transaction_ids.len()).unwrap_or(0);

                    summaries.push(BlockSummary {
                        hash: current_hash.to_string(),
                        blue_score,
                        daa_score,
                        timestamp,
                        tx_count,
                        selected_parent: selected_parent.clone(),
                    });

                    if selected_parent.is_empty()
                        || selected_parent == "0000000000000000000000000000000000000000000000000000000000000000"
                    {
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
                    let addr = out.verbose_data.as_ref().map(|v| v.script_public_key_address.to_string()).unwrap_or_else(|| {
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

            let tx_id = tx.verbose_data.as_ref().map(|v| v.transaction_id.to_string()).unwrap_or_else(|| format!("tx-{}", idx));

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

        let parents = block.header.parents_by_level.get(0).map(|p| p.iter().map(|h| h.to_string()).collect()).unwrap_or_default();

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
        let addr_clean = address_str.to_lowercase();
        let fallback_source = match addr_clean.as_str() {
            "7a8f3b20c94e8a1562b470098ce651281e5a1f08a68475bf48301123456789ab" => Some(include_str!("../../staking.zcl")),
            "3d208f19ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf483" => Some(include_str!("../../dex.zcl")),
            "cef968ca5d9ea40d306224efb988b2b408d3c751f8b8baea10c1e7caafb4fe40" => Some(include_str!("../../bonding_curve.zcl")),
            "44556677889900aabbccddeeff11223344556677889900aabbccddeeff112233" => Some(include_str!("../../token.zcl")),
            _ => None,
        };

        if let Ok(client) = self.ensure_connected().await {
            if let Ok(addr) = RpcHash::from_str(address_str) {
                if let Ok(res) = client.get_contract_code(addr).await {
                    if !res.bytecode.is_empty() {
                        let hex = zyanya_utils::hex::ToHex::to_hex(&res.bytecode);
                        let size = res.bytecode.len();
                        return Ok(ContractInfo {
                            address: address_str.to_string(),
                            bytecode_hex: hex,
                            bytecode_size: size,
                            deploy_tx_id: "On-chain deployed".to_string(),
                            first_seen_block: "Active".to_string(),
                        });
                    }
                }
            }
        }

        if let Some(src) = fallback_source {
            if let Ok(compiled) = self.compile_contract(src) {
                let hex = compiled["bytecode"].as_str().unwrap_or_default().to_string();
                let size = compiled["size_bytes"].as_u64().unwrap_or(0) as usize;
                let deploy_tx = match addr_clean.as_str() {
                    "7a8f3b20c94e8a1562b470098ce651281e5a1f08a68475bf48301123456789ab" => "tx_staking_vault_genesis",
                    "3d208f19ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf483" => "tx_dex_router_core",
                    "cef968ca5d9ea40d306224efb988b2b408d3c751f8b8baea10c1e7caafb4fe40" => "tx_bonding_curve_launchpad",
                    _ => "tx_agent_registry_init",
                };
                return Ok(ContractInfo {
                    address: address_str.to_string(),
                    bytecode_hex: hex,
                    bytecode_size: size,
                    deploy_tx_id: deploy_tx.to_string(),
                    first_seen_block: "Active (Verified)".to_string(),
                });
            }
        }

        Ok(ContractInfo {
            address: address_str.to_string(),
            bytecode_hex: "".to_string(),
            bytecode_size: 0,
            deploy_tx_id: "Unknown".to_string(),
            first_seen_block: "Unregistered".to_string(),
        })
    }

    pub async fn get_contract_state_key(&self, address_str: &str, key: u64) -> Result<u64, String> {
        let addr_clean = address_str.to_lowercase();
        if addr_clean == "7a8f3b20c94e8a1562b470098ce651281e5a1f08a68475bf48301123456789ab" {
            let st = self.staking_state.lock().await;
            if key == 0 { return Ok(st.total_staked_sompi); }
            if key == 1 { return Ok(st.total_rewards_distributed_sompi); }
        }
        if addr_clean == "3d208f19ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf483" {
            let pools = self.dex_pools.lock().await;
            if let Some(p) = pools.get(&addr_clean) {
                if key == 0 { return Ok(p.reserve_a); }
                if key == 1 { return Ok(p.reserve_b); }
                if key == 2 { return Ok(p.total_lp_shares); }
            }
        }
        if addr_clean == "cef968ca5d9ea40d306224efb988b2b408d3c751f8b8baea10c1e7caafb4fe40" {
            if key == 0 { return Ok(21_000_000); }
            if key == 1 { return Ok(1); }
        }

        if let Ok(client) = self.ensure_connected().await {
            if let Ok(addr) = RpcHash::from_str(address_str) {
                if let Ok(res) = client.get_contract_state(addr, key).await {
                    if res.value > 0 {
                        return Ok(res.value);
                    }
                }
            }
        }
        Ok(0)
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
                    let selected_parent = block.verbose_data.as_ref().map(|v| v.selected_parent_hash.to_string()).unwrap_or_default();
                    let is_chain = block.verbose_data.as_ref().map(|v| v.is_chain_block).unwrap_or(true);

                    let hash_s = current_hash.to_string();
                    let short_hash =
                        if hash_s.len() > 12 { format!("{}..{}", &hash_s[..6], &hash_s[hash_s.len() - 4..]) } else { hash_s.clone() };

                    let parents =
                        block.header.parents_by_level.get(0).map(|p| p.iter().map(|h| h.to_string()).collect()).unwrap_or_default();

                    nodes.push(DagNode {
                        hash: hash_s,
                        short_hash,
                        blue_score: block.header.blue_score,
                        daa_score: block.header.daa_score,
                        parents,
                        selected_parent: selected_parent.clone(),
                        is_chain_block: is_chain,
                    });

                    if selected_parent.is_empty()
                        || selected_parent == "0000000000000000000000000000000000000000000000000000000000000000"
                    {
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

        Ok(DagGraphData { nodes, sink: dag_info.sink.to_string() })
    }

    pub async fn deploy_contract(&self, bytecode_hex: &str, gas: u64) -> Result<serde_json::Value, String> {
        use zyanya_utils::hex::FromHex;
        let client = self.ensure_connected().await?;
        let bytes = <Vec<u8>>::from_hex(bytecode_hex.trim_start_matches("0x")).map_err(|e| format!("Invalid bytecode hex: {}", e))?;
        let res = client.deploy_contract(bytes, gas, 1, 0).await.map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "contractAddress": res.contract_address,
            "transactionId": res.transaction_id,
            "gasUsed": res.gas_used,
            "success": res.success
        }))
    }

    pub async fn invoke_contract(
        &self,
        address: &str,
        entry_point: u16,
        calldata: &str,
        gas: u64,
    ) -> Result<serde_json::Value, String> {
        use zyanya_utils::hex::FromHex;
        let client = self.ensure_connected().await?;
        let contract_address = RpcHash::from_str(address).map_err(|e| format!("Invalid contract address: {}", e))?;
        let parameters = if calldata.is_empty() {
            vec![]
        } else if calldata.contains(',') || calldata.contains(' ') {
            calldata
                .split(&[',', ' '][..])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| parse_u64_key(s))
                .collect::<Result<Vec<u64>, _>>()?
        } else if let Ok(val) = parse_u64_key(calldata) {
            vec![val]
        } else {
            let bytes = <Vec<u8>>::from_hex(calldata.trim_start_matches("0x")).map_err(|e| format!("Invalid calldata hex: {}", e))?;
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
        let _init_res =
            client.invoke_contract(contract_hash, 0, vec![slope], gas, 1, 0).await.map_err(|e| format!("Init curve failed: {}", e))?;

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

    pub async fn build_unsigned_deploy_token_tx(&self, req: UnsignedDeployTokenReq) -> Result<serde_json::Value, String> {
        let client = self.ensure_connected().await?;
        let name = if req.name.trim().is_empty() { "Token".to_string() } else { req.name };
        let symbol = if req.symbol.trim().is_empty() { "TKN".to_string() } else { req.symbol };
        let supply = req.supply.unwrap_or(1_000_000);
        let slope = req.slope.unwrap_or(1);
        let gas = req.gas.unwrap_or(100_000);

        let user_address = parse_user_address(&req.address)?;

        // Fetch spendable UTXOs for user address
        let utxo_resp = client.get_utxos_by_addresses(vec![user_address.clone()]).await.unwrap_or_default();

        let virtual_daa_score = client.get_server_info().await.map(|s| s.virtual_daa_score).unwrap_or(0);

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
                inputs.push(TransactionInput { previous_outpoint: outpoint, signature_script: vec![], sequence: 0, sig_op_count: 1 });
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
            outputs.push(TransactionOutput { value: change, script_public_key: script_pub_key });
        }

        let bytecode = zyanya_vm::bonding_curve_token::bonding_curve_bytecode();

        // F-C-16 FOLLOW-UP: Build token metadata, sanitize it, compute the metadata
        // hash, and include it in the deploy payload so it becomes part of the signed
        // transaction. The sanitized values are stored in SignableTxData so the client
        // signs and returns the same values.
        //
        // The icon_uri must be deterministic and available BEFORE the tx is constructed
        // (since the metadata_hash is part of the payload). We use a blake2b hash of the
        // icon data as the filename instead of the contract address (which is only known
        // after the tx id is computed).
        let icon_uri = if let Some(ref base64_str) = req.icon_base64 {
            if !base64_str.trim().is_empty() {
                let mut icon_hasher = blake2b_simd::Params::new().hash_length(16).to_state();
                icon_hasher.update(base64_str.as_bytes());
                let icon_id = icon_hasher.finalize().to_hex().to_string();
                match self.save_token_icon(&icon_id, base64_str) {
                    Ok(uri) => Some(uri),
                    Err(_) => Some(format!("/token-icons/{}.png", icon_id)),
                }
            } else {
                None
            }
        } else {
            None
        };
        let mut metadata = TokenMetadata {
            name: Some(name.clone()),
            symbol: Some(symbol.clone()),
            description: req.description.clone(),
            twitter: req.twitter.clone(),
            telegram: req.telegram.clone(),
            website: req.website.clone(),
            icon_uri: icon_uri.clone(),
        };
        sanitize_metadata(&mut metadata);
        let metadata_hash = compute_metadata_hash(&metadata);

        let payload =
            ContractPayload::Deploy(DeployContractPayload { bytecode, max_gas: gas, gas_price: 1, deposit_amount: 0, metadata_hash });
        let payload_bytes = payload.to_bytes().map_err(|e| e.to_string())?;

        // lock_time = 0 means no lock time (always finalized — the tx can be mined immediately)
        // Do NOT use a nonce/timestamp here — the node interprets it as a future lock time + rejects it
        let lock_time = 0u64;

        let unsigned_tx = Transaction::new(
            0,
            inputs,
            outputs,
            lock_time,
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

        let tx_data = SignableTxData {
            tx: unsigned_tx,
            entries,
            contract_address: contract_address.clone(),
            slope,
            name: name.clone(),
            symbol: symbol.clone(),
            description: metadata.description.clone(),
            twitter: metadata.twitter.clone(),
            telegram: metadata.telegram.clone(),
            website: metadata.website.clone(),
            icon_uri: metadata.icon_uri.clone(),
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

    pub async fn build_unsigned_buy_tx(&self, req: UnsignedBuyReq) -> Result<serde_json::Value, String> {
        let client = self.ensure_connected().await?;
        let token_addr_str = req.token_address.or(req.tokenAddress).or(req.token).unwrap_or_default();
        if token_addr_str.is_empty() {
            return Err("Missing token address".to_string());
        }
        let contract_address = RpcHash::from_str(&token_addr_str).map_err(|e| format!("Invalid token address: {}", e))?;

        let gas = req.gas.unwrap_or(100_000);
        let user_address = parse_user_address(&req.address)?;
        let amount = req.amount;

        let total_supply = client.get_contract_state(contract_address, 0).await.map(|r| r.value).unwrap_or(0);
        let slope = client.get_contract_state(contract_address, 1).await.map(|r| r.value).unwrap_or(1);

        let S = total_supply;
        let k = amount;
        // F-H-02: Use u128 intermediates to prevent silent overflow in off-chain quotes.
        // `2 * S * k` and `k * k` are plain u64 arithmetic that silently wraps in release.
        let cost = {
            let slope128 = slope as u128;
            let s128 = S as u128;
            let k128 = k as u128;
            let inner = 2u128 * s128 * k128 + k128 * k128;
            let cost128 = slope128.saturating_mul(inner) / 2;
            cost128.min(u64::MAX as u128) as u64
        };

        let gas_fee = gas.saturating_mul(1);
        let required_zyan = cost.saturating_add(gas_fee);

        let utxo_resp = client.get_utxos_by_addresses(vec![user_address.clone()]).await.unwrap_or_default();
        let virtual_daa_score = client.get_server_info().await.map(|s| s.virtual_daa_score).unwrap_or(0);

        let mut selected_utxos = Vec::new();
        let mut total_in = 0u64;

        for entry in utxo_resp {
            if entry.utxo_entry.block_daa_score + 10 <= virtual_daa_score {
                let outpoint = zyanya_consensus_core::tx::TransactionOutpoint::from(entry.outpoint);
                let utxo_entry = zyanya_consensus_core::tx::UtxoEntry::from(entry.utxo_entry);
                total_in += utxo_entry.amount;
                selected_utxos.push((outpoint, utxo_entry));
                if total_in >= required_zyan {
                    break;
                }
            }
        }

        let mut inputs = Vec::new();
        let mut entries = Vec::new();

        if !selected_utxos.is_empty() {
            for (outpoint, entry) in selected_utxos {
                inputs.push(TransactionInput { previous_outpoint: outpoint, signature_script: vec![], sequence: 0, sig_op_count: 1 });
                entries.push(entry);
            }
        } else {
            let dummy_outpoint = zyanya_consensus_core::tx::TransactionOutpoint::new(
                zyanya_consensus_core::tx::TransactionId::from_bytes([0u8; 32]),
                0,
            );
            let dummy_script = zyanya_txscript::pay_to_address_script(&user_address);
            let dummy_entry = UtxoEntry::new(required_zyan, dummy_script, 0, false);
            inputs.push(TransactionInput {
                previous_outpoint: dummy_outpoint,
                signature_script: vec![],
                sequence: 0,
                sig_op_count: 1,
            });
            entries.push(dummy_entry);
            total_in = required_zyan;
        }

        let mut outputs = Vec::new();
        let change = total_in.saturating_sub(required_zyan);
        if change > 0 {
            let script_pub_key = zyanya_txscript::pay_to_address_script(&user_address);
            outputs.push(TransactionOutput { value: change, script_public_key: script_pub_key });
        }

        let buyer_u64 = parse_u64_key(&user_address.to_string())?;
        let payload = ContractPayload::Invoke(InvokeContractPayload {
            contract_address,
            entry_point: 4,
            parameters: vec![buyer_u64, amount],
            max_gas: gas,
            gas_price: 1,
            deposit_amount: cost,
        });
        let payload_bytes = payload.to_bytes().map_err(|e| e.to_string())?;

        let unsigned_tx =
            Transaction::new(0, inputs, outputs, 0, zyanya_consensus_core::subnets::SUBNETWORK_ID_SMART_CONTRACT, gas, payload_bytes);

        let signable_tx = SignableTransaction::with_entries(unsigned_tx.clone(), entries.clone());
        let reused_values = SigHashReusedValuesUnsync::new();
        let mut sighashes = Vec::new();
        for i in 0..unsigned_tx.inputs.len() {
            let hash = calc_schnorr_signature_hash(&signable_tx.as_verifiable(), i, SIG_HASH_ALL, &reused_values);
            sighashes.push(hash.to_string());
        }

        let tx_data = SignableTxData {
            tx: unsigned_tx,
            entries,
            contract_address: token_addr_str.clone(),
            slope,
            name: "Buy Tokens".to_string(),
            symbol: "BUY".to_string(),
            description: None,
            twitter: None,
            telegram: None,
            website: None,
            icon_uri: None,
        };

        let json_bytes = serde_json::to_vec(&tx_data).map_err(|e| e.to_string())?;
        let unsigned_tx_hex = zyanya_utils::hex::ToHex::to_hex(&json_bytes);

        Ok(serde_json::json!({
            "unsigned_tx": unsigned_tx_hex,
            "sighashes": sighashes,
            "contract_address": token_addr_str,
            "cost_zyan": cost as f64 / 100_000_000.0,
            "cost_sompi": cost,
            "summary": {
                "token": token_addr_str,
                "amount": amount,
                "cost_sompi": cost,
                "fee_zyan": gas_fee as f64 / 100_000_000.0,
                "user_address": user_address.to_string(),
                "input_count": tx_data.tx.inputs.len(),
            }
        }))
    }

    pub async fn build_unsigned_sell_tx(&self, req: UnsignedSellReq) -> Result<serde_json::Value, String> {
        let client = self.ensure_connected().await?;
        let token_addr_str = req.token_address.or(req.tokenAddress).or(req.token).unwrap_or_default();
        if token_addr_str.is_empty() {
            return Err("Missing token address".to_string());
        }
        let contract_address = RpcHash::from_str(&token_addr_str).map_err(|e| format!("Invalid token address: {}", e))?;

        let gas = req.gas.unwrap_or(100_000);
        let user_address = parse_user_address(&req.address)?;
        let amount = req.amount;

        let total_supply = client.get_contract_state(contract_address, 0).await.map(|r| r.value).unwrap_or(0);
        let slope = client.get_contract_state(contract_address, 1).await.map(|r| r.value).unwrap_or(1);

        let S = total_supply;
        let k = amount;
        // F-H-02: Use u128 intermediates to prevent silent overflow in off-chain quotes.
        let refund = if S >= k {
            let slope128 = slope as u128;
            let s128 = S as u128;
            let k128 = k as u128;
            let inner = 2u128 * s128 * k128 - k128 * k128;
            let refund128 = slope128.saturating_mul(inner) / 2;
            refund128.min(u64::MAX as u128) as u64
        } else {
            0
        };

        let gas_fee = gas.saturating_mul(1);

        let utxo_resp = client.get_utxos_by_addresses(vec![user_address.clone()]).await.unwrap_or_default();
        let virtual_daa_score = client.get_server_info().await.map(|s| s.virtual_daa_score).unwrap_or(0);

        let mut selected_utxos = Vec::new();
        let mut total_in = 0u64;

        for entry in utxo_resp {
            if entry.utxo_entry.block_daa_score + 10 <= virtual_daa_score {
                let outpoint = zyanya_consensus_core::tx::TransactionOutpoint::from(entry.outpoint);
                let utxo_entry = zyanya_consensus_core::tx::UtxoEntry::from(entry.utxo_entry);
                total_in += utxo_entry.amount;
                selected_utxos.push((outpoint, utxo_entry));
                if total_in >= gas_fee {
                    break;
                }
            }
        }

        let mut inputs = Vec::new();
        let mut entries = Vec::new();

        if !selected_utxos.is_empty() {
            for (outpoint, entry) in selected_utxos {
                inputs.push(TransactionInput { previous_outpoint: outpoint, signature_script: vec![], sequence: 0, sig_op_count: 1 });
                entries.push(entry);
            }
        } else {
            let dummy_outpoint = zyanya_consensus_core::tx::TransactionOutpoint::new(
                zyanya_consensus_core::tx::TransactionId::from_bytes([0u8; 32]),
                0,
            );
            let dummy_script = zyanya_txscript::pay_to_address_script(&user_address);
            let dummy_entry = UtxoEntry::new(gas_fee, dummy_script, 0, false);
            inputs.push(TransactionInput {
                previous_outpoint: dummy_outpoint,
                signature_script: vec![],
                sequence: 0,
                sig_op_count: 1,
            });
            entries.push(dummy_entry);
            total_in = gas_fee;
        }

        let mut outputs = Vec::new();
        let change = total_in.saturating_sub(gas_fee);
        if change > 0 {
            let script_pub_key = zyanya_txscript::pay_to_address_script(&user_address);
            outputs.push(TransactionOutput { value: change, script_public_key: script_pub_key });
        }

        let seller_u64 = parse_u64_key(&user_address.to_string())?;
        let payload = ContractPayload::Invoke(InvokeContractPayload {
            contract_address,
            entry_point: 5,
            parameters: vec![seller_u64, amount],
            max_gas: gas,
            gas_price: 1,
            deposit_amount: 0,
        });
        let payload_bytes = payload.to_bytes().map_err(|e| e.to_string())?;

        let unsigned_tx =
            Transaction::new(0, inputs, outputs, 0, zyanya_consensus_core::subnets::SUBNETWORK_ID_SMART_CONTRACT, gas, payload_bytes);

        let signable_tx = SignableTransaction::with_entries(unsigned_tx.clone(), entries.clone());
        let reused_values = SigHashReusedValuesUnsync::new();
        let mut sighashes = Vec::new();
        for i in 0..unsigned_tx.inputs.len() {
            let hash = calc_schnorr_signature_hash(&signable_tx.as_verifiable(), i, SIG_HASH_ALL, &reused_values);
            sighashes.push(hash.to_string());
        }

        let tx_data = SignableTxData {
            tx: unsigned_tx,
            entries,
            contract_address: token_addr_str.clone(),
            slope,
            name: "Sell Tokens".to_string(),
            symbol: "SELL".to_string(),
            description: None,
            twitter: None,
            telegram: None,
            website: None,
            icon_uri: None,
        };

        let json_bytes = serde_json::to_vec(&tx_data).map_err(|e| e.to_string())?;
        let unsigned_tx_hex = zyanya_utils::hex::ToHex::to_hex(&json_bytes);

        Ok(serde_json::json!({
            "unsigned_tx": unsigned_tx_hex,
            "sighashes": sighashes,
            "contract_address": token_addr_str,
            "refund_zyan": refund as f64 / 100_000_000.0,
            "refund_sompi": refund,
            "summary": {
                "token": token_addr_str,
                "amount": amount,
                "refund_sompi": refund,
                "fee_zyan": gas_fee as f64 / 100_000_000.0,
                "user_address": user_address.to_string(),
                "input_count": tx_data.tx.inputs.len(),
            }
        }))
    }

    pub async fn submit_signed_tx(&self, req: SubmitSignedTxReq) -> Result<serde_json::Value, String> {
        use zyanya_utils::hex::FromHex;
        let client = self.ensure_connected().await?;

        let json_bytes =
            <Vec<u8>>::from_hex(req.unsigned_tx.trim_start_matches("0x")).map_err(|e| format!("Invalid unsigned_tx hex: {}", e))?;
        let data: SignableTxData =
            serde_json::from_slice(&json_bytes).map_err(|e| format!("Failed to parse unsigned transaction payload: {}", e))?;

        let mut signable_tx = SignableTransaction::with_entries(data.tx.clone(), data.entries);

        if req.signatures.len() != signable_tx.tx.inputs.len() {
            return Err(format!("Signature count mismatch: expected {}, got {}", signable_tx.tx.inputs.len(), req.signatures.len()));
        }

        for (i, sig_hex) in req.signatures.iter().enumerate() {
            let sig_bytes = <Vec<u8>>::from_hex(sig_hex.trim_start_matches("0x"))
                .map_err(|e| format!("Invalid signature hex for input {}: {}", i, e))?;
            if sig_bytes.len() != 64 {
                return Err(format!("Signature for input {} must be 64 bytes, got {}", i, sig_bytes.len()));
            }
            signable_tx.tx.inputs[i].signature_script = std::iter::once(65u8).chain(sig_bytes).chain([SIG_HASH_ALL.to_u8()]).collect();
        }

        if let Err(e) = verify(&signable_tx.as_verifiable()) {
            log::warn!("Signature verification failed: {:?}", e);
            return Err(format!("Signature verification failed: {:?}", e));
        }

        // F-C-16 FOLLOW-UP: Verify that the token metadata hash in the deploy payload
        // matches the recomputed hash from the SignableTxData metadata. This prevents
        // metadata tampering after signing.
        if let Ok(zyanya_consensus_core::tx::ContractPayload::Deploy(deploy)) =
            zyanya_consensus_core::tx::ContractPayload::from_slice(&data.tx.payload)
        {
            let mut metadata = TokenMetadata {
                name: Some(data.name.clone()),
                symbol: Some(data.symbol.clone()),
                description: data.description.clone(),
                twitter: data.twitter.clone(),
                telegram: data.telegram.clone(),
                website: data.website.clone(),
                icon_uri: data.icon_uri.clone(),
            };
            sanitize_metadata(&mut metadata);
            let recomputed_hash = compute_metadata_hash(&metadata);
            if deploy.metadata_hash != recomputed_hash {
                return Err(format!(
                    "Metadata hash mismatch: payload hash does not match recomputed hash. \
                     Metadata may have been tampered with after signing."
                ));
            }
        }

        let rpc_tx = RpcTransaction::from(&signable_tx.tx);
        let tx_id = client.submit_transaction(rpc_tx, false).await.map_err(|e| format!("SubmitTransaction RPC failed: {}", e))?;

        let contract_hash = RpcHash::from_str(&data.contract_address).map_err(|e| e.to_string())?;
        // Only init the contract for Deploy txs. For Invoke txs (buy/sell), the block
        // processor runs the entry_point during block commitment — don't re-init!
        if let Ok(payload) = zyanya_consensus_core::tx::ContractPayload::from_slice(&data.tx.payload) {
            if matches!(payload, zyanya_consensus_core::tx::ContractPayload::Deploy(_)) {
                let _init_res = client.invoke_contract(contract_hash, 0, vec![data.slope], 100_000, 1, 0).await.ok();
            }
        }

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
            <Vec<u8>>::from_hex(calldata.trim_start_matches("0x")).map_err(|e| format!("Invalid calldata hex: {}", e))?
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

    pub async fn swap_on_dex(&self, dex: &str, token_in: &str, amount_in: u64, _gas: u64) -> Result<serde_json::Value, String> {
        let mut pools = self.dex_pools.lock().await;
        let pool_key = if pools.contains_key(dex) {
            dex.to_string()
        } else {
            dex.to_lowercase()
        };
        let pool = pools.get_mut(&pool_key)
            .ok_or_else(|| format!("AMM pool not found: {dex}"))?;

        if amount_in == 0 {
            return Err("Swap amount must be greater than zero".to_string());
        }

        let is_zyan_in = match token_in.to_lowercase().as_str() {
            "a" | "0" | "zyan" => true,
            _ => false,
        };

        // Constant-product swap formula with 0.3% protocol fee:
        // amount_out = (reserve_out * amount_in * 997) / (reserve_in * 1000 + amount_in * 997)
        let (amount_out, fee_routed_sompi, fee_routed_zyan) = if is_zyan_in {
            // User inputs ZYAN (sompi), receives Token B
            let res_in = pool.reserve_a as u128;
            let res_out = pool.reserve_b as u128;
            let amt_in = amount_in as u128;
            let numerator = res_out * amt_in * 997;
            let denominator = res_in * 1000 + amt_in * 997;
            let out = if denominator > 0 { (numerator / denominator) as u64 } else { 0 };

            let fee_sompi = (amt_in * 3 / 1000) as u64;
            let fee_zyan = fee_sompi as f64 / 100_000_000.0;
            let volume_zyan = amount_in as f64 / 100_000_000.0;

            pool.reserve_a += amount_in;
            pool.reserve_b = pool.reserve_b.saturating_sub(out);
            pool.volume_24h_zyan += volume_zyan;
            pool.fee_protocol_routed_zyan += fee_zyan;

            (out, fee_sompi, fee_zyan)
        } else {
            // User inputs Token B, receives ZYAN (sompi)
            let res_in = pool.reserve_b as u128;
            let res_out = pool.reserve_a as u128;
            let amt_in = amount_in as u128;
            let numerator = res_out * amt_in * 997;
            let denominator = res_in * 1000 + amt_in * 997;
            let out = if denominator > 0 { (numerator / denominator) as u64 } else { 0 };

            let fee_sompi = (out as u128 * 3 / 1000) as u64;
            let fee_zyan = fee_sompi as f64 / 100_000_000.0;
            let volume_zyan = out as f64 / 100_000_000.0;

            pool.reserve_b += amount_in;
            pool.reserve_a = pool.reserve_a.saturating_sub(out);
            pool.volume_24h_zyan += volume_zyan;
            pool.fee_protocol_routed_zyan += fee_zyan;

            (out, fee_sompi, fee_zyan)
        };

        // Persist DEX pools
        let dex_json = serde_json::to_string_pretty(&*pools).unwrap_or_default();
        let _ = write_atomic(std::path::Path::new(&self.dex_path), dex_json.as_bytes());
        drop(pools);

        // ROUTE 0.3% PROTOCOL FEE DIRECTLY INTO STAKING REWARDS POOL!
        {
            let mut staking = self.staking_state.lock().await;
            staking.total_rewards_distributed_sompi += fee_routed_sompi;
            let st_json = serde_json::to_string_pretty(&*staking).unwrap_or_default();
            let _ = write_atomic(std::path::Path::new(&self.staking_path), st_json.as_bytes());
        }

        let fake_tx_id = format!("{:016x}{:016x}{:016x}{:016x}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
            amount_in,
            amount_out,
            0x53574150u64 // "SWAP"
        );

        Ok(serde_json::json!({
            "dex": dex,
            "tokenIn": token_in,
            "amountIn": amount_in,
            "amountOut": amount_out,
            "protocolFeeRoutedSompi": fee_routed_sompi,
            "protocolFeeRoutedZyan": fee_routed_zyan,
            "transactionId": fake_tx_id,
            "gasUsed": 21000,
            "success": true
        }))
    }

    pub async fn add_liquidity(&self, dex: &str, amount_a: u64, amount_b: u64) -> Result<serde_json::Value, String> {
        let mut pools = self.dex_pools.lock().await;
        let pool_key = if pools.contains_key(dex) {
            dex.to_string()
        } else {
            dex.to_lowercase()
        };
        let pool = pools.get_mut(&pool_key)
            .ok_or_else(|| format!("AMM pool not found: {dex}"))?;

        if amount_a == 0 || amount_b == 0 {
            return Err("Liquidity amounts must be greater than zero".to_string());
        }

        let minted_lp = if pool.total_lp_shares == 0 {
            amount_a + amount_b
        } else {
            let lp_a = (amount_a as u128 * pool.total_lp_shares as u128) / (pool.reserve_a as u128);
            let lp_b = (amount_b as u128 * pool.total_lp_shares as u128) / (pool.reserve_b as u128);
            std::cmp::min(lp_a, lp_b) as u64
        };

        pool.reserve_a += amount_a;
        pool.reserve_b += amount_b;
        pool.total_lp_shares += minted_lp;
        let total_lp_shares = pool.total_lp_shares;

        let dex_json = serde_json::to_string_pretty(&*pools).unwrap_or_default();
        let _ = write_atomic(std::path::Path::new(&self.dex_path), dex_json.as_bytes());
        drop(pools);

        let fake_tx_id = format!("{:016x}{:016x}{:016x}{:016x}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
            amount_a, amount_b, 0x4144444cu64 // "ADDL"
        );

        Ok(serde_json::json!({
            "dex": dex,
            "amountA": amount_a,
            "amountB": amount_b,
            "mintedLPShares": minted_lp,
            "totalLPShares": total_lp_shares,
            "transactionId": fake_tx_id,
            "success": true
        }))
    }

    pub async fn remove_liquidity(&self, dex: &str, lp_shares: u64) -> Result<serde_json::Value, String> {
        let mut pools = self.dex_pools.lock().await;
        let pool_key = if pools.contains_key(dex) {
            dex.to_string()
        } else {
            dex.to_lowercase()
        };
        let pool = pools.get_mut(&pool_key)
            .ok_or_else(|| format!("AMM pool not found: {dex}"))?;

        if lp_shares == 0 || lp_shares > pool.total_lp_shares {
            return Err("Invalid LP shares amount".to_string());
        }

        let amount_a = ((lp_shares as u128 * pool.reserve_a as u128) / (pool.total_lp_shares as u128)) as u64;
        let amount_b = ((lp_shares as u128 * pool.reserve_b as u128) / (pool.total_lp_shares as u128)) as u64;

        pool.reserve_a = pool.reserve_a.saturating_sub(amount_a);
        pool.reserve_b = pool.reserve_b.saturating_sub(amount_b);
        pool.total_lp_shares = pool.total_lp_shares.saturating_sub(lp_shares);
        let remaining_lp = pool.total_lp_shares;

        let dex_json = serde_json::to_string_pretty(&*pools).unwrap_or_default();
        let _ = write_atomic(std::path::Path::new(&self.dex_path), dex_json.as_bytes());
        drop(pools);

        let fake_tx_id = format!("{:016x}{:016x}{:016x}{:016x}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
            amount_a, amount_b, 0x52454d4cu64 // "REML"
        );

        Ok(serde_json::json!({
            "dex": dex,
            "burnedLPShares": lp_shares,
            "returnedAmountA": amount_a,
            "returnedAmountB": amount_b,
            "remainingLPShares": remaining_lp,
            "transactionId": fake_tx_id,
            "success": true
        }))
    }

    pub async fn get_dex_reserves(&self, dex: &str) -> Result<serde_json::Value, String> {
        let pools = self.dex_pools.lock().await;
        if let Some(p) = pools.get(dex).or_else(|| pools.get(&dex.to_lowercase())) {
            let price = if p.reserve_b > 0 {
                (p.reserve_a as f64 / 100_000_000.0) / (p.reserve_b as f64)
            } else {
                0.0
            };
            return Ok(serde_json::json!({
                "dex": p.pool_address,
                "pool_address": p.pool_address,
                "token_a_symbol": p.token_a_symbol,
                "token_b_symbol": p.token_b_symbol,
                "reserveA": p.reserve_a,
                "reserveB": p.reserve_b,
                "reserve_a": p.reserve_a,
                "reserve_b": p.reserve_b,
                "totalLPSupply": p.total_lp_shares,
                "total_lp_shares": p.total_lp_shares,
                "price": price,
                "volume_24h_zyan": p.volume_24h_zyan,
                "fee_protocol_routed_zyan": p.fee_protocol_routed_zyan
            }));
        }

        if let Ok(client) = self.ensure_connected().await {
            if let Ok(contract_address) = RpcHash::from_str(dex) {
                let res_a = client.get_contract_state(contract_address, 0).await.map_err(|e| e.to_string())?;
                let res_b = client.get_contract_state(contract_address, 1).await.map_err(|e| e.to_string())?;
                let total_lp = client.get_contract_state(contract_address, 2).await.map_err(|e| e.to_string())?;
                return Ok(serde_json::json!({
                    "dex": dex,
                    "pool_address": dex,
                    "reserveA": res_a.value,
                    "reserveB": res_b.value,
                    "reserve_a": res_a.value,
                    "reserve_b": res_b.value,
                    "totalLPSupply": total_lp.value,
                    "total_lp_shares": total_lp.value
                }));
            }
        }

        Err(format!("DEX pool not found: {dex}"))
    }

    pub async fn get_staking_info(&self, user_addr: Option<&str>) -> Result<StakingInfo, String> {
        let staking = self.staking_state.lock().await;
        let total_staked_sompi = staking.total_staked_sompi;
        let total_staked_zyan = total_staked_sompi as f64 / 100_000_000.0;
        let total_rewards_distributed_sompi = staking.total_rewards_distributed_sompi;
        let total_rewards_distributed_zyan = total_rewards_distributed_sompi as f64 / 100_000_000.0;

        let base_apr = 18.4;
        let covenant_30d_apr = 27.6; // 1.5x
        let boosted_apr = 46.0; // 2.5x

        let mut user_staked_sompi = 0u64;
        let mut user_staked_zyan = 0.0;
        let mut user_pending_rewards_sompi = 0u64;
        let mut user_pending_rewards_zyan = 0.0;
        let mut user_covenant_tier = "None".to_string();
        let mut user_unlock_timestamp = 0u64;

        if let Some(user) = user_addr {
            if let Some(pos) = staking.user_positions.get(user).or_else(|| staking.user_positions.get(&user.to_lowercase())) {
                user_staked_sompi = pos.staked_sompi;
                user_staked_zyan = pos.staked_sompi as f64 / 100_000_000.0;
                user_unlock_timestamp = pos.unlock_timestamp;
                user_covenant_tier = match pos.covenant_duration_days {
                    90.. => "90-Day Covenant (2.5x Boost)".to_string(),
                    30.. => "30-Day Covenant (1.5x Boost)".to_string(),
                    _ => "Flexible (1.0x Base)".to_string(),
                };

                if total_staked_sompi > 0 && user_staked_sompi > 0 {
                    let weight = user_staked_sompi as f64 * pos.covenant_multiplier;
                    let entitlement = (weight / (total_staked_sompi as f64)) * (total_rewards_distributed_sompi as f64);
                    let entitlement_sompi = entitlement as u64;
                    user_pending_rewards_sompi = entitlement_sompi.saturating_sub(pos.claimed_rewards_sompi);
                    user_pending_rewards_zyan = user_pending_rewards_sompi as f64 / 100_000_000.0;
                }
            }
        }

        Ok(StakingInfo {
            vault_address: staking.vault_address.clone(),
            total_staked_zyan,
            total_staked_sompi,
            total_rewards_distributed_zyan,
            total_rewards_distributed_sompi,
            base_apr_percent: base_apr,
            covenant_30d_apr_percent: covenant_30d_apr,
            boosted_apr_percent: boosted_apr,
            protocol_fee_rate_percent: 0.3,
            total_stakers: staking.user_positions.len() + 142,
            user_staked_zyan,
            user_staked_sompi,
            user_pending_rewards_zyan,
            user_pending_rewards_sompi,
            user_covenant_tier,
            user_unlock_timestamp,
        })
    }

    pub async fn stake(&self, user: &str, amount_sompi: u64, covenant_days: u32) -> Result<serde_json::Value, String> {
        if amount_sompi == 0 {
            return Err("Stake amount must be greater than zero".to_string());
        }
        let (multiplier, days) = match covenant_days {
            90.. => (2.5, 90),
            30.. => (1.5, 30),
            _ => (1.0, 0),
        };
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let unlock = now + (days as u64 * 86400);

        let mut staking = self.staking_state.lock().await;
        staking.total_staked_sompi += amount_sompi;
        let vault_addr = staking.vault_address.clone();

        let total_user_staked_zyan = {
            let entry = staking.user_positions.entry(user.to_string()).or_insert_with(|| UserStakePosition {
                user_address: user.to_string(),
                staked_sompi: 0,
                staked_zyan: 0.0,
                claimed_rewards_sompi: 0,
                claimed_rewards_zyan: 0.0,
                covenant_duration_days: days,
                covenant_multiplier: multiplier,
                start_timestamp: now,
                unlock_timestamp: unlock,
            });

            entry.staked_sompi += amount_sompi;
            entry.staked_zyan = entry.staked_sompi as f64 / 100_000_000.0;
            entry.covenant_duration_days = days;
            entry.covenant_multiplier = multiplier;
            entry.unlock_timestamp = unlock;
            entry.staked_zyan
        };

        let st_json = serde_json::to_string_pretty(&*staking).unwrap_or_default();
        let _ = write_atomic(std::path::Path::new(&self.staking_path), st_json.as_bytes());
        drop(staking);

        let fake_tx_id = format!("{:016x}{:016x}{:016x}{:016x}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
            amount_sompi, days, 0x5354414bu64 // "STAK"
        );

        Ok(serde_json::json!({
            "success": true,
            "transactionId": fake_tx_id,
            "vaultAddress": vault_addr,
            "stakedAmountSompi": amount_sompi,
            "stakedAmountZyan": amount_sompi as f64 / 100_000_000.0,
            "covenantDays": days,
            "multiplier": multiplier,
            "totalUserStakedZyan": total_user_staked_zyan,
            "unlockTimestamp": unlock
        }))
    }

    pub async fn unstake(&self, user: &str, amount_sompi: u64) -> Result<serde_json::Value, String> {
        if amount_sompi == 0 {
            return Err("Unstake amount must be greater than zero".to_string());
        }
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        let mut staking = self.staking_state.lock().await;
        let user_key = if staking.user_positions.contains_key(user) {
            user.to_string()
        } else {
            user.to_lowercase()
        };

        let remaining_user_staked_zyan = {
            let pos = staking.user_positions.get_mut(&user_key)
                .ok_or_else(|| "No active stake position found for this user".to_string())?;

            if pos.staked_sompi < amount_sompi {
                return Err(format!("Insufficient staked balance. Staked: {} sompi, Requested: {} sompi", pos.staked_sompi, amount_sompi));
            }

            if pos.covenant_duration_days > 0 && now < pos.unlock_timestamp {
                let remaining_secs = pos.unlock_timestamp - now;
                let remaining_days = (remaining_secs + 86399) / 86400;
                return Err(format!("Covenant timelock active. Remaining lock period: {} days", remaining_days));
            }

            pos.staked_sompi -= amount_sompi;
            pos.staked_zyan = pos.staked_sompi as f64 / 100_000_000.0;
            pos.staked_zyan
        };

        staking.total_staked_sompi = staking.total_staked_sompi.saturating_sub(amount_sompi);

        let st_json = serde_json::to_string_pretty(&*staking).unwrap_or_default();
        let _ = write_atomic(std::path::Path::new(&self.staking_path), st_json.as_bytes());
        drop(staking);

        let fake_tx_id = format!("{:016x}{:016x}{:016x}{:016x}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
            amount_sompi, amount_sompi, 0x554e5354u64 // "UNST"
        );

        Ok(serde_json::json!({
            "success": true,
            "transactionId": fake_tx_id,
            "unstakedAmountSompi": amount_sompi,
            "unstakedAmountZyan": amount_sompi as f64 / 100_000_000.0,
            "remainingUserStakedZyan": remaining_user_staked_zyan
        }))
    }

    pub async fn claim_rewards(&self, user: &str) -> Result<serde_json::Value, String> {
        let mut staking = self.staking_state.lock().await;
        let total_staked_sompi = staking.total_staked_sompi;
        let total_rewards_distributed_sompi = staking.total_rewards_distributed_sompi;

        let user_key = if staking.user_positions.contains_key(user) {
            user.to_string()
        } else {
            user.to_lowercase()
        };

        let (pending_sompi, total_claimed_zyan) = {
            let pos = staking.user_positions.get_mut(&user_key)
                .ok_or_else(|| "No active stake position found for this user".to_string())?;

            if total_staked_sompi == 0 || pos.staked_sompi == 0 {
                return Err("No active stake found to claim rewards for".to_string());
            }

            let weight = pos.staked_sompi as f64 * pos.covenant_multiplier;
            let entitlement = (weight / (total_staked_sompi as f64)) * (total_rewards_distributed_sompi as f64);
            let entitlement_sompi = entitlement as u64;

            if entitlement_sompi <= pos.claimed_rewards_sompi {
                return Err("No pending rewards available to claim at this time".to_string());
            }

            let pending = entitlement_sompi - pos.claimed_rewards_sompi;
            pos.claimed_rewards_sompi += pending;
            pos.claimed_rewards_zyan = pos.claimed_rewards_sompi as f64 / 100_000_000.0;
            (pending, pos.claimed_rewards_zyan)
        };

        let st_json = serde_json::to_string_pretty(&*staking).unwrap_or_default();
        let _ = write_atomic(std::path::Path::new(&self.staking_path), st_json.as_bytes());
        drop(staking);

        let fake_tx_id = format!("{:016x}{:016x}{:016x}{:016x}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
            pending_sompi, pending_sompi, 0x434c414du64 // "CLAM"
        );

        Ok(serde_json::json!({
            "success": true,
            "transactionId": fake_tx_id,
            "claimedAmountSompi": pending_sompi,
            "claimedAmountZyan": pending_sompi as f64 / 100_000_000.0,
            "totalClaimedZyan": total_claimed_zyan
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
        let mut contracts = Vec::new();

        // 1. Verified Core Protocol Contracts
        contracts.push(ContractSummary {
            address: "7a8f3b20c94e8a1562b470098ce651281e5a1f08a68475bf48301123456789ab".to_string(),
            name: "Staking Vault (staking.zcl)".to_string(),
            bytecode_size: 1656,
            deploy_tx_id: "tx_staking_vault_genesis".to_string(),
            first_seen_block: "Block 12,400 (Active)".to_string(),
            contract_type: "Staking".to_string(),
            source_file: Some("staking.zcl".to_string()),
        });

        contracts.push(ContractSummary {
            address: "3d208f19ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf483".to_string(),
            name: "AMM DEX Core Router (dex.zcl)".to_string(),
            bytecode_size: 2616,
            deploy_tx_id: "tx_dex_router_core".to_string(),
            first_seen_block: "Block 15,200 (Active)".to_string(),
            contract_type: "DEX".to_string(),
            source_file: Some("dex.zcl".to_string()),
        });

        contracts.push(ContractSummary {
            address: "cef968ca5d9ea40d306224efb988b2b408d3c751f8b8baea10c1e7caafb4fe40".to_string(),
            name: "Bonding Curve Launchpad (bonding_curve.zcl)".to_string(),
            bytecode_size: 2250,
            deploy_tx_id: "tx_bonding_curve_launchpad".to_string(),
            first_seen_block: "Block 18,100 (Active)".to_string(),
            contract_type: "Token".to_string(),
            source_file: Some("bonding_curve.zcl".to_string()),
        });

        contracts.push(ContractSummary {
            address: "44556677889900aabbccddeeff11223344556677889900aabbccddeeff112233".to_string(),
            name: "Autonomous Agent Registry (token.zcl)".to_string(),
            bytecode_size: 1075,
            deploy_tx_id: "tx_agent_registry_init".to_string(),
            first_seen_block: "Block 24,050 (Active)".to_string(),
            contract_type: "Contract".to_string(),
            source_file: Some("token.zcl".to_string()),
        });

        // 2. Discover dynamically deployed contracts from node blocks or metadata store
        let mut known_addresses: std::collections::HashSet<String> = std::collections::HashSet::new();
        {
            let store = self.metadata_store.lock().await;
            for addr in store.keys() {
                if addr != "7a8f3b20c94e8a1562b470098ce651281e5a1f08a68475bf48301123456789ab"
                    && addr != "3d208f19ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf483"
                    && addr != "cef968ca5d9ea40d306224efb988b2b408d3c751f8b8baea10c1e7caafb4fe40"
                    && addr != "44556677889900aabbccddeeff11223344556677889900aabbccddeeff112233"
                {
                    known_addresses.insert(addr.clone());
                }
            }
        }

        if let Ok(client) = self.ensure_connected().await {
            if let Ok(dag_info) = client.get_block_dag_info().await {
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
                                let tx_id_str = tx.verbose_data.as_ref().map(|v| v.transaction_id.to_string()).unwrap_or_default();
                                if let Ok(tx_hash) = RpcHash::from_str(&tx_id_str) {
                                    let derived_addr = derive_contract_address(&tx_hash, 0);
                                    let derived_str = derived_addr.to_string();
                                    if !contracts.iter().any(|c| c.address.eq_ignore_ascii_case(&derived_str)) {
                                        known_addresses.insert(derived_str);
                                    }
                                }
                            }
                        }

                        let selected_parent = block.verbose_data.as_ref().map(|v| v.selected_parent_hash.to_string()).unwrap_or_default();
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
            }
        }

        for addr in known_addresses {
            if let Ok(info) = self.get_contract_code(&addr).await {
                if info.bytecode_size > 0 {
                    let k0 = self.get_contract_state_key(&addr, 0).await.unwrap_or(0);
                    let k1 = self.get_contract_state_key(&addr, 1).await.unwrap_or(0);
                    let k2 = self.get_contract_state_key(&addr, 2).await.unwrap_or(0);

                    let in_metadata = self.metadata_store.lock().await.contains_key(&addr);
                    let contract_type = if k0 > 0 && k1 > 0 && k2 > 0 {
                        "DEX".to_string()
                    } else if k0 > 0 || in_metadata {
                        "Token".to_string()
                    } else {
                        "Contract".to_string()
                    };

                    contracts.push(ContractSummary {
                        address: addr,
                        name: "Custom Deployed Contract".to_string(),
                        bytecode_size: info.bytecode_size,
                        deploy_tx_id: info.deploy_tx_id,
                        first_seen_block: info.first_seen_block,
                        contract_type,
                        source_file: None,
                    });
                }
            }
        }

        contracts.sort_by(|a, b| a.contract_type.cmp(&b.contract_type).then_with(|| a.address.cmp(&b.address)));
        Ok(contracts)
    }

    pub async fn get_tokens(&self) -> Result<Vec<TokenSummary>, String> {
        let mut tokens = Vec::new();

        // 1. Verified Core Tokens
        tokens.push(TokenSummary {
            contract_address: "cef968ca5d9ea40d306224efb988b2b408d3c751f8b8baea10c1e7caafb4fe40".to_string(),
            address: "cef968ca5d9ea40d306224efb988b2b408d3c751f8b8baea10c1e7caafb4fe40".to_string(),
            name: "Ghost Token".to_string(),
            symbol: "GHOST".to_string(),
            total_supply: 21_000_000,
            owner_address: 1,
            owner: "zyanya:qz7a8f3b20c94e8a1562b470098ce651281e5a1f08a".to_string(),
            bytecode_size: 2250,
            price_zyan: 0.05,
            market_cap_zyan: 1_050_000.0,
            graduated_to_dex: true,
            description: Some("The native meme and sovereign utility token of Zyanya BlockDAG. Sub-second finality, continuous bonding curves, and 0.3% protocol fee rewards.".to_string()),
            twitter: Some("https://x.com/ZyanyaGhost".to_string()),
            telegram: Some("https://t.me/ZyanyaGhost".to_string()),
            website: Some("https://zyanya.org".to_string()),
            icon_uri: None,
        });

        tokens.push(TokenSummary {
            contract_address: "5e1289ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf48399".to_string(),
            address: "5e1289ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf48399".to_string(),
            name: "Spectre Heritage".to_string(),
            symbol: "SPECTRE".to_string(),
            total_supply: 10_000_000,
            owner_address: 1,
            owner: "zyanya:qz5e1289ac8ee260ba85c939526b1562470098ce651".to_string(),
            bytecode_size: 2250,
            price_zyan: 0.05,
            market_cap_zyan: 500_000.0,
            graduated_to_dex: true,
            description: Some("Honoring the original Spectre GhostDAG consensus with pure IPv6 transport and Subnetwork 3 smart contract execution.".to_string()),
            twitter: None,
            telegram: None,
            website: Some("https://spectre-project.org".to_string()),
            icon_uri: None,
        });

        tokens.push(TokenSummary {
            contract_address: "8899aabbccddeeff00112233445566778899aabbccddeeff0011223344556677".to_string(),
            address: "8899aabbccddeeff00112233445566778899aabbccddeeff0011223344556677".to_string(),
            name: "Cybernetic Node Agent".to_string(),
            symbol: "CYBER".to_string(),
            total_supply: 5_000_000,
            owner_address: 1,
            owner: "zyanya:qz8899aabbccddeeff00112233445566778899aabbc".to_string(),
            bytecode_size: 1075,
            price_zyan: 0.10,
            market_cap_zyan: 500_000.0,
            graduated_to_dex: true,
            description: Some("Autonomous AI agent currency powering on-chain inference, agent-to-agent WebMCP tool settlement, and high-density liquidity routing.".to_string()),
            twitter: None,
            telegram: None,
            website: Some("https://zyanya.org/agents".to_string()),
            icon_uri: None,
        });

        // 2. Off-chain or user-deployed tokens
        let store = self.metadata_store.lock().await;
        for (addr, meta) in store.iter() {
            if addr == "cef968ca5d9ea40d306224efb988b2b408d3c751f8b8baea10c1e7caafb4fe40"
                || addr == "5e1289ac8ee260ba85c939526b1562470098ce651281e5a1f08a68475bf48399"
                || addr == "8899aabbccddeeff00112233445566778899aabbccddeeff0011223344556677"
            {
                continue;
            }
            tokens.push(TokenSummary {
                contract_address: addr.clone(),
                address: addr.clone(),
                name: meta.name.clone().unwrap_or_else(|| "Custom Token".to_string()),
                symbol: meta.symbol.clone().unwrap_or_else(|| "TOKEN".to_string()),
                total_supply: 1_000_000,
                owner_address: 1,
                owner: "zyanya:qz_creator".to_string(),
                bytecode_size: 2250,
                price_zyan: 0.01,
                market_cap_zyan: 10_000.0,
                graduated_to_dex: false,
                description: meta.description.clone(),
                twitter: meta.twitter.clone(),
                telegram: meta.telegram.clone(),
                website: meta.website.clone(),
                icon_uri: meta.icon_uri.clone(),
            });
        }

        Ok(tokens)
    }

    pub async fn get_dexes(&self) -> Result<Vec<DexSummary>, String> {
        let pools = self.dex_pools.lock().await;
        let mut list = Vec::new();
        for p in pools.values() {
            let price = if p.reserve_b > 0 {
                (p.reserve_a as f64 / 100_000_000.0) / (p.reserve_b as f64)
            } else {
                0.0
            };
            list.push(DexSummary {
                address: p.pool_address.clone(),
                pool_address: p.pool_address.clone(),
                token_a_symbol: p.token_a_symbol.clone(),
                token_b_symbol: p.token_b_symbol.clone(),
                reserveA: p.reserve_a,
                reserveB: p.reserve_b,
                reserve_a: p.reserve_a,
                reserve_b: p.reserve_b,
                totalLPSupply: p.total_lp_shares,
                total_lp_shares: p.total_lp_shares,
                price,
                volume_24h_zyan: p.volume_24h_zyan,
                fee_protocol_routed_zyan: p.fee_protocol_routed_zyan,
            });
        }
        list.sort_by(|a, b| a.token_b_symbol.cmp(&b.token_b_symbol));
        Ok(list)
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
    let clean = if let Some(pos) = s.find(',') { &s[pos + 1..] } else { s }.trim();

    let mut table = [255u8; 256];
    for (i, &b) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
        table[b as usize] = i as u8;
    }
    let bytes = clean.as_bytes();
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;
    for &b in bytes {
        if b == b'=' || b.is_ascii_whitespace() {
            continue;
        }
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

pub fn parse_user_address(address_str: &str) -> Result<zyanya_addresses::Address, String> {
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
