use crate::assembler::{Assembler, AssemblerError};
use crate::compiler::ast::*;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CodegenError {
    #[error("Undefined variable '{0}'")]
    UndefinedVariable(String),

    #[error("Invalid arguments for builtin '{0}'")]
    InvalidBuiltinArgs(String),

    #[error("Assembler error: {0}")]
    AssemblerError(#[from] AssemblerError),
}

pub struct CodeGenerator {
    lines: Vec<String>,
    next_label_id: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            next_label_id: 0,
        }
    }

    pub fn generate_assembly(&mut self, program: &Program) -> Result<String, CodegenError> {
        self.lines.clear();

        // 1. Dispatch Table Header
        self.lines.push("// --- Zyanya Contract Language Dispatch ---".into());
        for (i, f) in program.functions.iter().enumerate() {
            let fn_label = format!(":fn_{}_{}", i, f.name);
            self.lines.push("DUP".into());
            self.lines.push(format!("PUSH {}", i));
            self.lines.push("EQ".into());
            self.lines.push(format!("JUMPIF {}", fn_label));
        }
        self.lines.push("PUSH 0".into());
        self.lines.push("RETURN".into());
        self.lines.push("".into());

        // 2. Generate Functions
        for (i, f) in program.functions.iter().enumerate() {
            self.generate_function(i, f)?;
        }

        Ok(self.lines.join("\n"))
    }

    pub fn compile_to_bytecode(&mut self, program: &Program) -> Result<Vec<u8>, CodegenError> {
        let asm = self.generate_assembly(program)?;
        let bytecode = Assembler::assemble(&asm)?;
        Ok(bytecode)
    }

    fn generate_function(&mut self, fn_idx: usize, f: &FunctionDef) -> Result<(), CodegenError> {
        let fn_label = format!(":fn_{}_{}", fn_idx, f.name);
        self.lines.push(format!("// --- Function: {} (Entry Point {}) ---", f.name, fn_idx));
        self.lines.push(fn_label);
        self.lines.push("POP".into()); // Pop entry_point ID

        let mut symbols: HashMap<String, usize> = HashMap::new();
        let mut next_reg = 0usize;

        // Map parameters to registers
        for p in &f.params {
            symbols.insert(p.clone(), next_reg);
            self.lines.push(format!("STORE {}", next_reg));
            next_reg += 1;
        }

        let mut returns = false;
        for stmt in &f.body {
            if self.generate_statement(stmt, &mut symbols, &mut next_reg, f, fn_idx)? {
                returns = true;
            }
        }

        if !returns {
            self.lines.push("PUSH 0".into());
            self.lines.push("RETURN".into());
        }

        self.lines.push("".into());
        Ok(())
    }

    fn generate_statement(
        &mut self,
        stmt: &Statement,
        symbols: &mut HashMap<String, usize>,
        next_reg: &mut usize,
        current_fn: &FunctionDef,
        fn_idx: usize,
    ) -> Result<bool, CodegenError> {
        match stmt {
            Statement::Let { name, initializer } => {
                self.generate_expression(initializer, symbols, current_fn, fn_idx)?;
                let reg = *next_reg;
                *next_reg += 1;
                symbols.insert(name.clone(), reg);
                self.lines.push(format!("STORE {}", reg));
                Ok(false)
            }
            Statement::Assign { name, value } => {
                self.generate_expression(value, symbols, current_fn, fn_idx)?;
                let &reg = symbols
                    .get(name)
                    .ok_or_else(|| CodegenError::UndefinedVariable(name.clone()))?;
                self.lines.push(format!("STORE {}", reg));
                Ok(false)
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let label_id = self.next_label_id;
                self.next_label_id += 1;

                let then_label = format!(":if_then_{}", label_id);
                let end_label = format!(":if_end_{}", label_id);

                self.generate_expression(condition, symbols, current_fn, fn_idx)?;
                self.lines.push(format!("JUMPIF {}", then_label));

                if let Some(else_stmts) = else_branch {
                    for s in else_stmts {
                        self.generate_statement(s, symbols, next_reg, current_fn, fn_idx)?;
                    }
                    self.lines.push(format!("JUMP {}", end_label));
                } else {
                    self.lines.push(format!("JUMP {}", end_label));
                }

                self.lines.push(then_label);
                for s in then_branch {
                    self.generate_statement(s, symbols, next_reg, current_fn, fn_idx)?;
                }
                self.lines.push(end_label);

                Ok(false)
            }
            Statement::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    self.generate_expression(expr, symbols, current_fn, fn_idx)?;
                } else {
                    self.lines.push("PUSH 0".into());
                }
                self.lines.push("RETURN".into());
                Ok(true)
            }
            Statement::Expr(expr) => {
                self.generate_expression(expr, symbols, current_fn, fn_idx)?;
                if let Expression::Call { name, .. } = expr {
                    if name == "sload" {
                        self.lines.push("POP".into());
                    }
                }
                Ok(false)
            }
        }
    }

    fn generate_expression(
        &mut self,
        expr: &Expression,
        symbols: &HashMap<String, usize>,
        current_fn: &FunctionDef,
        fn_idx: usize,
    ) -> Result<(), CodegenError> {
        match expr {
            Expression::Number(n) => {
                self.lines.push(format!("PUSH {}", n));
            }
            Expression::Variable(name) => {
                let &reg = symbols
                    .get(name)
                    .ok_or_else(|| CodegenError::UndefinedVariable(name.clone()))?;
                self.lines.push(format!("LOAD {}", reg));
            }
            Expression::Binary { op, left, right } => {
                self.generate_expression(left, symbols, current_fn, fn_idx)?;
                self.generate_expression(right, symbols, current_fn, fn_idx)?;
                match op {
                    BinaryOp::Add => self.lines.push("ADD".into()),
                    BinaryOp::Sub => self.lines.push("SUB".into()),
                    BinaryOp::Mul => self.lines.push("MUL".into()),
                    BinaryOp::Div => self.lines.push("DIV".into()),
                    BinaryOp::Mod => self.lines.push("MOD".into()),
                    BinaryOp::Eq => self.lines.push("EQ".into()),
                    BinaryOp::NotEq => {
                        self.lines.push("EQ".into());
                        self.lines.push("PUSH 0".into());
                        self.lines.push("EQ".into());
                    }
                    BinaryOp::Lt => self.lines.push("LT".into()),
                    BinaryOp::Gt => self.lines.push("GT".into()),
                    BinaryOp::LtEq => self.lines.push("LTE".into()),
                    BinaryOp::GtEq => self.lines.push("GTE".into()),
                }
            }
            Expression::Call { name, args } => match name.as_str() {
                "sstore" => {
                    if args.len() != 2 {
                        return Err(CodegenError::InvalidBuiltinArgs("sstore requires 2 arguments".into()));
                    }
                    self.generate_expression(&args[0], symbols, current_fn, fn_idx)?;
                    self.generate_expression(&args[1], symbols, current_fn, fn_idx)?;
                    self.lines.push("SSTORE".into());
                }
                "sload" => {
                    if args.len() != 1 {
                        return Err(CodegenError::InvalidBuiltinArgs("sload requires 1 argument".into()));
                    }
                    self.generate_expression(&args[0], symbols, current_fn, fn_idx)?;
                    self.lines.push("SLOAD".into());
                }
                "call" => {
                    if args.len() < 3 {
                        return Err(CodegenError::InvalidBuiltinArgs(
                            "call requires at least 3 arguments (addr, gas, calldata...)".into(),
                        ));
                    }
                    // Format: call(addr, gas, calldata...)
                    // If addr is a variable, push it onto stack first for dynamic call evaluation.
                    let is_dynamic_addr = matches!(&args[0], Expression::Variable(_));
                    if is_dynamic_addr {
                        self.generate_expression(&args[0], symbols, current_fn, fn_idx)?;
                    }

                    self.generate_expression(&args[1], symbols, current_fn, fn_idx)?; // gas

                    let is_multi = args.len() > 3;
                    if !is_multi {
                        self.generate_expression(&args[2], symbols, current_fn, fn_idx)?; // single calldata
                    } else {
                        let num_calldata = args.len() - 2;
                        for arg in &args[2..] {
                            self.generate_expression(arg, symbols, current_fn, fn_idx)?;
                        }
                        self.lines.push(format!("PUSH {}", num_calldata));
                    }

                    let addr_str = match &args[0] {
                        Expression::Number(n) => {
                            let hex = format!("{:x}", n);
                            if hex.len() == 1 || hex.len() == 2 {
                                format!("0x{}", hex.repeat(32))
                            } else {
                                format!("0x{:064x}", n)
                            }
                        }
                        _ => "0x0000000000000000000000000000000000000000000000000000000000000000".into(),
                    };
                    if is_multi {
                        self.lines.push(format!("CALLMULTI {}", addr_str));
                    } else {
                        self.lines.push(format!("CALL {}", addr_str));
                    }
                }
                _ => {
                    // Function call fallback
                    return Err(CodegenError::InvalidBuiltinArgs(format!("Unknown function or builtin '{}'", name)));
                }
            },
        }
        Ok(())
    }
}
