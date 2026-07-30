use std::env;
use std::fs;
use std::process::ExitCode;
use zyanya_utils::hex::ToHex;
use zyanya_vm::Compiler;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: zyanya-compile <source.zcl> [--asm]");
        return ExitCode::FAILURE;
    }

    let source_path = &args[1];
    let show_asm = args.iter().any(|arg| arg == "--asm");

    let source = match fs::read_to_string(source_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", source_path, err);
            return ExitCode::FAILURE;
        }
    };

    if show_asm {
        match Compiler::compile_to_assembly(&source) {
            Ok(asm) => {
                println!("{}", asm);
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("Compilation error: {}", err);
                ExitCode::FAILURE
            }
        }
    } else {
        match Compiler::compile(&source) {
            Ok(bytecode) => {
                let hex_str = bytecode.to_hex();
                println!("{}", hex_str);
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("Compilation error: {}", err);
                ExitCode::FAILURE
            }
        }
    }
}
