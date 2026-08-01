use std::collections::BTreeMap;
use std::sync::Arc;

use rocksdb::WriteBatch;
use serde::{Deserialize, Serialize};
use zyanya_consensus_core::tx::{ContractPayload, Transaction, TransactionId};
use zyanya_database::prelude::{BatchDbWriter, CachePolicy, CachedDbAccess, StoreResult, DB};
use zyanya_database::registry::DatabaseStorePrefixes;
use zyanya_hashes::{Hash, HasherBase, TransactionSigningHash};
use zyanya_utils::mem_size::MemSizeEstimator;
use zyanya_vm::{OpCode, StateBackend, VMError, VM};

/// Derive contract address deterministically from deploy tx ID and output index.
pub fn derive_contract_address(deploy_tx_id: &TransactionId, index: u32) -> Hash {
    let mut hasher = TransactionSigningHash::new();
    hasher.update(deploy_tx_id.as_bytes());
    hasher.update(index.to_le_bytes());
    hasher.finalize()
}

/// Composite key for contract storage: contract address (32 bytes) + key (8 bytes).
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ContractStorageKey {
    pub contract_address: [u8; 32],
    pub key: u64,
}

impl ContractStorageKey {
    pub fn new(contract_address: [u8; 32], key: u64) -> Self {
        Self { contract_address, key }
    }
}

impl AsRef<[u8]> for ContractStorageKey {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

impl std::fmt::Display for ContractStorageKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{}", self.contract_address, self.key)
    }
}

impl MemSizeEstimator for ContractStorageKey {}

/// Transactional in-memory state cache for smart contract execution.
///
/// Uses `BTreeMap` (not `HashMap`) for deterministic iteration order — critical so that
/// `commit_cache_batch` writes and any state-root hashing produce identical ordering across
/// all consensus nodes (avoids network forks). See AUDIT.md HIGH-01.
#[derive(Clone, Default)]
pub struct ContractStateCache {
    pub code: BTreeMap<[u8; 32], Vec<u8>>,
    pub storage: BTreeMap<([u8; 32], u64), u64>,
    pub balances: BTreeMap<[u8; 32], u64>,
    pub fallback_storage: Option<Arc<dyn Fn([u8; 32], u64) -> u64 + Send + Sync>>,
    pub fallback_balance: Option<Arc<dyn Fn([u8; 32]) -> u64 + Send + Sync>>,
}

impl std::fmt::Debug for ContractStateCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContractStateCache")
            .field("code", &self.code)
            .field("storage", &self.storage)
            .field("balances", &self.balances)
            .field("has_fallback", &self.fallback_storage.is_some())
            .field("has_fallback_balance", &self.fallback_balance.is_some())
            .finish()
    }
}

impl ContractStateCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_balance(&self, addr_bytes: &[u8; 32]) -> u64 {
        if let Some(bal) = self.balances.get(addr_bytes) {
            return *bal;
        }
        if let Some(ref fallback) = self.fallback_balance {
            return fallback(*addr_bytes);
        }
        0
    }
}

impl StateBackend for ContractStateCache {
    fn sload(&self, contract_address: &[u8; 32], key: u64) -> Result<u64, VMError> {
        if let Some(val) = self.storage.get(&(*contract_address, key)) {
            return Ok(*val);
        }
        if let Some(ref fallback) = self.fallback_storage {
            return Ok(fallback(*contract_address, key));
        }
        Ok(0)
    }

    fn sstore(&mut self, contract_address: &[u8; 32], key: u64, value: u64) -> Result<(), VMError> {
        self.storage.insert((*contract_address, key), value);
        Ok(())
    }

    fn get_code(&self, contract_address: &[u8; 32]) -> Result<Vec<u8>, VMError> {
        self.code
            .get(contract_address)
            .cloned()
            .ok_or_else(|| VMError::StorageError("Contract code not found".to_string()))
    }
}

/// Database store for persistent smart contract code, storage, and balances.
#[derive(Clone)]
pub struct DbContractStore {
    db: Arc<DB>,
    code_access: CachedDbAccess<Hash, Vec<u8>>,
    storage_access: CachedDbAccess<ContractStorageKey, u64>,
    balance_access: CachedDbAccess<Hash, u64>,
}

impl DbContractStore {
    pub fn new(db: Arc<DB>, cache_policy: CachePolicy) -> Self {
        Self {
            db: db.clone(),
            code_access: CachedDbAccess::new(db.clone(), cache_policy, DatabaseStorePrefixes::ContractCode.into()),
            storage_access: CachedDbAccess::new(db.clone(), cache_policy, DatabaseStorePrefixes::ContractStorage.into()),
            balance_access: CachedDbAccess::new(db, cache_policy, DatabaseStorePrefixes::ContractBalance.into()),
        }
    }

