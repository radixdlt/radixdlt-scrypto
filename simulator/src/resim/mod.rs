mod addressing;
mod cmd_call_function;
mod cmd_call_method;
mod cmd_export_schema;
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
mod error;

pub use addressing::*;
pub use cmd_call_function::*;
pub use cmd_call_method::*;
pub use cmd_export_schema::*;
pub use cmd_generate_key_pair::*;
pub use cmd_mint::*;
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
pub use error::*;

pub const DEFAULT_SCRYPTO_DIR_UNDER_HOME: &'static str = ".scrypto";
pub const ENV_DATA_DIR: &'static str = "DATA_DIR";
pub const ENV_DISABLE_MANIFEST_OUTPUT: &'static str = "DISABLE_MANIFEST_OUTPUT";

use clap::{Parser, Subcommand};
use radix_engine::system::bootstrap::Bootstrapper;
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::TransactionOutcome;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::transaction::TransactionReceiptDisplayContextBuilder;
use radix_engine::transaction::TransactionResult;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::ScryptoVm;
use radix_engine_interface::api::node_modules::auth::ACCESS_RULES_BLUEPRINT;
use radix_engine_interface::api::node_modules::metadata::METADATA_BLUEPRINT;
use radix_engine_interface::api::node_modules::royalty::COMPONENT_ROYALTY_BLUEPRINT;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::package::PackageInfoSubstate;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use radix_engine_interface::crypto::hash;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_interface::schema::PackageSchema;
use radix_engine_interface::schema::{IndexedBlueprintSchema, IndexedPackageSchema};
use radix_engine_stores::interface::SubstateDatabase;
use radix_engine_stores::jmt_support::JmtMapper;
use radix_engine_stores::rocks_db::RocksdbSubstateStore;
use std::env;
use std::fs;
use std::path::PathBuf;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::manifest::decompile;
use transaction::model::Instruction;
use transaction::model::SystemTransaction;
use transaction::model::TestTransaction;
use transaction::model::TransactionManifest;
use utils::ContextualDisplay;

/// The address of the faucet component, test network only.
/// TODO: remove
pub const FAUCET_COMPONENT: ComponentAddress = ComponentAddress::new_or_panic([
    EntityType::GlobalGenericComponent as u8,
    1,
    214,
    31,
    81,
    94,
    195,
    164,
    245,
    22,
    133,
    219,
    196,
    153,
    116,
    249,
    229,
    98,
    136,
    55,
    21,
    2,
    33,
    180,
    121,
    11,
    178,
    57,
    153,
    132,
]);

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
    ExportSchema(ExportSchema),
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

