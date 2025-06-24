use std::fs;

use camino::Utf8PathBuf;
use clap::{Parser as ClapParser, Subcommand};
use miette::{IntoDiagnostic, NamedSource, Result};

use crate::{compiler::Compiler, lexer::Lexer, parser::Parser, preprocessor::PreProcessor, vm::VM};

mod compiler;
mod lexer;
mod parser;
mod preprocessor;
mod span;
mod vm;

#[derive(ClapParser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    #[clap(visible_alias = "b", about = "Compile source code to bytecode")]
    Build {
        #[clap(required = true, help = "Path to the source file to compile")]
        input: Utf8PathBuf,

        #[clap(
            long,
            short = 'o',
            help = "Optional path to write the compiled bytecode output",
            default_value = "out.nyb"
        )]
        output: Utf8PathBuf,
    },

    #[clap(
        visible_alias = "r",
        about = "Compile and run source code in the virtual machine"
    )]
    Run {
        #[clap(required = true, help = "Path to the source file to compile and run")]
        input: Utf8PathBuf,

        #[clap(
            long,
            short = 'o',
            help = "Optional path to write the compiled bytecode before running",
            default_value = "out.nyb"
        )]
        output: Utf8PathBuf,

        #[clap(
            long,
            short = 'm',
            help = "Size of virtual machine memory in bytes",
            default_value_t = 4096
        )]
        memory: usize,
    },

    #[clap(
        visible_alias = "x",
        about = "Run existing bytecode in the virtual machine"
    )]
    Execute {
        #[clap(
            required = true,
            help = "Path to the precompiled bytecode file to execute"
        )]
        input: Utf8PathBuf,

        #[clap(
            long,
            short = 'm',
            help = "Size of virtual machine memory in bytes",
            default_value_t = 4096
        )]
        memory: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Build { input, output } => {
            let source_code = fs::read_to_string(&input).into_diagnostic()?;
            let named_source = NamedSource::new(input, source_code);

            let lexer = Lexer::new(named_source.clone());
            let mut parser = Parser::new(lexer);
            let mut preprocessor = PreProcessor::new(parser.parse()?);
            let mut compiler = Compiler::new(preprocessor.process()?, named_source);
            let bytecode = compiler.compile()?;

            fs::write(output, bytecode).into_diagnostic()?;
        }
        Command::Run {
            input,
            output,
            memory,
        } => {
            let source_code = fs::read_to_string(&input).into_diagnostic()?;
            let named_source = NamedSource::new(input, source_code);

            let lexer = Lexer::new(named_source.clone());
            let mut parser = Parser::new(lexer);
            let mut preprocessor = PreProcessor::new(parser.parse()?);
            let mut compiler = Compiler::new(preprocessor.process()?, named_source);
            let bytecode = compiler.compile()?;

            fs::write(output, bytecode).into_diagnostic()?;

            let mut vm = VM::new(Vec::from(bytecode), memory);
            vm.run()?;
        }
        Command::Execute { input, memory } => {
            let bytecode = fs::read(input).into_diagnostic()?;
            let mut vm = VM::new(bytecode, memory);
            vm.run()?;
        }
    }

    Ok(())
}
