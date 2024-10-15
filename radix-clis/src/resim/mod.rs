mod addressing;
mod cmd_call_function;
mod cmd_call_method;
mod cmd_export_package_definition;
mod cmd_generate_key_pair;
mod cmd_mint;
mod cmd_new_account;
mod cmd_new_badge_fixed;
mod cmd_new_badge_mutable;
mod cmd_new_simple_badge;
mod cmd_new_token_fixed;
mod cmd_new_token_mutable;
mod cmd_publish;
mod cmd_reset;
mod cmd_run;
mod cmd_set_current_epoch;
mod cmd_set_current_time;
mod cmd_set_default_account;
mod cmd_show;
mod cmd_show_configs;
mod cmd_show_ledger;
mod cmd_transfer;
mod config;
mod dumper;
mod error;

pub use addressing::*;
pub use cmd_call_function::CallFunction;
pub use cmd_call_method::CallMethod;
pub use cmd_export_package_definition::*;
pub use cmd_generate_key_pair::*;
pub use cmd_new_account::*;
pub use cmd_new_badge_fixed::*;
pub use cmd_new_badge_mutable::*;
pub use cmd_new_simple_badge::*;
pub use cmd_new_token_fixed::*;
pub use cmd_new_token_mutable::*;
pub use cmd_publish::*;
pub use cmd_reset::*;
pub use cmd_run::*;
pub use cmd_set_current_epoch::*;
pub use cmd_set_current_time::*;
pub use cmd_set_default_account::*;
pub use cmd_show::*;
pub use cmd_show_configs::*;
pub use cmd_show_ledger::*;
pub use cmd_transfer::*;
pub use config::*;
pub use dumper::*;
pub use error::*;

pub const DEFAULT_SCRYPTO_DIR_UNDER_HOME: &'static str = ".scrypto";
pub const ENV_DATA_DIR: &'static str = "DATA_DIR";
pub const ENV_DISABLE_MANIFEST_OUTPUT: &'static str = "DISABLE_MANIFEST_OUTPUT";

use crate::prelude::*;
use clap::{Parser, Subcommand};
use radix_engine::blueprints::consensus_manager::*;
use radix_engine::blueprints::models::FieldPayload;
use radix_engine::system::system_db_reader::*;
use radix_engine::transaction::*;
use radix_engine_interface::api::ModuleId;
use radix_engine_interface::blueprints::package::*;
use radix_substate_store_impls::rocks_db::RocksdbSubstateStore;
use radix_transactions::validation::TransactionValidator;

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
    ExportPackageDefinition(ExportPackageDefinition),
    GenerateKeyPair(GenerateKeyPair),
    Mint(crate::resim::cmd_mint::Mint),
    NewAccount(NewAccount),
    NewSimpleBadge(NewSimpleBadge),
    NewBadgeFixed(NewBadgeFixed),
    NewBadgeMutable(NewBadgeMutable),
    NewTokenFixed(NewTokenFixed),
    NewTokenMutable(NewTokenMutable),
    Publish(Publish),
    Reset(Reset),
    Run(Run),
    SetCurrentEpoch(SetCurrentEpoch),
    SetCurrentTime(SetCurrentTime),
    SetDefaultAccount(SetDefaultAccount),
    ShowConfigs(ShowConfigs),
    ShowLedger(ShowLedger),
    Show(Show),
    Transfer(Transfer),
}

pub fn run() -> Result<(), String> {
    let cli = ResimCli::parse();

    let mut out = std::io::stdout();

    match cli.command {
        Command::CallFunction(cmd) => cmd.run(&mut out),
        Command::CallMethod(cmd) => cmd.run(&mut out),
        Command::ExportPackageDefinition(cmd) => cmd.run(&mut out),
        Command::GenerateKeyPair(cmd) => cmd.run(&mut out),
        Command::Mint(cmd) => cmd.run(&mut out),
        Command::NewAccount(cmd) => cmd.run(&mut out),
        Command::NewSimpleBadge(cmd) => cmd.run(&mut out).map(|_| ()),
        Command::NewBadgeFixed(cmd) => cmd.run(&mut out),
        Command::NewBadgeMutable(cmd) => cmd.run(&mut out),
        Command::NewTokenFixed(cmd) => cmd.run(&mut out),
        Command::NewTokenMutable(cmd) => cmd.run(&mut out),
        Command::Publish(cmd) => cmd.run(&mut out),
        Command::Reset(cmd) => cmd.run(&mut out),
        Command::Run(cmd) => cmd.run(&mut out),
        Command::SetCurrentEpoch(cmd) => cmd.run(&mut out),
        Command::SetCurrentTime(cmd) => cmd.run(&mut out),
        Command::SetDefaultAccount(cmd) => cmd.run(&mut out),
        Command::ShowConfigs(cmd) => cmd.run(&mut out),
        Command::ShowLedger(cmd) => cmd.run(&mut out),
        Command::Show(cmd) => cmd.run(&mut out),
        Command::Transfer(cmd) => cmd.run(&mut out),
    }
}