pub fn run() -> Result<(), Error> {
    let cli = ResimCli::parse();

    let mut out = std::io::stdout();

    match cli.command {
        Command::CallFunction(cmd) => cmd.run(&mut out),
        Command::CallMethod(cmd) => cmd.run(&mut out),
        Command::ExportSchema(cmd) => cmd.run(&mut out),
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
    instructions: Vec<Instruction>,
    blobs: Vec<Vec<u8>>,
    initial_proofs: BTreeSet<NonFungibleGlobalId>,
    trace: bool,
    print_receipt: bool,
    out: &mut O,
) -> Result<TransactionReceipt, Error> {
    let scrypto_interpreter = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = RocksdbSubstateStore::standard(get_data_dir()?);
    Bootstrapper::new(&mut substate_db, &scrypto_interpreter).bootstrap_test_default();

    let nonce = get_nonce()?;
    let transaction = SystemTransaction {
        instructions,
        blobs,
        nonce,
        pre_allocated_ids: BTreeSet::new(),
    };

    let receipt = execute_and_commit_transaction(
        &mut substate_db,
        &scrypto_interpreter,
        &FeeReserveConfig::default(),
        &ExecutionConfig::standard().with_trace(trace),
        &transaction.get_executable(initial_proofs),
    );

    if print_receipt {
        let encoder = Bech32Encoder::for_simulator();
        let display_context = TransactionReceiptDisplayContextBuilder::new()
            .encoder(&encoder)
            .schema_lookup_callback(|event_type_identifier: &EventTypeIdentifier| {
                get_event_schema(&substate_db, event_type_identifier)
            })
            .build();
        writeln!(out, "{}", receipt.display(display_context)).map_err(Error::IOError)?;
    }
    drop(substate_db);

    process_receipt(receipt)
}

pub fn handle_manifest<O: std::io::Write>(
    manifest: TransactionManifest,
    signing_keys: &Option<String>,
    network: &Option<String>,
    write_manifest: &Option<PathBuf>,
    trace: bool,
    print_receipt: bool,
    out: &mut O,
) -> Result<Option<TransactionReceipt>, Error> {
    let network = match network {
        Some(n) => NetworkDefinition::from_str(&n).map_err(Error::ParseNetworkError)?,
        None => NetworkDefinition::simulator(),
    };
    match write_manifest {
        Some(path) => {
            if !env::var(ENV_DISABLE_MANIFEST_OUTPUT).is_ok() {
                let manifest_str =
                    decompile(&manifest.instructions, &network).map_err(Error::DecompileError)?;
                fs::write(path, manifest_str).map_err(Error::IOError)?;
                for blob in manifest.blobs {
                    let blob_hash = hash(&blob);
                    let mut blob_path = path
                        .parent()
                        .expect("Manifest file parent not found")
                        .to_owned();
                    blob_path.push(format!("{}.blob", blob_hash));
                    fs::write(blob_path, blob).map_err(Error::IOError)?;
                }
            }
            Ok(None)
        }
        None => {
            let scrypto_interpreter = ScryptoVm::<DefaultWasmEngine>::default();
            let mut substate_db = RocksdbSubstateStore::standard(get_data_dir()?);
            Bootstrapper::new(&mut substate_db, &scrypto_interpreter).bootstrap_test_default();

            let sks = get_signing_keys(signing_keys)?;
            let initial_proofs = sks
                .into_iter()
                .map(|e| NonFungibleGlobalId::from_public_key(&e.public_key()))
                .collect::<BTreeSet<NonFungibleGlobalId>>();
            let nonce = get_nonce()?;
            let transaction = TestTransaction::new(manifest, nonce, DEFAULT_COST_UNIT_LIMIT);

            let receipt = execute_and_commit_transaction(
                &mut substate_db,
                &scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::standard().with_trace(trace),
                &transaction.get_executable(initial_proofs),
            );

            if print_receipt {
                let encoder = Bech32Encoder::for_simulator();
                let display_context = TransactionReceiptDisplayContextBuilder::new()
                    .encoder(&encoder)
                    .schema_lookup_callback(|event_type_identifier: &EventTypeIdentifier| {
                        get_event_schema(&substate_db, event_type_identifier)
                    })
                    .build();
                writeln!(out, "{}", receipt.display(display_context)).map_err(Error::IOError)?;
            }
            drop(substate_db);

            process_receipt(receipt).map(Option::Some)
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
            Err(Error::TransactionRejected(rejection.error.clone()))
        }
        TransactionResult::Abort(result) => Err(Error::TransactionAborted(result.reason.clone())),
    }
}

pub fn get_signing_keys(
    signing_keys: &Option<String>,
) -> Result<Vec<EcdsaSecp256k1PrivateKey>, Error> {
    let private_keys = if let Some(keys) = signing_keys {
        keys.split(",")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|key| {
                hex::decode(key)
                    .map_err(|_| Error::InvalidPrivateKey)
                    .and_then(|bytes| {
                        EcdsaSecp256k1PrivateKey::from_bytes(&bytes)
                            .map_err(|_| Error::InvalidPrivateKey)
                    })
            })
            .collect::<Result<Vec<EcdsaSecp256k1PrivateKey>, Error>>()?
    } else {
        vec![get_default_private_key()?]
    };

    Ok(private_keys)
}

