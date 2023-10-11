pub mod ledger_transaction;

mod cmd_prepare;
mod cmd_run;
mod error;

pub use cmd_prepare::*;
pub use cmd_run::*;
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
    Run(Run),
}

pub fn run() -> Result<(), Error> {
    let cli = ReplayCli::parse();

    match cli.command {
        Command::Prepare(cmd) => cmd.run(),
        Command::Run(cmd) => cmd.run(),
    }
}