pub fn handle_system_transaction<O: std::io::Write>(
    manifest: SystemTransactionManifestV1,
    initial_proofs: BTreeSet<NonFungibleGlobalId>,
    trace: bool,
    print_receipt: bool,
    out: &mut O,
) -> Result<TransactionReceipt, Error> {
    let SimulatorEnvironment {
        mut db, vm_modules, ..
    } = SimulatorEnvironment::new()?;

    let nonce = get_nonce()?;
    let unique_hash = hash(format!("Simulator system transaction: {}", nonce));
    let transaction = manifest.into_transaction(unique_hash);
    let validator = TransactionValidator::new(&db, &NetworkDefinition::simulator());

    let receipt = execute_and_commit_transaction(
        &mut db,
        &vm_modules,
        &ExecutionConfig::for_system_transaction(NetworkDefinition::simulator())
            .with_kernel_trace(trace),
        transaction
            .with_proofs(initial_proofs)
            .into_executable(&validator)
            .map_err(Error::TransactionPrepareError)?,
    );

    if print_receipt {
        let encoder = AddressBech32Encoder::for_simulator();
        let display_context = TransactionReceiptDisplayContextBuilder::new()
            .encoder(&encoder)
            .schema_lookup_from_db(&db)
            .build();
        writeln!(out, "{}", receipt.display(display_context)).map_err(Error::IOError)?;
    }
    drop(db);

    process_receipt(receipt)
}

pub fn handle_manifest<O: std::io::Write>(
    manifest: AnyManifest,
    signing_keys: &Option<String>,
    network: &Option<String>,
    write_manifest: &Option<PathBuf>,
    trace: bool,
    print_receipt: bool,
    out: &mut O,
) -> Result<Option<TransactionReceipt>, String> {
    let network = match network {
        Some(n) => NetworkDefinition::from_str(&n).map_err(Error::ParseNetworkError)?,
        None => NetworkDefinition::simulator(),
    };

    manifest
        .validate(ValidationRuleset::all())
        .map_err(|err| format!("{err:?}"))?;

    match write_manifest {
        Some(path) => {
            if !env::var(ENV_DISABLE_MANIFEST_OUTPUT).is_ok() {
                let manifest_str =
                    decompile_any(&manifest, &network).map_err(Error::DecompileError)?;
                write_ensuring_folder_exists(path, manifest_str).map_err(Error::IOError)?;
                for (blob_hash, blob) in manifest.get_blobs() {
                    let mut blob_path = path
                        .parent()
                        .expect("Manifest file parent not found")
                        .to_owned();
                    blob_path.push(format!("{}.blob", blob_hash));
                    write_ensuring_folder_exists(blob_path, blob).map_err(Error::IOError)?;
                }
            }
            Ok(None)
        }
        None => {
            let SimulatorEnvironment {
                mut db, vm_modules, ..
            } = SimulatorEnvironment::new()?;

            let sks = get_signing_keys(signing_keys)?;
            let initial_proofs = sks
                .into_iter()
                .map(|e| NonFungibleGlobalId::from_public_key(&e.public_key()))
                .collect::<BTreeSet<NonFungibleGlobalId>>();
            let nonce = get_nonce()?;
            let validator = TransactionValidator::new(&db, &NetworkDefinition::simulator());
            let transaction =
                TestTransaction::new_from_any_manifest(manifest, nonce, initial_proofs)?;

            let receipt = execute_and_commit_transaction(
                &mut db,
                &vm_modules,
                &ExecutionConfig::for_test_transaction().with_kernel_trace(trace),
                transaction
                    .into_executable(&validator)
                    .map_err(Error::TransactionPrepareError)?,
            );

            if print_receipt {
                let encoder = AddressBech32Encoder::for_simulator();
                let display_context = TransactionReceiptDisplayContextBuilder::new()
                    .encoder(&encoder)
                    .schema_lookup_from_db(&db)
                    .build();
                writeln!(out, "{}", receipt.display(display_context)).map_err(Error::IOError)?;
            }
            drop(db);

            process_receipt(receipt)
                .map(Option::Some)
                .map_err(|err| err.into())
        }
    }
}

