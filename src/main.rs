#![allow(clippy::uninlined_format_args)]
#![allow(clippy::new_without_default)]
#![allow(clippy::enum_variant_names)]

use std::fs::File;
use std::io::{Cursor, Read};
use std::process::ExitCode;

use crate::compile::Compiler;
use crate::parse::Parser;

mod builtin;
mod compile;
mod parse;
mod util;
mod vm;

#[derive(clap::Parser, Debug)]
struct Args {
    #[clap(flatten)]
    source_args: SourceArgs,
    /// print error values directly to stderr with no extra information
    #[arg(long)]
    raw_errors: bool,
    /// print debug information after compiling the program
    #[arg(long)]
    print_vm: bool,
    /// compile the program and find errors, but do not run it
    #[arg(long)]
    check: bool,
}

#[derive(Debug, clap::Parser)]
#[group(required = true, multiple = false)]
struct SourceArgs {
    /// a source file to run
    #[arg(group = "source")]
    file: Option<String>,
    /// one or more statements to execute
    #[arg(short = 'c', group = "source")]
    code: Option<String>,
}

fn main() -> ExitCode {
    let args = <Args as clap::Parser>::parse();
    assert!(args.source_args.file.is_some() ^ args.source_args.code.is_some());
    let (source, filename): (Box<dyn Read>, &str) = if let Some(filename) = &args.source_args.file {
        match File::open(filename) {
            Ok(file) => (Box::new(file), filename),
            Err(err) => {
                eprintln!(
                    "failed to open source file {:?}: {}",
                    args.source_args.file, err
                );
                return ExitCode::FAILURE;
            }
        }
    } else {
        let code = &args.source_args.code.unwrap();
        let code = format!("Main -> {{$scanner_reset_location\n{}\n}}", code);
        let cursor = Box::new(Cursor::new(code.into_bytes()));
        (cursor, "cmdline")
    };
    let code = match Parser::new(filename, source).parse_all() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        }
    };
    let mut vm = match Compiler::new(code).and_then(|compiler| compiler.build()) {
        Ok(vm) => vm,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        }
    };
    vm.options_mut().raw_errors = args.raw_errors;
    if args.print_vm {
        println!("{:#?}", &vm);
    }
    if !args.check {
        vm.run();
    }
    ExitCode::SUCCESS
}
