use crate::error::VMError;

/// Gas meter for tracking and enforcing gas consumption during VM execution.
#[derive(Debug, Clone)]
pub struct GasMeter {
    gas_limit: u64,
    used_gas: u64,
}

impl GasMeter {
    /// Create a new gas meter with the given gas limit.
    pub fn new(gas_limit: u64) -> Self {
        Self {
            gas_limit,
            used_gas: 0,
        }
    }

    /// Deduct gas for an operation.
    pub fn consume(&mut self, amount: u64) -> Result<(), VMError> {
        let new_used = self.used_gas.saturating_add(amount);
        if new_used > self.gas_limit {
            return Err(VMError::OutOfGas {
                limit: self.gas_limit,
                requested: new_used,
            });
        }
        self.used_gas = new_used;
        Ok(())
    }

    /// Total gas consumed so far.
    pub fn used_gas(&self) -> u64 {
        self.used_gas
    }

    /// Total gas limit.
    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    /// Remaining gas available.
    pub fn remaining_gas(&self) -> u64 {
        self.gas_limit.saturating_sub(self.used_gas)
    }

    /// Refund gas (e.g. unused forwarded gas from a child VM call).
    pub fn refund(&mut self, amount: u64) {
        self.used_gas = self.used_gas.saturating_sub(amount);
    }
}
