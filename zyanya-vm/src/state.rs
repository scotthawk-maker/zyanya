use crate::error::VMError;
use std::collections::HashMap;

/// Trait representing persistent contract state access backend.
pub trait StateBackend {
    /// Read value from contract storage for a given contract address and key.
    fn sload(&self, contract_address: &[u8; 32], key: u64) -> Result<u64, VMError>;
    /// Write value to contract storage for a given contract address and key.
    fn sstore(&mut self, contract_address: &[u8; 32], key: u64, value: u64) -> Result<(), VMError>;
    /// Read compiled bytecode for a given contract address.
    fn get_code(&self, contract_address: &[u8; 32]) -> Result<Vec<u8>, VMError>;
    /// Read the ZYAN balance held by a contract at the given address.
    fn get_balance(&self, contract_address: &[u8; 32]) -> Result<u64, VMError>;
    /// Withdraw ZYAN from the contract at `contract_address` to `recipient`.
    /// Returns `Ok(())` if the transfer succeeded, or an error if funds are insufficient.
    fn withdraw(&mut self, contract_address: &[u8; 32], recipient: u64, amount: u64) -> Result<(), VMError>;
}

/// A simple in-memory mock implementation of `StateBackend` for testing and standalone execution.
#[derive(Debug, Clone, Default)]
pub struct MockStateBackend {
    storage: HashMap<([u8; 32], u64), u64>,
    balances: HashMap<[u8; 32], u64>,
    code: HashMap<[u8; 32], Vec<u8>>,
}

impl MockStateBackend {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            code: HashMap::new(),
            balances: HashMap::new(),
        }
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

    /// Set the ZYAN balance for a contract (testing helper).
    pub fn set_balance(&mut self, contract_address: &[u8; 32], balance: u64) {
        self.balances.insert(*contract_address, balance);
    }

    /// Read the ZYAN balance for a contract (testing helper).
    pub fn balance(&self, contract_address: &[u8; 32]) -> u64 {
        self.balances.get(contract_address).copied().unwrap_or(0)
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
        self.code
            .get(contract_address)
            .cloned()
            .ok_or_else(|| VMError::StorageError("Contract code not found".to_string()))
    }

    fn get_balance(&self, contract_address: &[u8; 32]) -> Result<u64, VMError> {
        Ok(self.balances.get(contract_address).copied().unwrap_or(0))
    }

    fn withdraw(&mut self, contract_address: &[u8; 32], _recipient: u64, amount: u64) -> Result<(), VMError> {
        let current = self.balances.get(contract_address).copied().unwrap_or(0);
        let new_balance = current.checked_sub(amount).ok_or_else(|| {
            VMError::StorageError(format!(
                "Insufficient contract balance: have {}, withdraw {}",
                current, amount
            ))
        })?;
        self.balances.insert(*contract_address, new_balance);
        Ok(())
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

    fn get_balance(&self, _contract_address: &[u8; 32]) -> Result<u64, VMError> {
        Ok(0)
    }

    fn withdraw(&mut self, _contract_address: &[u8; 32], _recipient: u64, _amount: u64) -> Result<(), VMError> {
        Ok(())
    }
}