    pub fn get_code(&self, contract_address: Hash) -> StoreResult<Vec<u8>> {
        self.code_access.read(contract_address)
    }

    pub fn get_storage(&self, contract_address: [u8; 32], key: u64) -> StoreResult<u64> {
        let storage_key = ContractStorageKey::new(contract_address, key);
        self.storage_access.read(storage_key)
    }

    pub fn get_balance(&self, contract_address: Hash) -> StoreResult<u64> {
        self.balance_access.read(contract_address)
    }

    pub fn commit_cache_batch(&self, batch: &mut WriteBatch, cache: &ContractStateCache) -> StoreResult<()> {
        let mut writer = BatchDbWriter::new(batch);

        for (addr, bytecode) in &cache.code {
            let hash_addr = Hash::from_bytes(*addr);
            self.code_access.write(&mut writer, hash_addr, bytecode.clone())?;
        }

        for ((addr, key), val) in &cache.storage {
            let storage_key = ContractStorageKey::new(*addr, *key);
            self.storage_access.write(&mut writer, storage_key, *val)?;
        }

        for (addr, bal) in &cache.balances {
            let hash_addr = Hash::from_bytes(*addr);
            self.balance_access.write(&mut writer, hash_addr, *bal)?;
        }

        Ok(())
    }
}

/// Result of executing a contract transaction in consensus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractExecutionOutcome {
    pub tx_id: Hash,
    pub contract_address: Hash,
    pub gas_used: u64,
    pub gas_fee: u64,
    pub burned_fee: u64,
    pub miner_fee: u64,
    pub return_value: Option<u64>,
    pub success: bool,
}

/// Processor for executing smart contract transactions in GhostDAG order.
#[derive(Debug, Clone, Default)]
pub struct ContractProcessor;

