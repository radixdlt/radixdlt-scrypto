pub mod cmd_call_function;
pub mod cmd_call_method;
pub mod cmd_export_abi;
pub mod cmd_generate_key_pair;
pub mod cmd_mint;
pub mod cmd_new_account;
pub mod cmd_new_badge_fixed;
pub mod cmd_new_badge_mutable;
pub mod cmd_new_token_fixed;
pub mod cmd_new_token_mutable;
pub mod cmd_publish;
pub mod cmd_reset;
pub mod cmd_run;
pub mod cmd_set_current_epoch;
pub mod cmd_set_default_account;
pub mod cmd_show;
pub mod cmd_show_configs;
pub mod cmd_show_ledger;
pub mod cmd_transfer;
mod config;
mod error;

pub use cmd_call_function::*;
pub use cmd_call_method::*;
pub use cmd_export_abi::*;
pub use cmd_generate_key_pair::*;
pub use cmd_mint::*;
pub use cmd_new_account::*;
pub use cmd_new_badge_fixed::*;
pub use cmd_new_badge_mutable::*;
pub use cmd_new_token_fixed::*;
pub use cmd_new_token_mutable::*;
pub use cmd_publish::*;
pub use cmd_reset::*;
pub use cmd_run::*;
pub use cmd_set_current_epoch::*;
pub use cmd_set_default_account::*;
pub use cmd_show::*;
pub use cmd_show_configs::*;
pub use cmd_show_ledger::*;
pub use cmd_transfer::*;
pub use config::*;
pub use error::*;

use clap::{Parser, Subcommand};
use radix_engine::ledger::*;
use radix_engine::model::*;
use radix_engine::transaction::*;
use std::fs;
use std::path::PathBuf;
use transaction_manifest::decompile;

use crate::ledger::*;

/// Build fast, reward everyone, and scale without friction
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, name = "resim")]
pub struct ResimCli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    CallFunction(CallFunction),
    CallMethod(CallMethod),
    ExportAbi(ExportAbi),
    GenerateKeyPair(GenerateKeyPair),
    Mint(Mint),
    NewAccount(NewAccount),
    NewBadgeFixed(NewBadgeFixed),
    NewBadgeMutable(NewBadgeMutable),
    NewTokenFixed(NewTokenFixed),
    NewTokenMutable(NewTokenMutable),
    Publish(Publish),
    Reset(Reset),
    Run(Run),
    SetCurrentEpoch(SetCurrentEpoch),
    SetDefaultAccount(SetDefaultAccount),
    ShowConfigs(ShowConfigs),
    ShowLedger(ShowLedger),
    Show(Show),
    Transfer(Transfer),
}

pub fn run() -> Result<(), Error> {
    let cli = ResimCli::parse();

    let mut out = std::io::stdout() ;

    match cli.command {
        Command::CallFunction(cmd) => cmd.run(&mut out),
        Command::CallMethod(cmd) => cmd.run(&mut out),
        Command::ExportAbi(cmd) => cmd.run(&mut out),
        Command::GenerateKeyPair(cmd) => cmd.run(&mut out),
        Command::Mint(cmd) => cmd.run(&mut out),
        Command::NewAccount(cmd) => cmd.run(&mut out),
        Command::NewBadgeFixed(cmd) => cmd.run(&mut out),
        Command::NewBadgeMutable(cmd) => cmd.run(&mut out),
        Command::NewTokenFixed(cmd) => cmd.run(&mut out),
        Command::NewTokenMutable(cmd) => cmd.run(&mut out),
        Command::Publish(cmd) => cmd.run(&mut out),
        Command::Reset(cmd) => cmd.run(&mut out),
        Command::Run(cmd) => cmd.run(&mut out),
        Command::SetCurrentEpoch(cmd) => cmd.run(&mut out),
        Command::SetDefaultAccount(cmd) => cmd.run(&mut out),
        Command::ShowConfigs(cmd) => cmd.run(&mut out),
        Command::ShowLedger(cmd) => cmd.run(&mut out),
        Command::Show(cmd) => cmd.run(&mut out),
        Command::Transfer(cmd) => cmd.run(&mut out),
    }
}

pub fn process_transaction<L: SubstateStore,O: std::io::Write >(
    signed: SignedTransaction,
    executor: &mut TransactionExecutor<L>,
    manifest: &Option<PathBuf>,
    out: &mut O
) -> Result<(), Error> {
    match manifest {
        Some(path) => {
            let decompiled = decompile(&signed.transaction).map_err(Error::DecompileError)?;
            fs::write(path, decompiled).map_err(Error::IOError)
        }
        None => {
            let receipt = executor
                .validate_and_execute(&signed)
                .map_err(Error::TransactionValidationError)?;
            writeln!(out, "{:?}", receipt).map_err(Error::IOError)?;
            receipt.result.map_err(Error::TransactionExecutionError)
        }
    }
}
