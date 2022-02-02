mod cmd_call_function;
mod cmd_call_method;
mod cmd_export_abi;
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
use radix_engine::model::*;
use radix_engine::transaction::*;
use scrypto::types::*;

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

pub struct TransactionRunner {
    configs: Configs,
    ledger: FileBasedLedger,
}

impl TransactionRunner {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            configs: get_configs()?,
            ledger: FileBasedLedger::with_bootstrap(get_data_dir()?),
        })
    }

    pub fn default_account(&self) -> Result<Address, Error> {
        self.configs
            .default_account
            .ok_or(Error::NoDefaultAccount)
            .map(|a| a.0)
    }

    pub fn default_signers(&self) -> Result<Vec<Address>, Error> {
        self.configs
            .default_account
            .ok_or(Error::NoDefaultAccount)
            .map(|a| vec![a.1])
    }

    pub fn executor(&mut self, trace: bool) -> TransactionExecutor<FileBasedLedger> {
        TransactionExecutor::new(
            &mut self.ledger,
            self.configs.current_epoch,
            self.configs.nonce,
            trace,
        )
    }

    pub fn run_transaction(
        &mut self,
        transaction: Transaction,
        trace: bool,
        receipt_handler: fn(&Receipt) -> (),
    ) -> Result<(), Error> {
        let mut executor = self.executor(trace);
        let receipt = executor
            .run(transaction)
            .map_err(Error::TransactionValidationError)?;
        receipt_handler(&receipt);

        if receipt.result.is_ok() {
            self.configs.nonce = executor.nonce();
            set_configs(self.configs.clone())?;
        }

        receipt.result.map_err(Error::TransactionExecutionError)
    }
}