impl ContractProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Process a smart contract transaction (`Deploy` or `Invoke`), mutating `cache` on success.
    pub fn process_contract_tx(
        &self,
        tx: &Transaction,
        cache: &mut ContractStateCache,
    ) -> Option<ContractExecutionOutcome> {
        if !tx.subnetwork_id.is_smart_contract() {
            return None;
        }

        let payload = match ContractPayload::from_slice(&tx.payload) {
            Ok(p) => p,
            Err(_) => return None,
        };

        match payload {
            ContractPayload::Deploy(deploy) => {
                let contract_address = derive_contract_address(&tx.id(), 0);
                let addr_bytes: [u8; 32] = contract_address.as_bytes().try_into().unwrap();

                // Save contract bytecode in state cache
                cache.code.insert(addr_bytes, deploy.bytecode.clone());

                if deploy.deposit_amount > 0 {
                    let current_balance = cache.get_balance(&addr_bytes);
                    cache.balances.insert(addr_bytes, current_balance.saturating_add(deploy.deposit_amount));
                }

                // Execute constructor/initial bytecode
                let opcodes = match OpCode::deserialize_slice(&deploy.bytecode) {
                    Ok(ops) => ops,
                    Err(_) => {
                        let total_fee = deploy.max_gas.saturating_mul(deploy.gas_price);
                        let burned = total_fee / 2;
                        let miner = total_fee - burned;
                        return Some(ContractExecutionOutcome {
                            tx_id: tx.id(),
                            contract_address,
                            gas_used: deploy.max_gas,
                            gas_fee: total_fee,
                            burned_fee: burned,
                            miner_fee: miner,
                            return_value: None,
                            success: false,
                        });
                    }
                };

                // Save bytecode only — do NOT execute the constructor/init on deploy.
                // The init (setting the slope) is done separately via invoke_contract (entry_point 0).
                // Executing here would run the init with an empty stack → garbage slope.
                let total_fee = deploy.max_gas.saturating_mul(deploy.gas_price);
                let burned = total_fee / 2;
                let miner = total_fee - burned;
                Some(ContractExecutionOutcome {
                    tx_id: tx.id(),
                    contract_address,
                    gas_used: 0,
                    gas_fee: total_fee,
                    burned_fee: burned,
                    miner_fee: miner,
                    return_value: None,
                    success: true,
                })
            }
            ContractPayload::Invoke(invoke) => {
                let contract_address = invoke.contract_address;
                let addr_bytes: [u8; 32] = contract_address.as_bytes().try_into().unwrap();

                // Lookup contract bytecode from cache
                let bytecode = match cache.code.get(&addr_bytes) {
                    Some(b) => b.clone(),
                    None => {
                        let total_fee = invoke.max_gas.saturating_mul(invoke.gas_price);
                        let burned = total_fee / 2;
                        let miner = total_fee - burned;
                        return Some(ContractExecutionOutcome {
                            tx_id: tx.id(),
                            contract_address,
                            gas_used: invoke.max_gas,
                            gas_fee: total_fee,
                            burned_fee: burned,
                            miner_fee: miner,
                            return_value: None,
                            success: false,
                        });
                    }
                };

                if invoke.deposit_amount > 0 {
                    let current_balance = cache.get_balance(&addr_bytes);
                    cache.balances.insert(addr_bytes, current_balance.saturating_add(invoke.deposit_amount));
                }

                let opcodes = match OpCode::deserialize_slice(&bytecode) {
                    Ok(ops) => ops,
                    Err(_) => {
                        let total_fee = invoke.max_gas.saturating_mul(invoke.gas_price);
                        let burned = total_fee / 2;
                        let miner = total_fee - burned;
                        return Some(ContractExecutionOutcome {
                            tx_id: tx.id(),
                            contract_address,
                            gas_used: invoke.max_gas,
                            gas_fee: total_fee,
                            burned_fee: burned,
                            miner_fee: miner,
                            return_value: None,
                            success: false,
                        });
                    }
                };

                let mut vm = VM::new(invoke.max_gas);
                for param in invoke.parameters.iter().rev() {
                    let _ = vm.stack.push(*param);
                }
                let _ = vm.stack.push(invoke.entry_point as u64);
                let mut temp_cache = cache.clone();
                match vm.execute_stateful(&opcodes, &addr_bytes, &mut temp_cache) {
                    Ok(res) => {
                        let total_fee = res.gas_used.saturating_mul(invoke.gas_price);
                        let burned = total_fee / 2;
                        let miner = total_fee - burned;

                        let ret_val = res.return_value.unwrap_or(0);

                        // Real ZYAN Custody enforcement for bonding curve buy (entry point 4) and sell (entry point 5)
                        if invoke.entry_point == 4 {
                            let cost = ret_val;
                            if invoke.deposit_amount > 0 && invoke.deposit_amount < cost {
                                // Buyer deposited insufficient ZYAN to cover cost
                                return Some(ContractExecutionOutcome {
                                    tx_id: tx.id(),
                                    contract_address,
                                    gas_used: invoke.max_gas,
                                    gas_fee: total_fee,
                                    burned_fee: burned,
                                    miner_fee: miner,
                                    return_value: None,
                                    success: false,
                                });
                            }
                        } else if invoke.entry_point == 5 {
                            let refund = ret_val;
                            if refund > 0 {
                                let contract_bal = temp_cache.get_balance(&addr_bytes);
                                if contract_bal < refund {
                                    // Contract has insufficient ZYAN reserve to refund seller
                                    return Some(ContractExecutionOutcome {
                                        tx_id: tx.id(),
                                        contract_address,
                                        gas_used: invoke.max_gas,
                                        gas_fee: total_fee,
                                        burned_fee: burned,
                                        miner_fee: miner,
                                        return_value: None,
                                        success: false,
                                    });
                                }
                                temp_cache.balances.insert(addr_bytes, contract_bal - refund);
                            }
                        }

                        *cache = temp_cache;
                        Some(ContractExecutionOutcome {
                            tx_id: tx.id(),
                            contract_address,
                            gas_used: res.gas_used,
                            gas_fee: total_fee,
                            burned_fee: burned,
                            miner_fee: miner,
                            return_value: res.return_value,
                            success: true,
                        })
                    }
                    Err(_) => {
                        let total_fee = invoke.max_gas.saturating_mul(invoke.gas_price);
                        let burned = total_fee / 2;
                        let miner = total_fee - burned;
                        Some(ContractExecutionOutcome {
                            tx_id: tx.id(),
                            contract_address,
                            gas_used: invoke.max_gas,
                            gas_fee: total_fee,
                            burned_fee: burned,
                            miner_fee: miner,
                            return_value: None,
                            success: false,
                        })
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zyanya_consensus_core::{
        subnets::SUBNETWORK_ID_SMART_CONTRACT,
        tx::{DeployContractPayload, InvokeContractPayload, Transaction},
    };
    use zyanya_database::create_temp_db;

    #[test]
    fn test_smart_contract_end_to_end_integration() {
        // 1. Setup temporary database & stores
        let (_temp_dir, db) = create_temp_db!(zyanya_database::prelude::ConnBuilder::default().with_files_limit(10));
        let store = DbContractStore::new(db.clone(), CachePolicy::Count(100));
        let mut cache = ContractStateCache::new();
        let processor = ContractProcessor::new();

        // Smart contract program:
        // PUSH 100
        // PUSH 200
        // ADD (300)
        // DUP
        // PUSH 42 (Key)
        // SWAP (Stack: [42, 300])
        // SSTORE (Store Key 42 = 300)
        // PUSH 42
        // SLOAD (Load Key 42 -> 300)
        // RETURN (Return 300)
        let contract_opcodes = vec![
            OpCode::Push(100),
            OpCode::Push(200),
            OpCode::Add,
            OpCode::Dup,
            OpCode::Push(42),
            OpCode::Swap,
            OpCode::SStore,
            OpCode::Push(42),
            OpCode::SLoad,
            OpCode::Return,
        ];
        let bytecode = OpCode::serialize_slice(&contract_opcodes);

        // 2. Create DeployContract transaction
        let deploy_payload = ContractPayload::Deploy(DeployContractPayload {
            bytecode: bytecode.clone(),
            max_gas: 10000,
            gas_price: 10,
            deposit_amount: 5000,
        });

        let deploy_tx = Transaction::new(
            1,
            vec![],
            vec![],
            0,
            SUBNETWORK_ID_SMART_CONTRACT,
            10000,
            deploy_payload.to_bytes().unwrap(),
        );

        // 3. Process deploy transaction
        let deploy_outcome = processor
            .process_contract_tx(&deploy_tx, &mut cache)
            .expect("Deploy transaction processing failed");

        assert!(deploy_outcome.success, "Deploy contract execution failed");
        let contract_addr = deploy_outcome.contract_address;
        let addr_bytes: [u8; 32] = contract_addr.as_bytes().try_into().unwrap();

        // Deploy no longer executes the init bytecode (since commit a14be79): it only stores
        // the code + credits the deposit. Init runs separately via an invoke at entry_point 0.
        assert_eq!(deploy_outcome.return_value, None, "Deploy should not execute init");
        assert_eq!(deploy_outcome.gas_used, 0, "Deploy does not consume execution gas");
        assert_eq!(deploy_outcome.gas_fee, 10000 * 10, "Full max_gas fee charged on deploy");
        assert_eq!(deploy_outcome.burned_fee, deploy_outcome.gas_fee / 2);
        assert_eq!(deploy_outcome.miner_fee, deploy_outcome.gas_fee - deploy_outcome.burned_fee);
        assert_eq!(cache.sload(&addr_bytes, 42).unwrap_or(0), 0, "State not set until invoke runs");

        // 4. Create InvokeContract transaction targeting deployed contract
        let invoke_payload = ContractPayload::Invoke(InvokeContractPayload {
            contract_address: contract_addr,
            entry_point: 0,
            parameters: vec![],
            max_gas: 10000,
            gas_price: 10,
            deposit_amount: 1000,
        });

        let invoke_tx = Transaction::new(
            1,
            vec![],
            vec![],
            0,
            SUBNETWORK_ID_SMART_CONTRACT,
            10000,
            invoke_payload.to_bytes().unwrap(),
        );

        // 5. Process invoke transaction
        let invoke_outcome = processor
            .process_contract_tx(&invoke_tx, &mut cache)
            .expect("Invoke transaction processing failed");

        assert!(invoke_outcome.success, "Invoke contract execution failed");
        assert_eq!(invoke_outcome.return_value, Some(300));
        assert_eq!(invoke_outcome.burned_fee, invoke_outcome.gas_fee / 2);
        assert_eq!(invoke_outcome.miner_fee, invoke_outcome.gas_fee - invoke_outcome.burned_fee);

        // 6. Commit cache state to persistent RocksDB store
        let mut batch = WriteBatch::default();
        store.commit_cache_batch(&mut batch, &cache).unwrap();
        db.write(batch).unwrap();

        // 7. Verify persistent database reads
        let stored_code = store.get_code(contract_addr).unwrap();
        assert_eq!(stored_code, bytecode);

        let stored_val = store.get_storage(addr_bytes, 42).unwrap();
        assert_eq!(stored_val, 300, "Persistent RocksDB storage updated");

        let stored_balance = store.get_balance(contract_addr).unwrap();
        assert_eq!(stored_balance, 6000, "Contract account balance updated (5000 + 1000)");
    }

    #[test]
    fn test_token_contract_consensus_integration() {
        let (_temp_dir, db) = create_temp_db!(zyanya_database::prelude::ConnBuilder::default().with_files_limit(10));
        let store = DbContractStore::new(db.clone(), CachePolicy::Count(100));
        let mut cache = ContractStateCache::new();
        let processor = ContractProcessor::new();

        // Build token bytecode with 1,000,000 supply minted to owner 1
        let bytecode = zyanya_vm::token_contract_bytecode(1_000_000, 1).unwrap();

        // 1. Deploy Token
        let deploy_payload = ContractPayload::Deploy(DeployContractPayload {
            bytecode: bytecode.clone(),
            max_gas: 100_000,
            gas_price: 1,
            deposit_amount: 0,
        });
        let deploy_tx = Transaction::new(1, vec![], vec![], 0, SUBNETWORK_ID_SMART_CONTRACT, 100_000, deploy_payload.to_bytes().unwrap());
        let deploy_outcome = processor.process_contract_tx(&deploy_tx, &mut cache).unwrap();
        assert!(deploy_outcome.success, "Token deployment failed");
        let token_addr = deploy_outcome.contract_address;

        // 2. Query Total Supply (entry_point = 2)
        let supply_payload = ContractPayload::Invoke(InvokeContractPayload {
            contract_address: token_addr,
            entry_point: 2,
            parameters: vec![],
            max_gas: 100_000,
            gas_price: 1,
            deposit_amount: 0,
        });
        let supply_tx = Transaction::new(1, vec![], vec![], 0, SUBNETWORK_ID_SMART_CONTRACT, 100_000, supply_payload.to_bytes().unwrap());
        let supply_outcome = processor.process_contract_tx(&supply_tx, &mut cache).unwrap();
        assert_eq!(supply_outcome.return_value, Some(1_000_000));

        // 3. Query Owner Balance (entry_point = 1, holder = 1)
        let owner_bal_payload = ContractPayload::Invoke(InvokeContractPayload {
            contract_address: token_addr,
            entry_point: 1,
            parameters: vec![1],
            max_gas: 100_000,
            gas_price: 1,
            deposit_amount: 0,
        });
        let owner_bal_tx = Transaction::new(1, vec![], vec![], 0, SUBNETWORK_ID_SMART_CONTRACT, 100_000, owner_bal_payload.to_bytes().unwrap());
        let owner_bal_outcome = processor.process_contract_tx(&owner_bal_tx, &mut cache).unwrap();
        assert_eq!(owner_bal_outcome.return_value, Some(1_000_000));

        // 4. Transfer 100 GHOST from owner (1) to recipient (2) (entry_point = 0, params = [1, 2, 100])
        let transfer_payload = ContractPayload::Invoke(InvokeContractPayload {
            contract_address: token_addr,
            entry_point: 0,
            parameters: vec![1, 2, 100],
            max_gas: 100_000,
            gas_price: 1,
            deposit_amount: 0,
        });
        let transfer_tx = Transaction::new(1, vec![], vec![], 0, SUBNETWORK_ID_SMART_CONTRACT, 100_000, transfer_payload.to_bytes().unwrap());
        let transfer_outcome = processor.process_contract_tx(&transfer_tx, &mut cache).unwrap();
        assert!(transfer_outcome.success);
        assert_eq!(transfer_outcome.return_value, Some(1));

        // 5. Query both balances
        let owner_bal_outcome = processor.process_contract_tx(&owner_bal_tx, &mut cache).unwrap();
        assert_eq!(owner_bal_outcome.return_value, Some(999_900));

        let recip_bal_payload = ContractPayload::Invoke(InvokeContractPayload {
            contract_address: token_addr,
            entry_point: 1,
            parameters: vec![2],
            max_gas: 100_000,
            gas_price: 1,
            deposit_amount: 0,
        });
        let recip_bal_tx = Transaction::new(1, vec![], vec![], 0, SUBNETWORK_ID_SMART_CONTRACT, 100_000, recip_bal_payload.to_bytes().unwrap());
        let recip_bal_outcome = processor.process_contract_tx(&recip_bal_tx, &mut cache).unwrap();
        assert_eq!(recip_bal_outcome.return_value, Some(100));

        // 6. Commit to RocksDB store and verify persistent state
        let mut batch = WriteBatch::default();
        store.commit_cache_batch(&mut batch, &cache).unwrap();
        db.write(batch).unwrap();

        let addr_bytes: [u8; 32] = token_addr.as_bytes().try_into().unwrap();
        assert_eq!(store.get_storage(addr_bytes, 0).unwrap(), 1_000_000, "Total supply in RocksDB");
        assert_eq!(store.get_storage(addr_bytes, 1).unwrap(), 999_900, "Owner balance in RocksDB");
        assert_eq!(store.get_storage(addr_bytes, 2).unwrap(), 100, "Recipient balance in RocksDB");
    }
}
