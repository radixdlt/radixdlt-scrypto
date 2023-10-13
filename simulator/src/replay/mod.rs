pub mod ledger_transaction;
pub mod ledger_transaction_execution;
pub mod txn_reader;

mod cmd_execute;
mod cmd_execute_in_memory;
mod cmd_prepare;
mod cmd_sync;
mod error;

pub use cmd_execute::*;
pub use cmd_execute_in_memory::*;
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
    Prepare(Prepare),
    Execute(TxnExecute),
    TxnExecuteInMemory(TxnExecuteInMemory),
    Sync(TxnSync),
}

pub fn run() -> Result<(), Error> {
    let cli = ReplayCli::parse();

    match cli.command {
        Command::Prepare(cmd) => cmd.run(),
        Command::Execute(cmd) => cmd.run(),
        Command::TxnExecuteInMemory(cmd) => cmd.run(),
        Command::Sync(cmd) => cmd.sync(),
    }
}
