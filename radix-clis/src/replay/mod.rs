pub mod ledger_transaction_execution;
pub mod txn_reader;

mod cmd_alloc_dump;
mod cmd_execute;
mod cmd_execute_in_memory;
mod cmd_measure;
mod cmd_prepare;
mod cmd_sync;
mod error;

pub use cmd_alloc_dump::*;
pub use cmd_execute::*;
pub use cmd_execute_in_memory::*;
pub use cmd_measure::*;
pub use cmd_prepare::*;
pub use cmd_sync::*;
pub use error::*;

use clap::{Parser, Subcommand};

/// Transaction replay toolkit
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, name = "replay")]
pub struct ReplayCli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Prepare(TxnPrepare),
    Execute(TxnExecute),
    ExecuteInMemory(TxnExecuteInMemory),
    Sync(TxnSync),
    Measure(TxnMeasure),
    AllocDump(TxnAllocDump),
}

pub fn run() -> Result<(), String> {
    let cli = ReplayCli::parse();

    match cli.command {
        Command::Prepare(cmd) => cmd.run(),
        Command::Execute(cmd) => cmd.run(),
        Command::ExecuteInMemory(cmd) => cmd.run(),
        Command::Sync(cmd) => cmd.sync(),
        Command::Measure(cmd) => cmd.run(),
        Command::AllocDump(cmd) => cmd.run(),
    }
}
