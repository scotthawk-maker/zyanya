use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zyanya_addresses::Address;
use zyanya_consensus_core::{
    constants::TX_VERSION,
    subnets::SUBNETWORK_ID_NATIVE,
    tx::{ScriptPublicKey, Transaction, TransactionInput, TransactionOutpoint, TransactionOutput, UtxoEntry},
};
use zyanya_grpc_client::GrpcClient;
use zyanya_rpc_core::{
    api::rpc::RpcApi,
    model::address::RpcAddress,
    RpcHash,
    RpcTransaction,
};
use zyanya_txscript::pay_to_address_script;

use crate::key_management::WalletKeypair;

pub const DEFAULT_FEE_SOMPI: u64 = 1_000; // 1,000 sompi fee
pub const DEFAULT_GAS_LIMIT: u64 = 100_000;
pub const DEFAULT_GAS_PRICE: u64 = 1;

#[derive(Error, Debug)]
pub enum WalletOpsError {
    #[error("Key error: {0}")]
    Key(#[from] crate::key_management::KeyManagementError),
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Insufficient balance: required {required} sompi, available {available} sompi")]
    InsufficientBalance { required: u64, available: u64 },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("General error: {0}")]
    General(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TxKind {
    SendZyan { recipient: String, amount_sompi: u64 },
    SendToken { token_contract: String, to: String, amount: u64 },
    SwapDex { dex_contract: String, token_in: String, amount_in: u64 },
    DeployToken { token_contract: String, supply: u64 },
    DeployDex { dex_contract: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub tx_id: String,
    pub kind: TxKind,
    pub timestamp: u64,
    pub status: String,
}

pub struct WalletOps {
    pub keypair: WalletKeypair,
    pub rpc_url: String,
    pub history: Vec<TransactionRecord>,
}

impl WalletOps {
    pub fn new(keypair: WalletKeypair, rpc_url: String) -> Self {
        let history = Self::load_history(&keypair.address.to_string()).unwrap_or_default();
        Self {
            keypair,
            rpc_url,
            history,
        }
    }

    /// Path to history JSON file
    fn history_path(address_str: &str) -> PathBuf {
        let safe_addr = address_str.replace(':', "_");
        if let Some(home) = dirs::home_dir() {
            home.join(".zyanya").join(format!("history_{}.json", safe_addr))
        } else {
            PathBuf::from(format!("history_{}.json", safe_addr))
        }
    }

    /// Load transaction history
    pub fn load_history(address_str: &str) -> Result<Vec<TransactionRecord>, WalletOpsError> {
        let path = Self::history_path(address_str);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(path)?;
        let records: Vec<TransactionRecord> = serde_json::from_str(&content)?;
        Ok(records)
    }

    /// Save transaction history
    pub fn save_history(&self) -> Result<(), WalletOpsError> {
        let path = Self::history_path(&self.keypair.address.to_string());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.history)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Add a record to history and save
    pub fn record_tx(&mut self, tx_id: String, kind: TxKind, status: String) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let record = TransactionRecord {
            tx_id,
            kind,
            timestamp: now,
            status,
        };
        self.history.insert(0, record);
        let _ = self.save_history();
    }

    /// Connect to node gRPC RPC
    pub async fn connect_rpc(&self) -> Result<GrpcClient, WalletOpsError> {
        let mut url = self.rpc_url.clone();
        if !url.starts_with("grpc://") && !url.starts_with("http://") {
            url = format!("grpc://{}", url);
        }
        GrpcClient::connect(url.clone())
            .await
            .map_err(|e| WalletOpsError::Rpc(format!("Failed to connect to {}: {}", url, e)))
    }

    /// Get ZYAN balance and UTXOs for current address
    pub async fn get_zyan_balance(
        &self,
        client: &GrpcClient,
    ) -> Result<(u64, Vec<(TransactionOutpoint, UtxoEntry)>), WalletOpsError> {
        let rpc_addr = RpcAddress::try_from(self.keypair.address.clone())
            .map_err(|e| WalletOpsError::InvalidAddress(e.to_string()))?;

        match client.get_utxos_by_addresses(vec![rpc_addr.clone()]).await {
            Ok(entries) => {
                let mut total_balance = 0u64;
                let mut utxos = Vec::new();
                for entry in entries {
                    let outpoint = TransactionOutpoint::new(entry.outpoint.transaction_id, entry.outpoint.index);
                    let script_spk = ScriptPublicKey::from(entry.utxo_entry.script_public_key);
                    let core_utxo = UtxoEntry::new(
                        entry.utxo_entry.amount,
                        script_spk,
                        entry.utxo_entry.block_daa_score,
                        entry.utxo_entry.is_coinbase,
                    );
                    total_balance += entry.utxo_entry.amount;
                    utxos.push((outpoint, core_utxo));
                }
                Ok((total_balance, utxos))
            }
            Err(_) => {
                // Fallback to balance query
                let bal = client
                    .get_balance_by_address(rpc_addr)
                    .await
                    .map_err(|e| WalletOpsError::Rpc(e.to_string()))?;
                Ok((bal, Vec::new()))
            }
        }
    }

    /// Send ZYAN to recipient address (creates, signs with user private key, and submits tx)
    pub async fn send_zyan(
        &mut self,
        client: &GrpcClient,
        recipient_str: &str,
        amount_sompi: u64,
    ) -> Result<String, WalletOpsError> {
        let recipient_addr = Address::try_from(recipient_str)
            .map_err(|e| WalletOpsError::InvalidAddress(e.to_string()))?;

        let (total_balance, utxos) = self.get_zyan_balance(client).await?;
        let required = amount_sompi + DEFAULT_FEE_SOMPI;

        if total_balance < required {
            return Err(WalletOpsError::InsufficientBalance {
                required,
                available: total_balance,
            });
        }

        // Select UTXOs
        let mut selected_utxos = Vec::new();
        let mut selected_amount = 0u64;

        for (op, entry) in utxos {
            selected_amount += entry.amount;
            selected_utxos.push((op, entry));
            if selected_amount >= required {
                break;
            }
        }

        if selected_amount < required {
            return Err(WalletOpsError::InsufficientBalance {
                required,
                available: selected_amount,
            });
        }

        let change_amount = selected_amount - required;

        // Build inputs
        let inputs: Vec<TransactionInput> = selected_utxos
            .iter()
            .map(|(op, _)| TransactionInput {
                previous_outpoint: *op,
                signature_script: vec![],
                sequence: 0,
                sig_op_count: 1,
            })
            .collect();

        // Build outputs
        let mut outputs = vec![TransactionOutput {
            value: amount_sompi,
            script_public_key: pay_to_address_script(&recipient_addr),
        }];

        if change_amount > 0 {
            outputs.push(TransactionOutput {
                value: change_amount,
                script_public_key: pay_to_address_script(&self.keypair.address),
            });
        }

        let unsigned_tx = Transaction::new_non_finalized(
            TX_VERSION,
            inputs,
            outputs,
            0,
            SUBNETWORK_ID_NATIVE,
            0,
            vec![],
        );

        let utxo_entries: Vec<UtxoEntry> = selected_utxos.into_iter().map(|(_, e)| e).collect();

        // Sign transaction with user's private key
        let signed_tx = self.keypair.sign_transaction(unsigned_tx, utxo_entries)?;
        let rpc_tx = RpcTransaction::from(&signed_tx);

        let tx_id = client
            .submit_transaction(rpc_tx, false)
            .await
            .map_err(|e| WalletOpsError::Rpc(format!("Submission failed: {}", e)))?;

        let tx_id_str = tx_id.to_string();
        self.record_tx(
            tx_id_str.clone(),
            TxKind::SendZyan {
                recipient: recipient_str.to_string(),
                amount_sompi,
            },
            "Confirmed".to_string(),
        );

        Ok(tx_id_str)
    }

    /// Derive numeric storage key for a holder address (u64)
    pub fn holder_u64(address: &Address) -> u64 {
        if address.payload.len() >= 8 {
            u64::from_le_bytes(address.payload[0..8].try_into().unwrap_or([0u8; 8]))
        } else {
            1
        }
    }

    /// Get token balance for holder in a token contract
    pub async fn get_token_balance(
        &self,
        client: &GrpcClient,
        token_contract_str: &str,
        holder_u64: u64,
    ) -> Result<u64, WalletOpsError> {
        let contract_address = RpcHash::from_str(token_contract_str)
            .map_err(|e| WalletOpsError::General(format!("Invalid contract address: {}", e)))?;

        let res = client
            .get_contract_state(contract_address, holder_u64)
            .await
            .map_err(|e| WalletOpsError::Rpc(e.to_string()))?;

        Ok(res.value)
    }

    /// Send tokens (Entry Point 0: Transfer [from, to, amount])
    pub async fn send_token(
        &mut self,
        client: &GrpcClient,
        token_contract_str: &str,
        from_u64: u64,
        to_u64: u64,
        amount: u64,
    ) -> Result<String, WalletOpsError> {
        let contract_address = RpcHash::from_str(token_contract_str)
            .map_err(|e| WalletOpsError::General(format!("Invalid contract address: {}", e)))?;

        let parameters = vec![from_u64, to_u64, amount];

        let res = client
            .invoke_contract(
                contract_address,
                0, // entry_point 0 = transfer
                parameters,
                DEFAULT_GAS_LIMIT,
                DEFAULT_GAS_PRICE,
                0,
            )
            .await
            .map_err(|e| WalletOpsError::Rpc(format!("Invoke failed: {}", e)))?;

        let tx_id_str = res.transaction_id.to_string();
        self.record_tx(
            tx_id_str.clone(),
            TxKind::SendToken {
                token_contract: token_contract_str.to_string(),
                to: to_u64.to_string(),
                amount,
            },
            if res.success { "Success" } else { "Reverted" }.to_string(),
        );

        Ok(tx_id_str)
    }

    /// Swap tokens on DEX (Entry Point 2: swap [tokenIn, amountIn])
    pub async fn swap_on_dex(
        &mut self,
        client: &GrpcClient,
        dex_contract_str: &str,
        token_in_val: u64,
        amount_in: u64,
    ) -> Result<(String, u64), WalletOpsError> {
        let contract_address = RpcHash::from_str(dex_contract_str)
            .map_err(|e| WalletOpsError::General(format!("Invalid DEX address: {}", e)))?;

        let parameters = vec![token_in_val, amount_in];

        let res = client
            .invoke_contract(
                contract_address,
                2, // entry_point 2 = swap
                parameters,
                DEFAULT_GAS_LIMIT,
                DEFAULT_GAS_PRICE,
                0,
            )
            .await
            .map_err(|e| WalletOpsError::Rpc(format!("Swap failed: {}", e)))?;

        let tx_id_str = res.transaction_id.to_string();
        self.record_tx(
            tx_id_str.clone(),
            TxKind::SwapDex {
                dex_contract: dex_contract_str.to_string(),
                token_in: if token_in_val == 0 { "ZYAN (A)" } else { "GHOST (B)" }.to_string(),
                amount_in,
            },
            if res.success { "Success" } else { "Reverted" }.to_string(),
        );

        Ok((tx_id_str, res.return_value.unwrap_or(0)))
    }

    /// Query DEX reserves (returns reserveA, reserveB)
    pub async fn get_dex_reserves(
        &self,
        client: &GrpcClient,
        dex_contract_str: &str,
    ) -> Result<(u64, u64), WalletOpsError> {
        let contract_address = RpcHash::from_str(dex_contract_str)
            .map_err(|e| WalletOpsError::General(format!("Invalid DEX address: {}", e)))?;

        let res_a = client.get_contract_state(contract_address, 0).await.map_err(|e| WalletOpsError::Rpc(e.to_string()))?;
        let res_b = client.get_contract_state(contract_address, 1).await.map_err(|e| WalletOpsError::Rpc(e.to_string()))?;

        Ok((res_a.value, res_b.value))
    }

    /// Deploy GHOST Token helper
    pub async fn deploy_token(
        &mut self,
        client: &GrpcClient,
        supply: u64,
        owner_u64: u64,
    ) -> Result<String, WalletOpsError> {
        let bytecode = zyanya_vm::token_contract_bytecode(supply, owner_u64)
            .map_err(|e| WalletOpsError::General(e.to_string()))?;

        let res = client
            .deploy_contract(bytecode, DEFAULT_GAS_LIMIT, DEFAULT_GAS_PRICE, 0)
            .await
            .map_err(|e| WalletOpsError::Rpc(e.to_string()))?;

        let contract_addr = res.contract_address.to_string();
        self.record_tx(
            res.transaction_id.to_string(),
            TxKind::DeployToken {
                token_contract: contract_addr.clone(),
                supply,
            },
            "Success".to_string(),
        );

        Ok(contract_addr)
    }

    /// Deploy DEX helper
    pub async fn deploy_dex(
        &mut self,
        client: &GrpcClient,
        dex_source: Option<&str>,
    ) -> Result<String, WalletOpsError> {
        let zcl_code = dex_source.unwrap_or(include_str!("../../dex.zcl"));
        let bytecode = zyanya_vm::Compiler::compile(zcl_code)
            .map_err(|e| WalletOpsError::General(e.to_string()))?;

        let res = client
            .deploy_contract(bytecode, DEFAULT_GAS_LIMIT, DEFAULT_GAS_PRICE, 0)
            .await
            .map_err(|e| WalletOpsError::Rpc(e.to_string()))?;

        let contract_addr = res.contract_address.to_string();
        self.record_tx(
            res.transaction_id.to_string(),
            TxKind::DeployDex {
                dex_contract: contract_addr.clone(),
            },
            "Success".to_string(),
        );

        Ok(contract_addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_management::WalletKeypair;

    #[test]
    fn test_holder_u64_derivation() {
        let keypair = WalletKeypair::generate();
        let holder = WalletOps::holder_u64(&keypair.address);
        assert!(holder > 0);
    }

    #[test]
    fn test_transaction_record_serde() {
        let record = TransactionRecord {
            tx_id: "1234567890abcdef".to_string(),
            kind: TxKind::SendZyan {
                recipient: "zyanyadev:test".to_string(),
                amount_sompi: 50_000_000,
            },
            timestamp: 1700000000,
            status: "Confirmed".to_string(),
        };

        let json = serde_json::to_string(&record).unwrap();
        let restored: TransactionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(record.tx_id, restored.tx_id);
    }
}
