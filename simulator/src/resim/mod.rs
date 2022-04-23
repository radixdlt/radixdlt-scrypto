mod cmd_call_function;
mod cmd_call_method;
mod cmd_export_abi;
mod cmd_generate_key_pair;
mod cmd_mint;
mod cmd_new_account;
mod cmd_new_badge_fixed;
mod cmd_new_badge_mutable;
mod cmd_new_token_fixed;
mod cmd_new_token_mutable;
mod cmd_publish;
mod cmd_reset;
mod cmd_run;
mod cmd_set_current_epoch;
mod cmd_set_default_account;
mod cmd_show;
mod cmd_show_configs;
mod cmd_show_ledger;
mod cmd_transfer;
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

    match cli.command {
        Command::CallFunction(cmd) => cmd.run(),
        Command::CallMethod(cmd) => cmd.run(),
        Command::ExportAbi(cmd) => cmd.run(),
        Command::GenerateKeyPair(cmd) => cmd.run(),
        Command::Mint(cmd) => cmd.run(),
        Command::NewAccount(cmd) => cmd.run(),
        Command::NewBadgeFixed(cmd) => cmd.run(),
        Command::NewBadgeMutable(cmd) => cmd.run(),
        Command::NewTokenFixed(cmd) => cmd.run(),
        Command::NewTokenMutable(cmd) => cmd.run(),
        Command::Publish(cmd) => cmd.run(),
        Command::Reset(cmd) => cmd.run(),
        Command::Run(cmd) => cmd.run(),
        Command::SetCurrentEpoch(cmd) => cmd.run(),
        Command::SetDefaultAccount(cmd) => cmd.run(),
        Command::ShowConfigs(cmd) => cmd.run(),
        Command::ShowLedger(cmd) => cmd.run(),
        Command::Show(cmd) => cmd.run(),
        Command::Transfer(cmd) => cmd.run(),
    }
}

pub fn process_transaction<L: SubstateStore>(
    signed: SignedTransaction,
    executor: &mut TransactionExecutor<L>,
    manifest: &Option<PathBuf>,
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
            println!("{:?}", receipt);
            receipt.result.map_err(Error::TransactionExecutionError)
        }
    }
}