pub fn process_receipt(receipt: TransactionReceipt) -> Result<TransactionReceipt, Error> {
    match &receipt.result {
        TransactionResult::Commit(commit) => {
            let mut configs = get_configs()?;
            configs.nonce = get_nonce()? + 1;
            set_configs(&configs)?;

            match &commit.outcome {
                TransactionOutcome::Failure(error) => Err(Error::TransactionFailed(error.clone())),
                TransactionOutcome::Success(_) => Ok(receipt),
            }
        }
        TransactionResult::Reject(rejection) => {
            Err(Error::TransactionRejected(rejection.reason.clone()))
        }
        TransactionResult::Abort(result) => Err(Error::TransactionAborted(result.reason.clone())),
    }
}

pub fn parse_private_key_from_bytes(slice: &[u8]) -> Result<Secp256k1PrivateKey, Error> {
    Secp256k1PrivateKey::from_bytes(slice).map_err(|_| Error::InvalidPrivateKey)
}

pub fn parse_private_key_from_str(key: &str) -> Result<Secp256k1PrivateKey, Error> {
    hex::decode(key)
        .map_err(|_| Error::InvalidPrivateKey)
        .and_then(|bytes| parse_private_key_from_bytes(&bytes))
}

pub fn get_signing_keys(signing_keys: &Option<String>) -> Result<Vec<Secp256k1PrivateKey>, Error> {
    let private_keys = if let Some(keys) = signing_keys {
        keys.split(",")
            .map(str::trim)
            .filter(|s: &&str| !s.is_empty())
            .map(parse_private_key_from_str)
            .collect::<Result<Vec<Secp256k1PrivateKey>, Error>>()?
    } else {
        vec![get_default_private_key()?]
    };

    Ok(private_keys)
}

pub fn export_package_schema(
    package_address: PackageAddress,
) -> Result<BTreeMap<BlueprintVersionKey, BlueprintDefinition>, Error> {
    let SimulatorEnvironment { db, .. } = SimulatorEnvironment::new()?;

    let system_reader = SystemDatabaseReader::new(&db);
    let package_definition = system_reader.get_package_definition(package_address);
    Ok(package_definition)
}

pub fn export_object_info(component_address: ComponentAddress) -> Result<ObjectInfo, Error> {
    let SimulatorEnvironment { db, .. } = SimulatorEnvironment::new()?;

    let system_reader = SystemDatabaseReader::new(&db);
    system_reader
        .get_object_info(component_address)
        .map_err(|_| Error::ComponentNotFound(component_address))
}

pub fn export_schema(
    node_id: &NodeId,
    schema_hash: SchemaHash,
) -> Result<VersionedScryptoSchema, Error> {
    let SimulatorEnvironment { db, .. } = SimulatorEnvironment::new()?;

    let system_reader = SystemDatabaseReader::new(&db);
    let schema = system_reader
        .get_schema(node_id, &schema_hash)
        .map_err(|_| Error::SchemaNotFound(*node_id, schema_hash))?;

    Ok(schema.as_ref().clone())
}

pub fn export_blueprint_interface(
    package_address: PackageAddress,
    blueprint_name: &str,
) -> Result<BlueprintInterface, Error> {
    let interface = export_package_schema(package_address)?
        .get(&BlueprintVersionKey::new_default(blueprint_name))
        .cloned()
        .ok_or(Error::BlueprintNotFound(
            package_address,
            blueprint_name.to_string(),
        ))?
        .interface;
    Ok(interface)
}

pub fn get_blueprint_id(component_address: ComponentAddress) -> Result<BlueprintId, Error> {
    let SimulatorEnvironment { db, .. } = SimulatorEnvironment::new()?;

    let system_reader = SystemDatabaseReader::new(&db);
    let object_info = system_reader
        .get_object_info(component_address)
        .expect("Unexpected");
    Ok(object_info.blueprint_info.blueprint_id)
}