pub fn export_package_schema(
    package_address: PackageAddress,
) -> Result<IndexedPackageSchema, Error> {
    let scrypto_interpreter = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = RocksdbSubstateStore::standard(get_data_dir()?);
    Bootstrapper::new(&mut substate_db, &scrypto_interpreter).bootstrap_test_default();

    let package_info = substate_db
        .get_mapped_substate::<JmtMapper, PackageInfoSubstate>(
            package_address.as_node_id(),
            USER_BASE_MODULE,
            &PackageOffset::Info.into(),
        )
        .ok_or(Error::PackageNotFound(package_address))?;

    Ok(package_info.schema)
}

pub fn export_blueprint_schema(
    package_address: PackageAddress,
    blueprint_name: &str,
) -> Result<IndexedBlueprintSchema, Error> {
    let schema = export_package_schema(package_address)?
        .blueprints
        .get(blueprint_name)
        .cloned()
        .ok_or(Error::BlueprintNotFound(
            package_address,
            blueprint_name.to_string(),
        ))?;
    Ok(schema)
}

pub fn get_blueprint(component_address: ComponentAddress) -> Result<Blueprint, Error> {
    let scrypto_interpreter = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = RocksdbSubstateStore::standard(get_data_dir()?);
    Bootstrapper::new(&mut substate_db, &scrypto_interpreter).bootstrap_test_default();

    let type_info = substate_db
        .get_mapped_substate::<JmtMapper, TypeInfoSubstate>(
            component_address.as_node_id(),
            TYPE_INFO_BASE_MODULE,
            &TypeInfoOffset::TypeInfo.into(),
        )
        .ok_or(Error::ComponentNotFound(component_address))?;

    match type_info {
        TypeInfoSubstate::Object(ObjectInfo { blueprint, .. }) => Ok(blueprint.clone()),
        _ => panic!("Unexpected"),
    }
}

pub fn get_event_schema<S: SubstateDatabase>(
    substate_db: &S,
    event_type_identifier: &EventTypeIdentifier,
) -> Option<(LocalTypeIndex, ScryptoSchema)> {
    let (package_address, blueprint_name, local_type_index) = match event_type_identifier {
        EventTypeIdentifier(Emitter::Method(node_id, node_module), local_type_index) => {
            match node_module {
                ObjectModuleId::AccessRules => (
                    ACCESS_RULES_PACKAGE,
                    ACCESS_RULES_BLUEPRINT.into(),
                    *local_type_index,
                ),
                ObjectModuleId::Royalty => (
                    ROYALTY_PACKAGE,
                    COMPONENT_ROYALTY_BLUEPRINT.into(),
                    *local_type_index,
                ),
                ObjectModuleId::Metadata => (
                    METADATA_PACKAGE,
                    METADATA_BLUEPRINT.into(),
                    *local_type_index,
                ),
                ObjectModuleId::SELF => {
                    let type_info = substate_db
                        .get_mapped_substate::<JmtMapper, TypeInfoSubstate>(
                            node_id,
                            TYPE_INFO_BASE_MODULE,
                            &TypeInfoOffset::TypeInfo.into(),
                        )
                        .unwrap();
                    match type_info {
                        TypeInfoSubstate::Object(ObjectInfo { blueprint, .. }) => (
                            blueprint.package_address,
                            blueprint.blueprint_name,
                            *local_type_index,
                        ),
                        TypeInfoSubstate::KeyValueStore(..)
                        | TypeInfoSubstate::Index
                        | TypeInfoSubstate::SortedIndex => return None,
                    }
                }
            }
        }
        EventTypeIdentifier(Emitter::Function(node_id, _, blueprint_name), local_type_index) => (
            PackageAddress::new_or_panic(node_id.clone().into()),
            blueprint_name.to_owned(),
            *local_type_index,
        ),
    };

    let package_info = substate_db
        .get_mapped_substate::<JmtMapper, PackageInfoSubstate>(
            package_address.as_node_id(),
            USER_BASE_MODULE,
            &PackageOffset::Info.into(),
        )
        .unwrap();

    Some((
        local_type_index,
        package_info
            .schema
            .blueprints
            .get(&blueprint_name)
            .unwrap()
            .schema
            .clone(),
    ))
}
