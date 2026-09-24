use crate::error::VMError;
use std::collections::BTreeMap;

/// Trait representing persistent contract state access backend.
pub trait StateBackend {
    /// Read value from contract storage for a given contract address and key.
    fn sload(&self, contract_address: &[u8; 32], key: u64) -> Result<u64, VMError>;
    /// Write value to contract storage for a given contract address and key.
    fn sstore(&mut self, contract_address: &[u8; 32], key: u64, value: u64) -> Result<(), VMError>;
    /// Read compiled bytecode for a given contract address.
    fn get_code(&self, contract_address: &[u8; 32]) -> Result<Vec<u8>, VMError>;
    /// Checkpoint the current state, returning an identifier for rollback.
    fn checkpoint(&mut self) -> usize { 0 }
    /// Rollback the state to a previous checkpoint.
    fn rollback(&mut self, _checkpoint: usize) {}
}

/// A simple in-memory mock implementation of `StateBackend` for testing and standalone execution.
#[derive(Debug, Clone, Default)]
pub struct MockStateBackend {
    storage: BTreeMap<([u8; 32], u64), u64>,
    code: BTreeMap<[u8; 32], Vec<u8>>,
    checkpoints: Vec<(BTreeMap<([u8; 32], u64), u64>, BTreeMap<[u8; 32], Vec<u8>>)>,
}

impl MockStateBackend {
    pub fn new() -> Self {
        Self { storage: BTreeMap::new(), code: BTreeMap::new(), checkpoints: Vec::new() }
    }

    pub fn set_code(&mut self, contract_address: [u8; 32], code: Vec<u8>) {
        self.code.insert(contract_address, code);
    }

    pub fn get(&self, contract_address: &[u8; 32], key: u64) -> u64 {
        self.storage.get(&(*contract_address, key)).copied().unwrap_or(0)
    }

    pub fn set(&mut self, contract_address: &[u8; 32], key: u64, value: u64) {
        self.storage.insert((*contract_address, key), value);
    }
}

impl StateBackend for MockStateBackend {
    fn sload(&self, contract_address: &[u8; 32], key: u64) -> Result<u64, VMError> {
        Ok(self.storage.get(&(*contract_address, key)).copied().unwrap_or(0))
    }

    fn sstore(&mut self, contract_address: &[u8; 32], key: u64, value: u64) -> Result<(), VMError> {
        self.storage.insert((*contract_address, key), value);
        Ok(())
    }

    fn get_code(&self, contract_address: &[u8; 32]) -> Result<Vec<u8>, VMError> {
        self.code.get(contract_address).cloned().ok_or_else(|| VMError::StorageError("Contract code not found".to_string()))
    }

    fn checkpoint(&mut self) -> usize {
        let cp = self.checkpoints.len();
        self.checkpoints.push((self.storage.clone(), self.code.clone()));
        cp
    }

    fn rollback(&mut self, checkpoint: usize) {
        if let Some((storage, code)) = self.checkpoints.get(checkpoint) {
            self.storage = storage.clone();
            self.code = code.clone();
            self.checkpoints.truncate(checkpoint);
        }
    }
}

/// A dummy no-op state backend that returns 0 for loads and ignores stores.
#[derive(Debug, Clone, Default)]
pub struct NoopStateBackend;

impl StateBackend for NoopStateBackend {
    fn sload(&self, _contract_address: &[u8; 32], _key: u64) -> Result<u64, VMError> {
        Ok(0)
    }

    fn sstore(&mut self, _contract_address: &[u8; 32], _key: u64, _value: u64) -> Result<(), VMError> {
        Ok(())
    }

    fn get_code(&self, _contract_address: &[u8; 32]) -> Result<Vec<u8>, VMError> {
        Err(VMError::StorageError("Contract code not found".to_string()))
    }
}
