use crate::{
    error::VMError,
    gas::GasMeter,
    memory::Memory,
    opcode::OpCode,
    stack::Stack,
    state::{NoopStateBackend, StateBackend},
};

/// Result returned upon successful completion of VM execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VMResult {
    /// Optional return value produced by a `RETURN` instruction.
    pub return_value: Option<u64>,
    /// Total gas consumed during execution.
    pub gas_used: u64,
    /// Final state of the operand stack.
    pub stack_dump: Vec<u64>,
}

/// The `zyanya-vm` Virtual Machine execution engine.
#[derive(Debug)]
pub struct VM {
    pub stack: Stack,
    pub memory: Memory,
    pub gas_meter: GasMeter,
    pub pc: usize,
}

impl VM {
    /// Instantiate a new VM execution context with the specified gas limit.
    pub fn new(gas_limit: u64) -> Self {
        Self {
            stack: Stack::new(),
            memory: Memory::new(),
            gas_meter: GasMeter::new(gas_limit),
            pc: 0,
        }
    }

    /// Execute a sequence of opcodes without contract state access (uses default noop backend).
    pub fn execute(&mut self, code: &[OpCode]) -> Result<VMResult, VMError> {
        let mut noop_state = NoopStateBackend;
        self.execute_stateful(code, &[0u8; 32], &mut noop_state)
    }

    /// Execute a sequence of opcodes within this VM instance with persistent contract state access.
    pub fn execute_stateful<S: StateBackend>(
        &mut self,
        code: &[OpCode],
        contract_address: &[u8; 32],
        state: &mut S,
    ) -> Result<VMResult, VMError> {
        let mut return_val: Option<u64> = None;

        while self.pc < code.len() {
            let op = &code[self.pc];

            // 1. Gas deduction
            self.gas_meter.consume(op.base_gas_cost())?;

            // 2. Opcode execution
            match op {
                OpCode::Nop => {
                    self.pc += 1;
                }
                OpCode::Halt => {
                    break;
                }
                OpCode::Push(val) => {
                    self.stack.push(*val)?;
                    self.pc += 1;
                }
                OpCode::Pop => {
                    self.stack.pop()?;
                    self.pc += 1;
                }
                OpCode::Dup => {
                    self.stack.dup()?;
                    self.pc += 1;
                }
                OpCode::Swap => {
                    self.stack.swap()?;
                    self.pc += 1;
                }
                OpCode::Add => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    let res = a.checked_add(b).ok_or(VMError::ArithmeticOverflow)?;
                    self.stack.push(res)?;
                    self.pc += 1;
                }
                OpCode::Sub => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    let res = a.checked_sub(b).ok_or(VMError::ArithmeticOverflow)?;
                    self.stack.push(res)?;
                    self.pc += 1;
                }
                OpCode::Mul => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    let res = a.checked_mul(b).ok_or(VMError::ArithmeticOverflow)?;
                    self.stack.push(res)?;
                    self.pc += 1;
                }
                OpCode::Div => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    if b == 0 {
                        return Err(VMError::DivisionByZero);
                    }
                    self.stack.push(a / b)?;
                    self.pc += 1;
                }
                OpCode::Mod => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    if b == 0 {
                        return Err(VMError::DivisionByZero);
                    }
                    self.stack.push(a % b)?;
                    self.pc += 1;
                }
                OpCode::Pow => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    let extra_gas = 1 + (b as u64 / 32);
                    self.gas_meter.consume(extra_gas)?;
                    self.stack.push(a.wrapping_pow(b as u32))?;
                    self.pc += 1;
                }
                OpCode::And => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(a & b)?;
                    self.pc += 1;
                }
                OpCode::Or => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(a | b)?;
                    self.pc += 1;
                }
                OpCode::Xor => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(a ^ b)?;
                    self.pc += 1;
                }
                OpCode::Not => {
                    let a = self.stack.pop()?;
                    self.stack.push(!a)?;
                    self.pc += 1;
                }
                OpCode::Eq => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    let res = if a == b { 1 } else { 0 };
                    self.stack.push(res)?;
                    self.pc += 1;
                }
                OpCode::Lt => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    let res = if a < b { 1 } else { 0 };
                    self.stack.push(res)?;
                    self.pc += 1;
                }
                OpCode::Gt => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    let res = if a > b { 1 } else { 0 };
                    self.stack.push(res)?;
                    self.pc += 1;
                }
                OpCode::Lte => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    let res = if a <= b { 1 } else { 0 };
                    self.stack.push(res)?;
                    self.pc += 1;
                }
                OpCode::Gte => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    let res = if a >= b { 1 } else { 0 };
                    self.stack.push(res)?;
                    self.pc += 1;
                }
                OpCode::Jump(target) => {
                    if *target >= code.len() {
                        return Err(VMError::InvalidJumpTarget {
                            pc: *target,
                            code_len: code.len(),
                        });
                    }
                    self.pc = *target;
                }
                OpCode::JumpIf(target) => {
                    let cond = self.stack.pop()?;
                    if cond != 0 {
                        if *target >= code.len() {
                            return Err(VMError::InvalidJumpTarget {
                                pc: *target,
                                code_len: code.len(),
                            });
                        }
                        self.pc = *target;
                    } else {
                        self.pc += 1;
                    }
                }
                OpCode::Load(idx) => {
                    let val = self.memory.load(*idx)?;
                    self.stack.push(val)?;
                    self.pc += 1;
                }
                OpCode::Store(idx) => {
                    let val = self.stack.pop()?;
                    self.memory.store(*idx, val)?;
                    self.pc += 1;
                }
                OpCode::SLoad => {
                    let key = self.stack.pop()?;
                    let val = state.sload(contract_address, key)?;
                    self.stack.push(val)?;
                    self.pc += 1;
                }
                OpCode::SStore => {
                    let val = self.stack.pop()?;
                    let key = self.stack.pop()?;
                    state.sstore(contract_address, key, val)?;
                    self.pc += 1;
                }
                OpCode::Call(target_addr) => {
                    let calldata = self.stack.pop()?;
                    let forward_gas = self.stack.pop()?;

                    self.gas_meter.consume(forward_gas)?;

                    let target_bytecode = match state.get_code(target_addr) {
                        Ok(code) => code,
                        Err(_) => {
                            self.stack.push(0)?;
                            self.pc += 1;
                            continue;
                        }
                    };

                    let target_opcodes = match OpCode::deserialize_slice(&target_bytecode) {
                        Ok(ops) => ops,
                        Err(_) => {
                            self.stack.push(0)?;
                            self.pc += 1;
                            continue;
                        }
                    };

                    let mut child_vm = VM::new(forward_gas);
                    if calldata > 0 {
                        let _ = child_vm.stack.push(calldata);
                    }

                    match child_vm.execute_stateful(&target_opcodes, target_addr, state) {
                        Ok(res) => {
                            let unused = child_vm.gas_meter.gas_limit().saturating_sub(child_vm.gas_meter.used_gas());
                            self.gas_meter.refund(unused);
                            self.stack.push(res.return_value.unwrap_or(0))?;
                        }
                        Err(_) => {
                            self.stack.push(0)?;
                        }
                    }
                    self.pc += 1;
                }
                OpCode::Return => {
                    return_val = self.stack.pop().ok();
                    break;
                }
            }
        }

        Ok(VMResult {
            return_value: return_val,
            gas_used: self.gas_meter.used_gas(),
            stack_dump: self.stack.as_slice().to_vec(),
        })
    }
}