pub fn db_upsert_timestamps(
    milli_timestamp: ProposerMilliTimestampSubstate,
    minute_timestamp: ProposerMinuteTimestampSubstate,
) -> Result<(), Error> {
    let SimulatorEnvironment { mut db, .. } = SimulatorEnvironment::new()?;

    let mut writer = SystemDatabaseWriter::new(&mut db);

    writer
        .write_typed_object_field(
            CONSENSUS_MANAGER.as_node_id(),
            ModuleId::Main,
            ConsensusManagerField::ProposerMilliTimestamp.field_index(),
            ConsensusManagerProposerMilliTimestampFieldPayload::from_content_source(
                milli_timestamp,
            ),
        )
        .unwrap();

    writer
        .write_typed_object_field(
            CONSENSUS_MANAGER.as_node_id(),
            ModuleId::Main,
            ConsensusManagerField::ProposerMinuteTimestamp.field_index(),
            ConsensusManagerProposerMinuteTimestampFieldPayload::from_content_source(
                minute_timestamp,
            ),
        )
        .unwrap();

    Ok(())
}

pub fn db_upsert_epoch(epoch: Epoch) -> Result<(), Error> {
    let SimulatorEnvironment { mut db, .. } = SimulatorEnvironment::new()?;

    let reader = SystemDatabaseReader::new(&db);

    let mut consensus_mgr_state = reader
        .read_typed_object_field::<ConsensusManagerStateFieldPayload>(
            CONSENSUS_MANAGER.as_node_id(),
            ModuleId::Main,
            ConsensusManagerField::State.field_index(),
        )
        .unwrap_or_else(|_| {
            ConsensusManagerStateFieldPayload::from_content_source(ConsensusManagerSubstate {
                epoch: Epoch::zero(),
                effective_epoch_start_milli: 0,
                actual_epoch_start_milli: 0,
                round: Round::zero(),
                current_leader: Some(0),
                started: true,
            })
        })
        .fully_update_and_into_latest_version();

    consensus_mgr_state.epoch = epoch;

    let mut writer = SystemDatabaseWriter::new(&mut db);

    writer
        .write_typed_object_field(
            CONSENSUS_MANAGER.as_node_id(),
            ModuleId::Main,
            ConsensusManagerField::State.field_index(),
            ConsensusManagerStateFieldPayload::from_content_source(consensus_mgr_state),
        )
        .unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_no_value() {
        let mut out = std::io::stdout();
        let rtn = Reset {}.run(&mut out);
        assert!(rtn.is_ok(), "Reset failed with: {:?}", rtn);
        let new_account = NewAccount {
            network: None,
            manifest: None,
            trace: false,
        };
        assert!(new_account.run(&mut out).is_ok());
        let cmd = Show { address: None };
        assert!(cmd.run(&mut out).is_ok());
    }

    fn test_pre_process_manifest() {
        temp_env::with_vars(
            vec![
                (
                    "faucet",
                    Some("system_sim1qsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpql4sktx"),
                ),
                (
                    "xrd",
                    Some("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag"),
                ),
            ],
            || {
                let manifest = r#"CALL_METHOD ComponentAddress("${  faucet  }") "free";\nTAKE_ALL_FROM_WORKTOP ResourceAddress("${xrd}") Bucket("bucket1");\n"#;
                let after = r#"CALL_METHOD ComponentAddress("system_sim1qsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpql4sktx") "free";\nTAKE_ALL_FROM_WORKTOP ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Bucket("bucket1");\n"#;
                assert_eq!(Run::pre_process_manifest(manifest), after);
            },
        );
    }

    fn test_set_default_account_validation() {
        let mut out = std::io::stdout();
        let private_key = Secp256k1PrivateKey::from_hex(
            "6847c11e2d602548dbf38789e0a1f4543c1e7719e4f591d4aa6e5684f5c13d9c",
        )
        .unwrap();
        let public_key = private_key.public_key().to_string();

        let make_cmd = |key_string: String| {
            return SetDefaultAccount {
                component_address: SimulatorComponentAddress::from_str(
                    "account_sim1c9yeaya6pehau0fn7vgavuggeev64gahsh05dauae2uu25njk224xz",
                )
                .unwrap(),
                private_key: key_string,
                owner_badge: SimulatorNonFungibleGlobalId::from_str(
                    "resource_sim1ngvrads4uj3rgq2v9s78fzhvry05dw95wzf3p9r8skhqusf44dlvmr:#1#",
                )
                .unwrap(),
            };
        };

        assert!(make_cmd(private_key.to_hex()).run(&mut out).is_ok());
        assert!(make_cmd(public_key.to_string()).run(&mut out).is_err());
    }

    #[test]
    fn serial_resim_command_tests() {
        test_no_value();
        test_pre_process_manifest();
        test_set_default_account_validation();
    }
}
