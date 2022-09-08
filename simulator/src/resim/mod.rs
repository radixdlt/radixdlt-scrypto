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

pub const DEFAULT_SCRYPTO_DIR_UNDER_HOME: &'static str = ".scrypto";
pub const ENV_DATA_DIR: &'static str = "DATA_DIR";
pub const ENV_DISABLE_MANIFEST_OUTPUT: &'static str = "DISABLE_MANIFEST_OUTPUT";

use clap::{Parser, Subcommand};
use radix_engine::constants::*;
use radix_engine::engine::ExecutionPrivilege;
use radix_engine::model::*;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::transaction::TransactionExecutor;
use radix_engine::transaction::TransactionOutcome;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::transaction::TransactionResult;
use radix_engine::types::*;
use radix_engine::wasm::*;
use radix_engine_stores::rocks_db::RadixEngineDB;
use scrypto::abi;
use std::env;
use std::fs;
use std::path::PathBuf;
use transaction::builder::ManifestBuilder;
use transaction::manifest::decompile;
use transaction::model::TestTransaction;
use transaction::model::TransactionManifest;
use transaction::signing::EcdsaPrivateKey;

/// Build fast, reward everyone, and scale without friction
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, name = "resim")]
pub struct ResimCli {
    #[clap(subcommand)]
    pub(crate) command: Command,
}

impl ResimCli {
    pub fn get_command(&self) -> &Command {
        &self.command
    }
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

    let mut out = std::io::stdout();

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

pub fn handle_manifest<O: std::io::Write>(
    manifest: TransactionManifest,
    signing_keys: &Option<String>,
    manifest_path: &Option<PathBuf>,
    execution_privilege: ExecutionPrivilege,
    trace: bool,
    output_receipt: bool,
    out: &mut O,
) -> Result<Option<TransactionReceipt>, Error> {
    match manifest_path {
        Some(path) => {
            if !env::var(ENV_DISABLE_MANIFEST_OUTPUT).is_ok() {
                let bech32_encoder = Bech32Encoder::new(&NetworkDefinition::local_simulator());

                let manifest =
                    decompile(&manifest, &bech32_encoder).map_err(Error::DecompileError)?;
                fs::write(path, manifest).map_err(Error::IOError)?;
            }
            Ok(None)
        }
        None => {
            let mut substate_store = RadixEngineDB::with_bootstrap(get_data_dir()?);
            let mut wasm_engine = DefaultWasmEngine::new();
            let mut wasm_instrumenter = WasmInstrumenter::new();
            let mut executor = TransactionExecutor::new(
                &mut substate_store,
                &mut wasm_engine,
                &mut wasm_instrumenter,
            );

            let sks = get_signing_keys(signing_keys)?;
            let pks = sks
                .iter()
                .map(|e| e.public_key())
                .collect::<Vec<EcdsaPublicKey>>();
            let nonce = get_nonce()?;
            let transaction = TestTransaction::new(manifest, nonce, pks);

            let receipt = executor.execute_and_commit(
                &transaction,
                &ExecutionConfig {
                    cost_unit_price: DEFAULT_COST_UNIT_PRICE.parse().unwrap(),
                    max_call_depth: DEFAULT_MAX_CALL_DEPTH,
                    system_loan: DEFAULT_SYSTEM_LOAN,
                    execution_privilege,
                    trace,
                },
            );

            if output_receipt {
                writeln!(out, "{:?}", receipt).map_err(Error::IOError)?;
            }

            if receipt.is_commit() {
                let mut configs = get_configs()?;
                configs.nonce = nonce + 1;
                set_configs(&configs)?;
                return Ok(Some(receipt));
            }

            match receipt.result {
                TransactionResult::Commit(commit) => match commit.outcome {
                    TransactionOutcome::Failure(error) => {
                        Err(Error::TransactionExecutionError(error))
                    }
                    TransactionOutcome::Success(..) => {
                        panic!("Success case handled above to appease borrowing rules")
                    }
                },
                TransactionResult::Reject(rejection) => {
                    Err(Error::TransactionRejected(rejection.error))
                }
            }
        }
    }
}

pub fn get_signing_keys(signing_keys: &Option<String>) -> Result<Vec<EcdsaPrivateKey>, Error> {
    let private_keys = if let Some(keys) = signing_keys {
        keys.split(",")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|key| {
                hex::decode(key)
                    .map_err(|_| Error::InvalidPrivateKey)
                    .and_then(|bytes| {
                        EcdsaPrivateKey::from_bytes(&bytes).map_err(|_| Error::InvalidPrivateKey)
                    })
            })
            .collect::<Result<Vec<EcdsaPrivateKey>, Error>>()?
    } else {
        vec![get_default_private_key()?]
    };

    Ok(private_keys)
}

pub fn export_abi(
    package_address: PackageAddress,
    blueprint_name: &str,
) -> Result<abi::BlueprintAbi, Error> {
    let mut substate_store = RadixEngineDB::with_bootstrap(get_data_dir()?);
    radix_engine::model::export_abi(&mut substate_store, package_address, blueprint_name)
        .map_err(Error::AbiExportError)
}

pub fn export_abi_by_component(
    component_address: ComponentAddress,
) -> Result<abi::BlueprintAbi, Error> {
    let mut substate_store = RadixEngineDB::with_bootstrap(get_data_dir()?);
    radix_engine::model::export_abi_by_component(&mut substate_store, component_address)
        .map_err(Error::AbiExportError)
}
