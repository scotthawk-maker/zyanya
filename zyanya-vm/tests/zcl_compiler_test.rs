use zyanya_vm::compiler::{BinaryOp, Expression, Lexer, Parser, Statement, TokenKind};
use zyanya_vm::{Compiler, MockStateBackend, OpCode, VM};

#[test]
fn test_zcl_lexer_comprehensive() {
    let source = r#"
        // Zyanya Contract Language Test
        contract Token {
            state {
                total_supply: u64 = 1000,
            }

            fn transfer(from: u64, to: u64, amount: u64) -> u64 {
                let bal = sload(from);
                if (bal >= amount) {
                    sstore(from, bal - amount);
                    let to_bal = sload(to);
                    sstore(to, to_bal + amount);
                    return 1;
                } else {
                    return 0;
                }
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexing failed");

    let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
    assert!(kinds.contains(&TokenKind::Contract));
    assert!(kinds.contains(&TokenKind::State));
    assert!(kinds.contains(&TokenKind::Fn));
    assert!(kinds.contains(&TokenKind::Let));
    assert!(kinds.contains(&TokenKind::If));
    assert!(kinds.contains(&TokenKind::Else));
    assert!(kinds.contains(&TokenKind::Sload));
    assert!(kinds.contains(&TokenKind::Sstore));
    assert!(kinds.contains(&TokenKind::Return));
    assert!(kinds.contains(&TokenKind::GtEq));
    assert!(kinds.contains(&TokenKind::Minus));
    assert!(kinds.contains(&TokenKind::Plus));
}

#[test]
fn test_zcl_parser_ast_structure() {
    let source = r#"
        fn compute(a, b) {
            let res = (a + b) * 2 - 5;
            if (res > 10) {
                sstore(0, res);
            } else {
                sstore(0, 0);
            }
            return sload(0);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.functions.len(), 1);
    let f = &program.functions[0];
    assert_eq!(f.name, "compute");
    assert_eq!(f.params, vec!["a".to_string(), "b".to_string()]);
    assert_eq!(f.body.len(), 3); // let, if, return

    match &f.body[0] {
        Statement::Let { name, initializer } => {
            assert_eq!(name, "res");
            match initializer {
                Expression::Binary { op, .. } => assert_eq!(*op, BinaryOp::Sub),
                _ => panic!("Expected Binary Sub expression"),
            }
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_zcl_compiler_counter_contract_lifecycle() {
    let zcl_source = r#"
        // Counter Contract in ZCL
        fn init() {
            sstore(0, 100);
            return sload(0);
        }

        fn add_val(delta) {
            let current = sload(0);
            let updated = current + delta;
            sstore(0, updated);
            return updated;
        }

        fn get_val() {
            return sload(0);
        }
    "#;

    let bytecode = Compiler::compile(zcl_source).expect("Compilation failed");
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("Deserialization failed");

    let mut state = MockStateBackend::new();
    let contract_addr = [0x77u8; 32];

    // 1. Run init() -> should set key 0 to 100
    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap(); // entry point 0 (init)
    let res = vm.execute_stateful(&opcodes, &contract_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(100));
    assert_eq!(state.get(&contract_addr, 0), 100);

    // 2. Run add_val(50) -> should update key 0 to 150
    let mut vm = VM::new(100_000);
    vm.stack.push(50).unwrap(); // param delta = 50
    vm.stack.push(1).unwrap();  // entry point 1 (add_val)
    let res = vm.execute_stateful(&opcodes, &contract_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(150));
    assert_eq!(state.get(&contract_addr, 0), 150);

    // 3. Run get_val() -> should return 150
    let mut vm = VM::new(100_000);
    vm.stack.push(2).unwrap();  // entry point 2 (get_val)
    let res = vm.execute_stateful(&opcodes, &contract_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(150));
}

#[test]
fn test_zcl_compiler_token_contract_full_lifecycle() {
    let token_zcl = r#"
        contract Token {
            fn init_supply(owner, supply) {
                sstore(0, supply);
                sstore(owner, supply);
                return supply;
            }

            fn transfer(from, to, amount) {
                let from_bal = sload(from);
                if (from_bal < amount) {
                    return 0;
                }
                sstore(from, from_bal - amount);
                let to_bal = sload(to);
                sstore(to, to_bal + amount);
                return 1;
            }

            fn balance_of(holder) {
                return sload(holder);
            }

            fn total_supply() {
                return sload(0);
            }
        }
    "#;

    let bytecode = Compiler::compile(token_zcl).expect("Token compilation failed");
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("Deserialization failed");

    let mut state = MockStateBackend::new();
    let token_addr = [0x88u8; 32];

    let alice = 100u64;
    let bob = 200u64;
    let initial_supply = 1_000_000u64;

    // 1. Initialize supply: Alice gets 1,000,000 (entry point 0)
    let mut vm = VM::new(100_000);
    vm.stack.push(initial_supply).unwrap();
    vm.stack.push(alice).unwrap();
    vm.stack.push(0).unwrap(); // entry point 0
    let res = vm.execute_stateful(&opcodes, &token_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(initial_supply));
    assert_eq!(state.get(&token_addr, 0), initial_supply);
    assert_eq!(state.get(&token_addr, alice), initial_supply);

    // 2. Transfer 300,000 from Alice to Bob (entry point 1)
    let mut vm = VM::new(100_000);
    vm.stack.push(300_000).unwrap(); // amount
    vm.stack.push(bob).unwrap();     // to
    vm.stack.push(alice).unwrap();   // from
    vm.stack.push(1).unwrap();       // entry point 1
    let res = vm.execute_stateful(&opcodes, &token_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(1)); // success
    assert_eq!(state.get(&token_addr, alice), 700_000);
    assert_eq!(state.get(&token_addr, bob), 300_000);

    // 3. Failed Transfer: Bob tries to send 500,000 to Alice (insufficient balance)
    let mut vm = VM::new(100_000);
    vm.stack.push(500_000).unwrap(); // amount
    vm.stack.push(alice).unwrap();   // to
    vm.stack.push(bob).unwrap();     // from
    vm.stack.push(1).unwrap();       // entry point 1
    let res = vm.execute_stateful(&opcodes, &token_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0)); // failed
    assert_eq!(state.get(&token_addr, bob), 300_000); // balance unchanged

    // 4. Query balance_of(Bob) (entry point 2)
    let mut vm = VM::new(100_000);
    vm.stack.push(bob).unwrap();
    vm.stack.push(2).unwrap();
    let res = vm.execute_stateful(&opcodes, &token_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(300_000));

    // 5. Query total_supply() (entry point 3)
    let mut vm = VM::new(100_000);
    vm.stack.push(3).unwrap();
    let res = vm.execute_stateful(&opcodes, &token_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(initial_supply));
}

#[test]
fn test_zcl_compiler_multifunction_contract_jumpif() {
    let source = r#"
        fn init() {
            sstore(0, 0);
            return 0;
        }
        fn increment(n) {
            let count = sload(0);
            sstore(0, count + n);
            return sload(0);
        }
        fn get() {
            return sload(0);
        }
    "#;

    let bytecode = Compiler::compile(source).expect("Multi-function ZCL compilation failed");

    // Verify JUMPIF targets in compiled bytecode are absolute byte offsets
    // Byte 11 is JUMPIF (0x31). Bytes 12..20 are u64 target (70 = 0x46 for :fn_0_init).
    assert_eq!(bytecode[11], 0x31);
    let target_0 = u64::from_le_bytes(bytecode[12..20].try_into().unwrap());
    assert_eq!(target_0, 70, "JUMPIF target should be byte offset 70");

    let opcodes = OpCode::deserialize_slice(&bytecode).expect("Deserialization failed");
    let mut state = MockStateBackend::new();
    let addr = [0x42u8; 32];

    // 1. Simulate Deployment (entry point 0 pushed)
    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap(); // entry_point = 0 (init)
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Init failed");
    assert_eq!(res.return_value, Some(0));
    assert_eq!(state.get(&addr, 0), 0);

    // 2. Invoke increment(5) (entry point 1)
    let mut vm = VM::new(100_000);
    vm.stack.push(5).unwrap(); // param n = 5
    vm.stack.push(1).unwrap(); // entry_point = 1
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Increment failed");
    assert_eq!(res.return_value, Some(5));
    assert_eq!(state.get(&addr, 0), 5);

    // 3. Call get() (entry point 2)
    let mut vm = VM::new(100_000);
    vm.stack.push(2).unwrap(); // entry_point = 2
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Get failed");
    assert_eq!(res.return_value, Some(5));
}
