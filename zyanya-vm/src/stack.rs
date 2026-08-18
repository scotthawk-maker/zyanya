use crate::error::VMError;

/// Maximum depth allowed for the operand stack.
pub const MAX_STACK_DEPTH: usize = 1024;

/// Operand stack for storing 64-bit integer values during VM execution.
#[derive(Debug, Clone, Default)]
pub struct Stack {
    data: Vec<u64>,
    max_depth: usize,
}

impl Stack {
    /// Create a new stack with default max depth.
    pub fn new() -> Self {
        Self::with_max_depth(MAX_STACK_DEPTH)
    }

    /// Create a new stack with a specified max depth limit.
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            data: Vec::with_capacity(64),
            max_depth,
        }
    }

    /// Push a value onto the stack.
    pub fn push(&mut self, val: u64) -> Result<(), VMError> {
        if self.data.len() >= self.max_depth {
            return Err(VMError::StackOverflow(self.max_depth));
        }
        self.data.push(val);
        Ok(())
    }

    /// Pop a value from the top of the stack.
    pub fn pop(&mut self) -> Result<u64, VMError> {
        self.data.pop().ok_or(VMError::StackUnderflow)
    }

    /// Duplicate the top value on the stack.
    pub fn dup(&mut self) -> Result<(), VMError> {
        let top = self.peek()?;
        self.push(top)
    }

    /// Swap the top two values on the stack.
    pub fn swap(&mut self) -> Result<(), VMError> {
        let len = self.data.len();
        if len < 2 {
            return Err(VMError::StackUnderflow);
        }
        self.data.swap(len - 1, len - 2);
        Ok(())
    }

    /// Peek at the top value of the stack without popping it.
    pub fn peek(&self) -> Result<u64, VMError> {
        self.data.last().copied().ok_or(VMError::StackUnderflow)
    }

    /// Get current stack length.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if stack is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns a slice of the stack data.
    pub fn as_slice(&self) -> &[u64] {
        &self.data
    }
}
