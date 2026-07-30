use crate::error::VMError;

/// Maximum number of memory registers available to a contract execution context.
pub const MAX_MEMORY_REGISTERS: usize = 256;

/// Local variable memory registers for VM execution.
#[derive(Debug, Clone)]
pub struct Memory {
    registers: Vec<u64>,
}

impl Memory {
    /// Create a new memory storage instance.
    pub fn new() -> Self {
        Self {
            registers: vec![0; MAX_MEMORY_REGISTERS],
        }
    }

    /// Read value from a register index.
    pub fn load(&self, index: usize) -> Result<u64, VMError> {
        if index >= self.registers.len() {
            return Err(VMError::InvalidMemoryIndex(index));
        }
        Ok(self.registers[index])
    }

    /// Write value into a register index.
    pub fn store(&mut self, index: usize, value: u64) -> Result<(), VMError> {
        if index >= self.registers.len() {
            return Err(VMError::InvalidMemoryIndex(index));
        }
        self.registers[index] = value;
        Ok(())
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
