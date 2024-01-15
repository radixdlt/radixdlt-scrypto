use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use radix_engine::blueprints::consensus_manager::*;
use radix_engine::blueprints::models::FieldPayload;
use radix_engine::blueprints::pool::v1::constants::*;
use radix_engine::errors::*;
use radix_engine::system::bootstrap::*;
use radix_engine::system::checkers::*;
use radix_engine::system::system_callback::SystemConfig;
use radix_engine::system::system_db_reader::{
    ObjectCollectionKey, SystemDatabaseReader, SystemDatabaseWriter,
};
use radix_engine::system::system_substates::FieldSubstate;
use radix_engine::system::type_info::TypeInfoSubstate;
use radix_engine::transaction::{
    execute_preview, execute_transaction_with_system, BalanceChange, CommitResult,
    CostingParameters, ExecutionConfig, PreviewError, TransactionReceipt, TransactionResult,
    WrappedSystem,
};
use radix_engine::types::*;
use radix_engine::utils::*;
use radix_engine::vm::wasm::{DefaultWasmEngine, WasmValidatorConfigV1};
use radix_engine::vm::{NativeVm, NativeVmExtension, NoExtension, ScryptoVm, Vm};
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::ModuleId;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::ACCOUNT_SECURIFY_IDENT;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerConfig, ConsensusManagerGetCurrentEpochInput,
    ConsensusManagerGetCurrentTimeInputV2, ConsensusManagerNextRoundInput, EpochChangeCondition,
    LeaderProposalHistory, CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
    CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT, CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
    VALIDATOR_STAKE_AS_OWNER_IDENT,
};
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::pool::{
    OneResourcePoolInstantiateManifestInput, ONE_RESOURCE_POOL_INSTANTIATE_IDENT,
};
use radix_engine_interface::constants::CONSENSUS_MANAGER;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_interface::time::Instant;
use radix_engine_interface::{dec, freeze_roles, rule};
use radix_engine_queries::query::{ResourceAccounter, StateTreeTraverser, VaultFinder};
use radix_engine_queries::typed_native_events::to_typed_native_event;
use radix_engine_queries::typed_substate_layout::*;
use radix_engine_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_engine_store_interface::db_key_mapper::{DatabaseKeyMapper, MappedSubstateDatabase};
use radix_engine_store_interface::interface::{
    CommittableSubstateDatabase, DatabaseUpdate, ListableSubstateDatabase, SubstateDatabase,
};
use radix_engine_stores::hash_tree_support::HashTreeUpdatingDatabase;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use scrypto::prelude::*;
use transaction::prelude::*;
use transaction::validation::{
    NotarizedTransactionValidator, TransactionValidator, ValidationConfig,
};

pub struct Compile;

impl Compile {
    pub fn compile<P: AsRef<Path>>(package_dir: P) -> (Vec<u8>, PackageDefinition) {
        Self::compile_with_env_vars(
            package_dir,
            btreemap! {
                "RUSTFLAGS".to_owned() => "".to_owned(),
                "CARGO_ENCODED_RUSTFLAGS".to_owned() => "".to_owned(),
            },
        )
    }

    pub fn compile_with_env_vars<P: AsRef<Path>>(
        package_dir: P,
        env_vars: sbor::rust::collections::BTreeMap<String, String>,
    ) -> (Vec<u8>, PackageDefinition) {
        // Find wasm name
        let mut cargo = package_dir.as_ref().to_owned();
        cargo.push("Cargo.toml");
        let wasm_name = if cargo.exists() {
            let content = fs::read_to_string(&cargo).expect("Failed to read the Cargo.toml file");
            Self::extract_crate_name(&content)
                .expect("Failed to extract crate name from the Cargo.toml file")
                .replace('-', "_")
        } else {
            // file name
            package_dir
                .as_ref()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
                .replace('-', "_")
        };

        let mut path = PathBuf::from_str(&get_cargo_target_directory(&cargo)).unwrap();
        path.push("wasm32-unknown-unknown");
        path.push("release");
        path.push(&wasm_name);
        path.set_extension("wasm");

        #[cfg(feature = "coverage")]
        // Check if binary exists in coverage directory, if it doesn't only then build it
        {
            let mut coverage_path = PathBuf::from_str(&get_cargo_target_directory(&cargo)).unwrap();
            coverage_path.pop();
            coverage_path.push("coverage");
            coverage_path.push("wasm32-unknown-unknown");
            coverage_path.push("release");
            coverage_path.push(wasm_name);
            coverage_path.set_extension("wasm");
            if coverage_path.is_file() {
                let code = fs::read(&coverage_path).unwrap_or_else(|err| {
                    panic!(
                        "Failed to read built WASM from path {:?} - {:?}",
                        &path, err
                    )
                });
                coverage_path.set_extension("rpd");
                let definition = fs::read(&coverage_path).unwrap_or_else(|err| {
                    panic!(
                        "Failed to read package definition from path {:?} - {:?}",
                        &coverage_path, err
                    )
                });
                let definition = manifest_decode(&definition).unwrap_or_else(|err| {
                    panic!(
                        "Failed to parse package definition from path {:?} - {:?}",
                        &coverage_path, err
                    )
                });
                return (code, definition);
            }
        };

        // Build
        let features = vec![
            "scrypto/log-error",
            "scrypto/log-warn",
            "scrypto/log-info",
            "scrypto/log-debug",
            "scrypto/log-trace",
        ];

        let status = Command::new("cargo")
            .envs(env_vars)
            .current_dir(package_dir.as_ref())
            .args([
                "build",
                "--target",
                "wasm32-unknown-unknown",
                "--release",
                "--features",
                &features.join(","),
            ])
            .status()
            .unwrap_or_else(|error| {
                panic!(
                    "Compiling \"{:?}\" failed with \"{:?}\"",
                    package_dir.as_ref(),
                    error
                )
            });
        if !status.success() {
            panic!("Failed to compile package: {:?}", package_dir.as_ref());
        }

        // Extract schema
        let code = fs::read(&path).unwrap_or_else(|err| {
            panic!(
                "Failed to read built WASM from path {:?} - {:?}",
                &path, err
            )
        });
        let definition = extract_definition(&code).unwrap();

        (code, definition)
    }

    // Naive pattern matching to find the crate name.
    fn extract_crate_name(mut content: &str) -> Result<String, ()> {
        let idx = content.find("name").ok_or(())?;
        content = &content[idx + 4..];

        let idx = content.find('"').ok_or(())?;
        content = &content[idx + 1..];

        let end = content.find('"').ok_or(())?;
        Ok(content[..end].to_string())
    }
}

pub struct CustomGenesis {
    pub genesis_data_chunks: Vec<GenesisDataChunk>,
    pub genesis_epoch: Epoch,
    pub initial_config: ConsensusManagerConfig,
    pub initial_time_ms: i64,
    pub initial_current_leader: Option<ValidatorIndex>,
    pub faucet_supply: Decimal,
}

impl CustomGenesis {
    pub fn default(genesis_epoch: Epoch, initial_config: ConsensusManagerConfig) -> CustomGenesis {
        let pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
        Self::single_validator_and_staker(
            pub_key,
            Decimal::one(),
            Decimal::zero(),
            ComponentAddress::virtual_account_from_public_key(&pub_key),
            genesis_epoch,
            initial_config,
        )
    }

    pub fn with_faucet_supply(faucet_supply: Decimal) -> CustomGenesis {
        CustomGenesis {
            genesis_data_chunks: vec![],
            genesis_epoch: Epoch::of(1u64),
            initial_config: Self::default_consensus_manager_config(),
            initial_time_ms: 0,
            initial_current_leader: None,
            faucet_supply,
        }
    }

    pub fn default_with_xrd_amount(
        xrd_amount: Decimal,
        genesis_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
    ) -> CustomGenesis {
        let pub_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
        Self::single_validator_and_staker(
            pub_key,
            Decimal::one(),
            xrd_amount,
            ComponentAddress::virtual_account_from_public_key(&pub_key),
            genesis_epoch,
            initial_config,
        )
    }

    pub fn default_consensus_manager_config() -> ConsensusManagerConfig {
        ConsensusManagerConfig {
            max_validators: 10,
            epoch_change_condition: EpochChangeCondition {
                min_round_count: 1,
                max_round_count: 1,
                target_duration_millis: 0,
            },
            num_unstake_epochs: 1,
            total_emission_xrd_per_epoch: Decimal::one(),
            min_validator_reliability: Decimal::one(),
            num_owner_stake_units_unlock_epochs: 2,
            num_fee_increase_delay_epochs: 4,
            validator_creation_usd_cost: *DEFAULT_VALIDATOR_USD_COST,
        }
    }

    pub fn single_validator_and_staker(
        validator_public_key: Secp256k1PublicKey,
        stake_xrd_amount: Decimal,
        xrd_amount: Decimal,
        staker_account: ComponentAddress,
        genesis_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
    ) -> CustomGenesis {
        Self::validators_and_single_staker(
            vec![(validator_public_key, stake_xrd_amount)],
            staker_account,
            xrd_amount,
            genesis_epoch,
            initial_config,
        )
    }

    pub fn validators_and_single_staker(
        validators_and_stakes: Vec<(Secp256k1PublicKey, Decimal)>,
        staker_account: ComponentAddress,
        stacker_account_xrd_amount: Decimal,
        genesis_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
    ) -> CustomGenesis {
        let genesis_validators: Vec<GenesisValidator> = validators_and_stakes
            .iter()
            .map(|(key, _)| key.clone().into())
            .collect();
        let stake_allocations: Vec<(Secp256k1PublicKey, Vec<GenesisStakeAllocation>)> =
            validators_and_stakes
                .into_iter()
                .map(|(key, stake_xrd_amount)| {
                    (
                        key,
                        vec![GenesisStakeAllocation {
                            account_index: 0,
                            xrd_amount: stake_xrd_amount,
                        }],
                    )
                })
                .collect();
        let genesis_data_chunks = vec![
            GenesisDataChunk::Validators(genesis_validators),
            GenesisDataChunk::Stakes {
                accounts: vec![staker_account],
                allocations: stake_allocations,
            },
            GenesisDataChunk::ResourceBalances {
                accounts: vec![staker_account],
                allocations: vec![(
                    XRD,
                    vec![GenesisResourceAllocation {
                        account_index: 0u32,
                        amount: stacker_account_xrd_amount,
                    }],
                )],
            },
        ];
        CustomGenesis {
            genesis_data_chunks,
            genesis_epoch,
            initial_config,
            initial_time_ms: 0,
            initial_current_leader: Some(0),
            faucet_supply: *DEFAULT_TESTING_FAUCET_SUPPLY,
        }
    }
}

pub trait TestDatabase:
    SubstateDatabase + CommittableSubstateDatabase + ListableSubstateDatabase
{
}
impl<T: SubstateDatabase + CommittableSubstateDatabase + ListableSubstateDatabase> TestDatabase
    for T
{
}

pub type DefaultTestRunner = TestRunner<NoExtension, InMemorySubstateDatabase>;

pub struct TestRunnerBuilder<E, D> {
    custom_genesis: Option<CustomGenesis>,
    custom_extension: E,
    custom_database: D,
    trace: bool,
    skip_receipt_check: bool,

    // The following are protocol updates on mainnet
    with_seconds_precision_update: bool,
    with_crypto_utils_update: bool,
}

impl TestRunnerBuilder<NoExtension, InMemorySubstateDatabase> {
    pub fn new() -> Self {
        TestRunnerBuilder {
            custom_genesis: None,
            custom_extension: NoExtension,
            custom_database: InMemorySubstateDatabase::standard(),
            trace: true,
            skip_receipt_check: false,
            with_seconds_precision_update: true,
            with_crypto_utils_update: true,
        }
    }
}

impl<E: NativeVmExtension, D: TestDatabase> TestRunnerBuilder<E, D> {
    pub fn without_trace(mut self) -> Self {
        self.trace = false;
        self
    }

    pub fn with_state_hashing(self) -> TestRunnerBuilder<E, HashTreeUpdatingDatabase<D>> {
        TestRunnerBuilder {
            custom_genesis: self.custom_genesis,
            custom_extension: self.custom_extension,
            custom_database: HashTreeUpdatingDatabase::new(self.custom_database),
            trace: self.trace,
            skip_receipt_check: false,
            with_seconds_precision_update: self.with_seconds_precision_update,
            with_crypto_utils_update: self.with_crypto_utils_update,
        }
    }

    pub fn with_custom_genesis(mut self, genesis: CustomGenesis) -> Self {
        self.custom_genesis = Some(genesis);
        self
    }

    pub fn skip_receipt_check(mut self) -> Self {
        self.skip_receipt_check = true;
        self
    }

    pub fn with_custom_extension<NE: NativeVmExtension>(
        self,
        extension: NE,
    ) -> TestRunnerBuilder<NE, D> {
        TestRunnerBuilder::<NE, D> {
            custom_genesis: self.custom_genesis,
            custom_extension: extension,
            custom_database: self.custom_database,
            trace: self.trace,
            skip_receipt_check: self.skip_receipt_check,
            with_seconds_precision_update: self.with_seconds_precision_update,
            with_crypto_utils_update: self.with_crypto_utils_update,
        }
    }

    pub fn with_custom_database<ND: TestDatabase>(self, database: ND) -> TestRunnerBuilder<E, ND> {
        TestRunnerBuilder::<E, ND> {
            custom_genesis: self.custom_genesis,
            custom_extension: self.custom_extension,
            custom_database: database,
            trace: self.trace,
            skip_receipt_check: self.skip_receipt_check,
            with_seconds_precision_update: self.with_seconds_precision_update,
            with_crypto_utils_update: self.with_crypto_utils_update,
        }
    }

    pub fn without_seconds_precision_update(mut self) -> Self {
        self.with_seconds_precision_update = false;
        self
    }

    pub fn without_crypto_utils_update(mut self) -> Self {
        self.with_crypto_utils_update = false;
        self
    }

    pub fn build_from_snapshot(
        self,
        snapshot: TestRunnerSnapshot,
    ) -> TestRunner<E, InMemorySubstateDatabase> {
        //---------- Override configs for resource tracker ---------------
        #[cfg(not(feature = "resource_tracker"))]
        let trace = self.trace;
        #[cfg(feature = "resource_tracker")]
        let trace = false;
        //----------------------------------------------------------------

        TestRunner {
            scrypto_vm: ScryptoVm::default(),
            native_vm: NativeVm::new_with_extension(self.custom_extension),
            database: snapshot.database,
            next_private_key: snapshot.next_private_key,
            next_transaction_nonce: snapshot.next_transaction_nonce,
            trace,
            collected_events: snapshot.collected_events,
            xrd_free_credits_used: snapshot.xrd_free_credits_used,
            skip_receipt_check: snapshot.skip_receipt_check,
        }
    }

    pub fn build_and_get_epoch(self) -> (TestRunner<E, D>, ActiveValidatorSet) {
        //---------- Override configs for resource tracker ---------------
        let bootstrap_trace = false;

        #[cfg(not(feature = "resource_tracker"))]
        let trace = self.trace;
        #[cfg(feature = "resource_tracker")]
        let trace = false;
        //----------------------------------------------------------------

        let scrypto_vm = ScryptoVm {
            wasm_engine: DefaultWasmEngine::default(),
            wasm_validator_config: WasmValidatorConfigV1::new(),
        };
        let native_vm = NativeVm::new_with_extension(self.custom_extension);
        let vm = Vm::new(&scrypto_vm, native_vm.clone());
        let mut substate_db = self.custom_database;
        let mut bootstrapper = Bootstrapper::new(
            NetworkDefinition::simulator(),
            &mut substate_db,
            vm,
            bootstrap_trace,
        );
        let GenesisReceipts {
            system_bootstrap_receipt,
            data_ingestion_receipts,
            wrap_up_receipt,
        } = match self.custom_genesis {
            Some(custom_genesis) => bootstrapper
                .bootstrap_with_genesis_data(
                    custom_genesis.genesis_data_chunks,
                    custom_genesis.genesis_epoch,
                    custom_genesis.initial_config,
                    custom_genesis.initial_time_ms,
                    custom_genesis.initial_current_leader,
                    custom_genesis.faucet_supply,
                )
                .unwrap(),
            None => bootstrapper.bootstrap_test_default().unwrap(),
        };

        let mut events = Vec::new();

        events.push(
            system_bootstrap_receipt
                .expect_commit_success()
                .application_events
                .clone(),
        );
        for receipt in data_ingestion_receipts {
            events.push(receipt.expect_commit_success().application_events.clone());
        }
        events.push(
            wrap_up_receipt
                .expect_commit_success()
                .application_events
                .clone(),
        );

        // Note that 0 is not a valid private key
        let next_private_key = 100;

        // Starting from non-zero considering that bootstrap might have used a few.
        let next_transaction_nonce = 100;

        if self.with_seconds_precision_update {
            let state_updates = generate_seconds_precision_state_updates(&substate_db);
            let db_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
            substate_db.commit(&db_updates);
        };

        if self.with_crypto_utils_update {
            let state_updates = generate_vm_boot_scrypto_minor_version_state_updates();
            let db_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
            substate_db.commit(&db_updates);
        }

        let runner = TestRunner {
            scrypto_vm,
            native_vm,
            database: substate_db,
            next_private_key,
            next_transaction_nonce,
            trace,
            collected_events: events,
            xrd_free_credits_used: false,
            skip_receipt_check: self.skip_receipt_check,
        };

        let next_epoch = wrap_up_receipt
            .expect_commit_success()
            .next_epoch()
            .unwrap();
        (runner, next_epoch.validator_set)
    }

    pub fn build(self) -> TestRunner<E, D> {
        self.build_and_get_epoch().0
    }
}

pub struct TestRunner<E: NativeVmExtension, D: TestDatabase> {
    scrypto_vm: ScryptoVm<DefaultWasmEngine>,
    native_vm: NativeVm<E>,
    database: D,
    next_private_key: u64,
    next_transaction_nonce: u32,
    trace: bool,
    collected_events: Vec<Vec<(EventTypeIdentifier, Vec<u8>)>>,
    xrd_free_credits_used: bool,
    skip_receipt_check: bool,
}

#[cfg(feature = "post_run_db_check")]
impl<E: NativeVmExtension, D: TestDatabase> Drop for TestRunner<E, D> {
    fn drop(&mut self) {
        self.check_database()
    }
}

#[derive(Clone)]
pub struct TestRunnerSnapshot {
    database: InMemorySubstateDatabase,
    next_private_key: u64,
    next_transaction_nonce: u32,
    collected_events: Vec<Vec<(EventTypeIdentifier, Vec<u8>)>>,
    xrd_free_credits_used: bool,
    skip_receipt_check: bool,
}

impl<E: NativeVmExtension> TestRunner<E, InMemorySubstateDatabase> {
    pub fn create_snapshot(&self) -> TestRunnerSnapshot {
        TestRunnerSnapshot {
            database: self.database.clone(),
            next_private_key: self.next_private_key,
            next_transaction_nonce: self.next_transaction_nonce,
            collected_events: self.collected_events.clone(),
            xrd_free_credits_used: self.xrd_free_credits_used,
            skip_receipt_check: self.skip_receipt_check,
        }
    }

    pub fn restore_snapshot(&mut self, snapshot: TestRunnerSnapshot) {
        self.database = snapshot.database;
        self.next_private_key = snapshot.next_private_key;
        self.next_transaction_nonce = snapshot.next_transaction_nonce;
        self.collected_events = snapshot.collected_events;
        self.xrd_free_credits_used = snapshot.xrd_free_credits_used;
        self.skip_receipt_check = snapshot.skip_receipt_check;
    }
}

impl<E: NativeVmExtension, D: TestDatabase> TestRunner<E, D> {
    pub fn faucet_component(&self) -> GlobalAddress {
        FAUCET.clone().into()
    }

    pub fn substate_db(&self) -> &D {
        &self.database
    }

    pub fn substate_db_mut(&mut self) -> &mut D {
        &mut self.database
    }

    pub fn collected_events(&self) -> &Vec<Vec<(EventTypeIdentifier, Vec<u8>)>> {
        self.collected_events.as_ref()
    }

    pub fn next_private_key(&mut self) -> u64 {
        self.next_private_key += 1;
        self.next_private_key - 1
    }

    pub fn next_transaction_nonce(&mut self) -> u32 {
        self.next_transaction_nonce += 1;
        self.next_transaction_nonce - 1
    }

    pub fn new_key_pair(&mut self) -> (Secp256k1PublicKey, Secp256k1PrivateKey) {
        let private_key = Secp256k1PrivateKey::from_u64(self.next_private_key()).unwrap();
        let public_key = private_key.public_key();

        (public_key, private_key)
    }

    pub fn new_ed25519_key_pair(&mut self) -> (Ed25519PublicKey, Ed25519PrivateKey) {
        let private_key = Ed25519PrivateKey::from_u64(self.next_private_key()).unwrap();
        let public_key = private_key.public_key();

        (public_key, private_key)
    }

    pub fn new_key_pair_with_auth_address(
        &mut self,
    ) -> (Secp256k1PublicKey, Secp256k1PrivateKey, NonFungibleGlobalId) {
        let key_pair = self.new_allocated_account();
        (
            key_pair.0,
            key_pair.1,
            NonFungibleGlobalId::from_public_key(&key_pair.0),
        )
    }

    pub fn set_metadata(
        &mut self,
        address: GlobalAddress,
        key: &str,
        value: &str,
        proof: NonFungibleGlobalId,
    ) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .set_metadata(
                address,
                key.to_string(),
                MetadataValue::String(value.to_string()),
            )
            .build();

        let receipt = self.execute_manifest(manifest, vec![proof]);
        receipt.expect_commit_success();
    }

    pub fn get_metadata(&mut self, address: GlobalAddress, key: &str) -> Option<MetadataValue> {
        let reader = SystemDatabaseReader::new(self.substate_db());
        reader
            .read_object_collection_entry::<_, MetadataEntryEntryPayload>(
                address.as_node_id(),
                ModuleId::Metadata,
                ObjectCollectionKey::KeyValue(
                    MetadataCollection::EntryKeyValue.collection_index(),
                    &key.to_string(),
                ),
            )
            .unwrap()
            .map(|v| v.into_latest())
    }

    pub fn inspect_component_royalty(&mut self, component_address: ComponentAddress) -> Decimal {
        let reader = SystemDatabaseReader::new(self.substate_db());
        let accumulator = reader
            .read_typed_object_field::<ComponentRoyaltyAccumulatorFieldPayload>(
                component_address.as_node_id(),
                ModuleId::Royalty,
                ComponentRoyaltyField::Accumulator.field_index(),
            )
            .unwrap()
            .into_latest();

        let balance = reader
            .read_typed_object_field::<FungibleVaultBalanceFieldPayload>(
                accumulator.royalty_vault.0.as_node_id(),
                ModuleId::Main,
                FungibleVaultField::Balance.field_index(),
            )
            .unwrap()
            .into_latest();

        balance.amount()
    }

    pub fn inspect_package_royalty(&mut self, package_address: PackageAddress) -> Option<Decimal> {
        let reader = SystemDatabaseReader::new(self.substate_db());
        let accumulator = reader
            .read_typed_object_field::<PackageRoyaltyAccumulatorFieldPayload>(
                package_address.as_node_id(),
                ModuleId::Main,
                PackageField::RoyaltyAccumulator.field_index(),
            )
            .ok()?
            .into_latest();

        let balance = reader
            .read_typed_object_field::<FungibleVaultBalanceFieldPayload>(
                accumulator.royalty_vault.0.as_node_id(),
                ModuleId::Main,
                FungibleVaultField::Balance.field_index(),
            )
            .unwrap()
            .into_latest();

        Some(balance.amount())
    }

    pub fn find_all_nodes(&self) -> IndexSet<NodeId> {
        let mut node_ids = index_set_new();
        for pk in self.database.list_partition_keys() {
            let (node_id, _) = SpreadPrefixKeyMapper::from_db_partition_key(&pk);
            node_ids.insert(node_id);
        }
        node_ids
    }

    pub fn find_all_components(&self) -> Vec<ComponentAddress> {
        let mut addresses: Vec<ComponentAddress> = self
            .find_all_nodes()
            .iter()
            .filter_map(|node_id| ComponentAddress::try_from(node_id.as_bytes()).ok())
            .collect();
        addresses.sort();
        addresses
    }

    pub fn find_all_packages(&self) -> Vec<PackageAddress> {
        let mut addresses: Vec<PackageAddress> = self
            .find_all_nodes()
            .iter()
            .filter_map(|node_id| PackageAddress::try_from(node_id.as_bytes()).ok())
            .collect();
        addresses.sort();
        addresses
    }

    pub fn find_all_resources(&self) -> Vec<ResourceAddress> {
        let mut addresses: Vec<ResourceAddress> = self
            .find_all_nodes()
            .iter()
            .filter_map(|node_id| ResourceAddress::try_from(node_id.as_bytes()).ok())
            .collect();
        addresses.sort();
        addresses
    }

    pub fn get_package_scrypto_schemas(
        &self,
        package_address: &PackageAddress,
    ) -> IndexMap<SchemaHash, VersionedScryptoSchema> {
        let reader = SystemDatabaseReader::new(self.substate_db());
        reader
            .collection_iter(
                package_address.as_node_id(),
                ModuleId::Main,
                PackageCollection::SchemaKeyValue.collection_index(),
            )
            .unwrap()
            .map(|(key, value)| {
                let key = key.into_map();
                let hash: SchemaHash = scrypto_decode(&key).unwrap();
                let schema: PackageSchemaEntryPayload = scrypto_decode(&value).unwrap();
                (hash, schema.content)
            })
            .collect()
    }

    pub fn get_package_blueprint_definitions(
        &self,
        package_address: &PackageAddress,
    ) -> IndexMap<BlueprintVersionKey, BlueprintDefinition> {
        let reader = SystemDatabaseReader::new(self.substate_db());
        reader
            .collection_iter(
                package_address.as_node_id(),
                ModuleId::Main,
                PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
            )
            .unwrap()
            .map(|(key, value)| {
                let map_key = key.into_map();
                let key: BlueprintVersionKey = scrypto_decode(&map_key).unwrap();
                let definition: PackageBlueprintVersionDefinitionEntryPayload =
                    scrypto_decode(&value).unwrap();
                (key, definition.into_latest())
            })
            .collect()
    }

    pub fn sum_descendant_balance_changes(
        &mut self,
        commit: &CommitResult,
        node_id: &NodeId,
    ) -> IndexMap<ResourceAddress, BalanceChange> {
        SubtreeVaults::new(&self.database)
            .sum_balance_changes(node_id, commit.vault_balance_changes())
    }

    pub fn get_component_vaults(
        &mut self,
        component_address: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> Vec<NodeId> {
        SubtreeVaults::new(&self.database)
            .get_all(component_address.as_node_id())
            .remove(&resource_address)
            .unwrap_or_else(|| Vec::new())
    }

    pub fn get_component_balance(
        &mut self,
        account_address: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> Decimal {
        let vaults = self.get_component_vaults(account_address, resource_address);
        let mut sum = Decimal::ZERO;
        for vault in vaults {
            sum = sum
                .checked_add(self.inspect_vault_balance(vault).unwrap())
                .unwrap();
        }
        sum
    }

    pub fn inspect_vault_balance(&mut self, vault_id: NodeId) -> Option<Decimal> {
        if vault_id.is_internal_fungible_vault() {
            self.inspect_fungible_vault(vault_id)
        } else {
            self.inspect_non_fungible_vault(vault_id)
                .map(|(amount, ..)| amount)
        }
    }

    pub fn inspect_fungible_vault(&mut self, vault_id: NodeId) -> Option<Decimal> {
        let reader = SystemDatabaseReader::new(self.substate_db());
        let vault: Option<FungibleVaultBalanceFieldPayload> = reader
            .read_typed_object_field(
                &vault_id,
                ModuleId::Main,
                FungibleVaultField::Balance.into(),
            )
            .ok();

        vault.map(|v| v.into_latest().amount())
    }

    pub fn inspect_non_fungible_vault(
        &mut self,
        vault_id: NodeId,
    ) -> Option<(Decimal, Box<dyn Iterator<Item = NonFungibleLocalId> + '_>)> {
        let reader = SystemDatabaseReader::new(self.substate_db());
        let vault_balance: NonFungibleVaultBalanceFieldPayload = reader
            .read_typed_object_field(
                &vault_id,
                ModuleId::Main,
                NonFungibleVaultField::Balance.into(),
            )
            .ok()?;
        let amount = vault_balance.into_latest().amount;

        // TODO: Remove .collect() by using SystemDatabaseReader in test_runner
        let iter: Vec<NonFungibleLocalId> = reader
            .collection_iter(
                &vault_id,
                ModuleId::Main,
                NonFungibleVaultCollection::NonFungibleIndex.collection_index(),
            )
            .unwrap()
            .map(|(key, _)| {
                let map_key = key.into_map();
                let id: NonFungibleLocalId = scrypto_decode(&map_key).unwrap();
                id
            })
            .collect();

        Some((amount, Box::new(iter.into_iter())))
    }

    pub fn get_component_resources(
        &mut self,
        component_address: ComponentAddress,
    ) -> HashMap<ResourceAddress, Decimal> {
        let node_id = component_address.as_node_id();
        let mut accounter = ResourceAccounter::new(&self.database);
        accounter.traverse(node_id.clone());
        accounter.close().balances
    }

    pub fn component_state<T: ScryptoDecode>(&self, component_address: ComponentAddress) -> T {
        let node_id: &NodeId = component_address.as_node_id();
        let component_state = self
            .substate_db()
            .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<T>>(
                node_id,
                MAIN_BASE_PARTITION,
                &ComponentField::State0.into(),
            );
        component_state.unwrap().into_payload()
    }

    pub fn get_non_fungible_data<T: NonFungibleData>(
        &self,
        resource: ResourceAddress,
        non_fungible_id: NonFungibleLocalId,
    ) -> T {
        let reader = SystemDatabaseReader::new(self.substate_db());
        let payload = reader
            .read_object_collection_entry::<_, NonFungibleResourceManagerDataEntryPayload>(
                resource.as_node_id(),
                ModuleId::Main,
                ObjectCollectionKey::KeyValue(
                    NonFungibleResourceManagerCollection::DataKeyValue.collection_index(),
                    &non_fungible_id,
                ),
            )
            .unwrap()
            .unwrap();

        scrypto_decode(&scrypto_encode(&payload.content).unwrap()).unwrap()
    }

    pub fn get_kv_store_entry<K: ScryptoEncode, V: ScryptoEncode + ScryptoDecode>(
        &self,
        kv_store_id: Own,
        key: &K,
    ) -> Option<V> {
        let reader = SystemDatabaseReader::new(self.substate_db());
        reader.read_typed_kv_entry(kv_store_id.as_node_id(), key)
    }

    pub fn load_account_from_faucet(&mut self, account_address: ComponentAddress) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .take_all_from_worktop(XRD, "free_xrd")
            .try_deposit_or_abort(account_address, None, "free_xrd")
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
    }

    pub fn new_account_advanced(&mut self, owner_role: OwnerRole) -> ComponentAddress {
        let manifest = ManifestBuilder::new()
            .new_account_advanced(owner_role, None)
            .build();
        let receipt = self.execute_manifest_ignoring_fee(manifest, vec![]);
        receipt.expect_commit_success();

        let account = receipt.expect_commit(true).new_component_addresses()[0];

        let manifest = ManifestBuilder::new()
            .get_free_xrd_from_faucet()
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest_ignoring_fee(manifest, vec![]);
        receipt.expect_commit_success();

        account
    }

    pub fn new_virtual_account(
        &mut self,
    ) -> (Secp256k1PublicKey, Secp256k1PrivateKey, ComponentAddress) {
        let (pub_key, priv_key) = self.new_key_pair();
        let account = ComponentAddress::virtual_account_from_public_key(&PublicKey::Secp256k1(
            pub_key.clone(),
        ));
        self.load_account_from_faucet(account);
        (pub_key, priv_key, account)
    }

    pub fn new_ed25519_virtual_account(
        &mut self,
    ) -> (Ed25519PublicKey, Ed25519PrivateKey, ComponentAddress) {
        let (pub_key, priv_key) = self.new_ed25519_key_pair();
        let account =
            ComponentAddress::virtual_account_from_public_key(&PublicKey::Ed25519(pub_key.clone()));
        self.load_account_from_faucet(account);
        (pub_key, priv_key, account)
    }

    pub fn get_active_validator_info_by_key(&self, key: &Secp256k1PublicKey) -> ValidatorSubstate {
        let address = self.get_active_validator_with_key(key);
        self.get_validator_info(address)
    }

    pub fn get_validator_info(&self, address: ComponentAddress) -> ValidatorSubstate {
        let reader = SystemDatabaseReader::new(&self.database);
        let substate = reader
            .read_typed_object_field::<ValidatorStateFieldPayload>(
                address.as_node_id(),
                ModuleId::Main,
                ValidatorField::State.field_index(),
            )
            .unwrap()
            .into_latest();

        substate
    }

    pub fn get_active_validator_with_key(&self, key: &Secp256k1PublicKey) -> ComponentAddress {
        let reader = SystemDatabaseReader::new(&self.database);
        let substate = reader
            .read_typed_object_field::<ConsensusManagerCurrentValidatorSetFieldPayload>(
                CONSENSUS_MANAGER.as_node_id(),
                ModuleId::Main,
                ConsensusManagerField::CurrentValidatorSet.field_index(),
            )
            .unwrap()
            .into_latest();
        substate
            .validator_set
            .get_by_public_key(key)
            .unwrap()
            .0
            .clone()
    }

    pub fn new_allocated_account(
        &mut self,
    ) -> (Secp256k1PublicKey, Secp256k1PrivateKey, ComponentAddress) {
        let key_pair = self.new_key_pair();
        let withdraw_auth = rule!(require(NonFungibleGlobalId::from_public_key(&key_pair.0)));
        let account = self.new_account_advanced(OwnerRole::Fixed(withdraw_auth));
        (key_pair.0, key_pair.1, account)
    }

    pub fn new_ed25519_virtual_account_with_access_controller(
        &mut self,
        n_out_of_4: u8,
    ) -> (
        Ed25519PublicKey,
        Ed25519PrivateKey,
        Ed25519PublicKey,
        Ed25519PrivateKey,
        Ed25519PublicKey,
        Ed25519PrivateKey,
        Ed25519PublicKey,
        Ed25519PrivateKey,
        ComponentAddress,
        ComponentAddress,
    ) {
        let (pk1, sk1, account) = self.new_ed25519_virtual_account();
        let (pk2, sk2) = self.new_ed25519_key_pair();
        let (pk3, sk3) = self.new_ed25519_key_pair();
        let (pk4, sk4) = self.new_ed25519_key_pair();

        let access_rule = AccessRule::Protected(AccessRuleNode::ProofRule(ProofRule::CountOf(
            n_out_of_4,
            vec![
                ResourceOrNonFungible::NonFungible(NonFungibleGlobalId::from_public_key(&pk1)),
                ResourceOrNonFungible::NonFungible(NonFungibleGlobalId::from_public_key(&pk2)),
                ResourceOrNonFungible::NonFungible(NonFungibleGlobalId::from_public_key(&pk3)),
                ResourceOrNonFungible::NonFungible(NonFungibleGlobalId::from_public_key(&pk4)),
            ],
        )));

        let access_controller = self
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(account, ACCOUNT_SECURIFY_IDENT, manifest_args!())
                    .take_all_from_worktop(ACCOUNT_OWNER_BADGE, "owner_badge")
                    .call_function_with_name_lookup(
                        ACCESS_CONTROLLER_PACKAGE,
                        ACCESS_CONTROLLER_BLUEPRINT,
                        ACCESS_CONTROLLER_CREATE_IDENT,
                        |lookup| {
                            (
                                lookup.bucket("owner_badge"),
                                RuleSet {
                                    primary_role: access_rule.clone(),
                                    recovery_role: access_rule.clone(),
                                    confirmation_role: access_rule.clone(),
                                },
                                Some(1000u32),
                                None::<()>,
                            )
                        },
                    )
                    .build(),
                vec![NonFungibleGlobalId::from_public_key(&pk1)],
            )
            .expect_commit_success()
            .new_component_addresses()[0]
            .clone();

        (
            pk1,
            sk1,
            pk2,
            sk2,
            pk3,
            sk3,
            pk4,
            sk4,
            account,
            access_controller,
        )
    }

    pub fn new_account(
        &mut self,
        is_virtual: bool,
    ) -> (Secp256k1PublicKey, Secp256k1PrivateKey, ComponentAddress) {
        if is_virtual {
            self.new_virtual_account()
        } else {
            self.new_allocated_account()
        }
    }

    pub fn new_identity<P: Into<PublicKey> + Clone + HasPublicKeyHash>(
        &mut self,
        pk: P,
        is_virtual: bool,
    ) -> ComponentAddress {
        if is_virtual {
            ComponentAddress::virtual_identity_from_public_key(&pk)
        } else {
            let owner_id = NonFungibleGlobalId::from_public_key(&pk);
            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_identity_advanced(OwnerRole::Fixed(rule!(require(owner_id))))
                .build();
            let receipt = self.execute_manifest(manifest, vec![]);
            receipt.expect_commit_success();
            let component_address = receipt.expect_commit(true).new_component_addresses()[0];

            component_address
        }
    }

    pub fn new_securified_identity(&mut self, account: ComponentAddress) -> ComponentAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_identity()
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
        let component_address = receipt.expect_commit(true).new_component_addresses()[0];

        component_address
    }

    pub fn new_validator_with_pub_key(
        &mut self,
        pub_key: Secp256k1PublicKey,
        account: ComponentAddress,
    ) -> ComponentAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .take_from_worktop(XRD, *DEFAULT_VALIDATOR_XRD_COST, "xrd_creation_fee")
            .create_validator(pub_key, Decimal::ONE, "xrd_creation_fee")
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        let address = receipt.expect_commit(true).new_component_addresses()[0];
        address
    }

    pub fn new_staked_validator_with_pub_key(
        &mut self,
        pub_key: Secp256k1PublicKey,
        account: ComponentAddress,
    ) -> ComponentAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .take_from_worktop(XRD, *DEFAULT_VALIDATOR_XRD_COST, "xrd_creation_fee")
            .create_validator(pub_key, Decimal::ONE, "xrd_creation_fee")
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        let validator_address = receipt.expect_commit(true).new_component_addresses()[0];

        let receipt = self.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .create_proof_from_account_of_non_fungibles(
                    account,
                    VALIDATOR_OWNER_BADGE,
                    [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
                )
                .take_all_from_worktop(XRD, "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        validator_address,
                        VALIDATOR_STAKE_AS_OWNER_IDENT,
                        manifest_args!(bucket),
                    )
                })
                .deposit_batch(account)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pub_key)],
        );
        receipt.expect_commit_success();

        validator_address
    }

    pub fn publish_native_package(
        &mut self,
        native_package_code_id: u64,
        definition: PackageDefinition,
    ) -> PackageAddress {
        let receipt = self.execute_system_transaction(
            vec![InstructionV1::CallFunction {
                package_address: DynamicPackageAddress::Static(PACKAGE_PACKAGE),
                blueprint_name: PACKAGE_BLUEPRINT.to_string(),
                function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                    definition,
                    native_package_code_id,
                    metadata: MetadataInit::default(),
                    package_address: None,
                }),
            }],
            btreeset!(AuthAddresses::system_role()),
        );
        let package_address: PackageAddress = receipt.expect_commit(true).output(0);
        package_address
    }

    pub fn publish_package_at_address<P: Into<PackagePublishingSource>>(
        &mut self,
        source: P,
        address: PackageAddress,
    ) {
        let (code, definition) = source.into().code_and_definition();
        let code_hash = hash(&code);
        let nonce = self.next_transaction_nonce();

        let receipt = self.execute_transaction(
            SystemTransactionV1 {
                instructions: InstructionsV1(vec![InstructionV1::CallFunction {
                    package_address: PACKAGE_PACKAGE.into(),
                    blueprint_name: PACKAGE_BLUEPRINT.to_string(),
                    function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
                    args: to_manifest_value_and_unwrap!(&PackagePublishWasmAdvancedManifestInput {
                        code: ManifestBlobRef(code_hash.0),
                        definition: definition,
                        metadata: metadata_init!(),
                        package_address: Some(ManifestAddressReservation(0)),
                        owner_role: OwnerRole::Fixed(AccessRule::AllowAll),
                    }),
                }]),
                blobs: BlobsV1 {
                    blobs: vec![BlobV1(code)],
                },
                hash_for_execution: hash(format!("Test runner txn: {}", nonce)),
                pre_allocated_addresses: vec![PreAllocatedAddress {
                    blueprint_id: BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
                    address: address.into(),
                }],
            }
            .prepare()
            .expect("expected transaction to be preparable")
            .get_executable(btreeset!(AuthAddresses::system_role())),
            CostingParameters::default(),
            ExecutionConfig::for_system_transaction(NetworkDefinition::simulator()),
        );

        receipt.expect_commit_success();
    }

    pub fn publish_package<P: Into<PackagePublishingSource>>(
        &mut self,
        source: P,
        metadata: BTreeMap<String, MetadataValue>,
        owner_role: OwnerRole,
    ) -> PackageAddress {
        let (code, definition) = source.into().code_and_definition();
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .publish_package_advanced(None, code, definition, metadata, owner_role)
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_package_addresses()[0]
    }

    pub fn try_publish_package<P: Into<PackagePublishingSource>>(
        &mut self,
        source: P,
    ) -> TransactionReceipt {
        let (code, definition) = source.into().code_and_definition();
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .publish_package_advanced(None, code, definition, BTreeMap::new(), OwnerRole::None)
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt
    }

    pub fn publish_package_simple<P: Into<PackagePublishingSource>>(
        &mut self,
        source: P,
    ) -> PackageAddress {
        self.publish_package(source, Default::default(), OwnerRole::None)
    }

    pub fn publish_package_with_owner<P: Into<PackagePublishingSource>>(
        &mut self,
        source: P,
        owner_badge: NonFungibleGlobalId,
    ) -> PackageAddress {
        let (code, definition) = source.into().code_and_definition();
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .publish_package_with_owner(code, definition, owner_badge)
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_package_addresses()[0]
    }

    pub fn compile<P: AsRef<Path>>(&mut self, package_dir: P) -> (Vec<u8>, PackageDefinition) {
        Compile::compile(package_dir)
    }

    // Doesn't need to be here - kept for backward compatibility
    pub fn compile_and_publish<P: AsRef<Path>>(&mut self, package_dir: P) -> PackageAddress {
        self.publish_package(package_dir.as_ref(), BTreeMap::new(), OwnerRole::None)
    }

    // Doesn't need to be here - kept for backward compatibility
    pub fn compile_and_publish_at_address<P: AsRef<Path>>(
        &mut self,
        package_dir: P,
        address: PackageAddress,
    ) {
        self.publish_package_at_address(package_dir.as_ref(), address);
    }

    pub fn publish_retain_blueprints<
        P: Into<PackagePublishingSource>,
        F: FnMut(&String, &mut BlueprintDefinitionInit) -> bool,
    >(
        &mut self,
        source: P,
        retain: F,
    ) -> PackageAddress {
        let (code, mut definition) =
            Into::<PackagePublishingSource>::into(source).code_and_definition();
        definition.blueprints.retain(retain);
        self.publish_package((code, definition), BTreeMap::new(), OwnerRole::None)
    }

    // Doesn't need to be here - kept for backward compatibility
    pub fn compile_and_publish_with_owner<P: AsRef<Path>>(
        &mut self,
        package_dir: P,
        owner_badge: NonFungibleGlobalId,
    ) -> PackageAddress {
        self.publish_package_with_owner(package_dir.as_ref(), owner_badge)
    }

    pub fn execute_unsigned_built_manifest_with_faucet_lock_fee(
        &mut self,
        create_manifest: impl FnOnce(ManifestBuilder) -> ManifestBuilder,
    ) -> TransactionReceipt {
        self.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .then(create_manifest)
                .build(),
            [],
        )
    }

    pub fn execute_unsigned_built_manifest(
        &mut self,
        create_manifest: impl FnOnce(ManifestBuilder) -> ManifestBuilder,
    ) -> TransactionReceipt {
        self.execute_manifest(ManifestBuilder::new().then(create_manifest).build(), [])
    }

    pub fn execute_built_manifest_with_faucet_lock_fee(
        &mut self,
        create_manifest: impl FnOnce(ManifestBuilder) -> ManifestBuilder,
        signatures: impl ResolvableTransactionSignatures,
    ) -> TransactionReceipt {
        self.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .then(create_manifest)
                .build(),
            signatures.resolve(),
        )
    }

    pub fn execute_built_manifest(
        &mut self,
        create_manifest: impl FnOnce(ManifestBuilder) -> ManifestBuilder,
        signatures: impl ResolvableTransactionSignatures,
    ) -> TransactionReceipt {
        self.execute_manifest(
            ManifestBuilder::new().then(create_manifest).build(),
            signatures.resolve(),
        )
    }

    pub fn execute_manifest_with_fee_from_faucet<T>(
        &mut self,
        mut manifest: TransactionManifestV1,
        amount: Decimal,
        initial_proofs: T,
    ) -> TransactionReceipt
    where
        T: IntoIterator<Item = NonFungibleGlobalId>,
    {
        manifest.instructions.insert(
            0,
            transaction::model::InstructionV1::CallMethod {
                address: self.faucet_component().into(),
                method_name: "lock_fee".to_string(),
                args: manifest_args!(amount).into(),
            },
        );
        self.execute_manifest(manifest, initial_proofs)
    }

    pub fn execute_manifest_with_fee_from_faucet_with_system<
        'a,
        T,
        R: WrappedSystem<Vm<'a, DefaultWasmEngine, E>>,
    >(
        &'a mut self,
        mut manifest: TransactionManifestV1,
        amount: Decimal,
        initial_proofs: T,
        init: R::Init,
    ) -> TransactionReceipt
    where
        T: IntoIterator<Item = NonFungibleGlobalId>,
    {
        manifest.instructions.insert(
            0,
            transaction::model::InstructionV1::CallMethod {
                address: self.faucet_component().into(),
                method_name: "lock_fee".to_string(),
                args: manifest_args!(amount).into(),
            },
        );
        self.execute_manifest_with_system::<'a, T, R>(manifest, initial_proofs, init)
    }

    pub fn execute_manifest_ignoring_fee<T>(
        &mut self,
        mut manifest: TransactionManifestV1,
        initial_proofs: T,
    ) -> TransactionReceipt
    where
        T: IntoIterator<Item = NonFungibleGlobalId>,
    {
        manifest.instructions.insert(
            0,
            transaction::model::InstructionV1::CallMethod {
                address: self.faucet_component().into(),
                method_name: "lock_fee".to_string(),
                args: manifest_args!(dec!("500")).into(),
            },
        );
        self.execute_manifest(manifest, initial_proofs)
    }

    pub fn execute_raw_transaction(
        &mut self,
        network: &NetworkDefinition,
        raw_transaction: &RawNotarizedTransaction,
    ) -> TransactionReceipt {
        let validator = NotarizedTransactionValidator::new(ValidationConfig::default(network.id));
        let validated = validator
            .validate_from_raw(&raw_transaction)
            .expect("Expected raw transaction to be valid");
        self.execute_transaction(
            validated.get_executable(),
            CostingParameters::default(),
            ExecutionConfig::for_notarized_transaction(network.clone()),
        )
    }

    pub fn execute_manifest<T>(
        &mut self,
        manifest: TransactionManifestV1,
        initial_proofs: T,
    ) -> TransactionReceipt
    where
        T: IntoIterator<Item = NonFungibleGlobalId>,
    {
        self.execute_manifest_with_costing_params(
            manifest,
            initial_proofs,
            CostingParameters::default(),
        )
    }

    pub fn execute_manifest_with_system<'a, T, R: WrappedSystem<Vm<'a, DefaultWasmEngine, E>>>(
        &'a mut self,
        manifest: TransactionManifestV1,
        initial_proofs: T,
        init: R::Init,
    ) -> TransactionReceipt
    where
        T: IntoIterator<Item = NonFungibleGlobalId>,
    {
        let nonce = self.next_transaction_nonce();
        self.execute_transaction_with_system::<R>(
            TestTransaction::new_from_nonce(manifest, nonce)
                .prepare()
                .expect("expected transaction to be preparable")
                .get_executable(initial_proofs.into_iter().collect()),
            CostingParameters::default(),
            ExecutionConfig::for_test_transaction(),
            init,
        )
    }

    pub fn execute_manifest_with_costing_params<T>(
        &mut self,
        manifest: TransactionManifestV1,
        initial_proofs: T,
        costing_parameters: CostingParameters,
    ) -> TransactionReceipt
    where
        T: IntoIterator<Item = NonFungibleGlobalId>,
    {
        let nonce = self.next_transaction_nonce();
        self.execute_transaction(
            TestTransaction::new_from_nonce(manifest, nonce)
                .prepare()
                .expect("expected transaction to be preparable")
                .get_executable(initial_proofs.into_iter().collect()),
            costing_parameters,
            ExecutionConfig::for_test_transaction(),
        )
    }

    pub fn execute_manifest_with_execution_cost_unit_limit<T>(
        &mut self,
        manifest: TransactionManifestV1,
        initial_proofs: T,
        execution_cost_unit_limit: u32,
    ) -> TransactionReceipt
    where
        T: IntoIterator<Item = NonFungibleGlobalId>,
    {
        let nonce = self.next_transaction_nonce();
        self.execute_transaction(
            TestTransaction::new_from_nonce(manifest, nonce)
                .prepare()
                .expect("expected transaction to be preparable")
                .get_executable(initial_proofs.into_iter().collect()),
            CostingParameters::default().with_execution_cost_unit_limit(execution_cost_unit_limit),
            ExecutionConfig::for_test_transaction(),
        )
    }

    pub fn execute_transaction(
        &mut self,
        executable: Executable,
        costing_parameters: CostingParameters,
        execution_config: ExecutionConfig,
    ) -> TransactionReceipt {
        self.execute_transaction_with_system::<SystemConfig<Vm<'_, DefaultWasmEngine, E>>>(
            executable,
            costing_parameters,
            execution_config,
            (),
        )
    }

    pub fn execute_transaction_with_system<'a, T: WrappedSystem<Vm<'a, DefaultWasmEngine, E>>>(
        &'a mut self,
        executable: Executable,
        costing_parameters: CostingParameters,
        mut execution_config: ExecutionConfig,
        init: T::Init,
    ) -> TransactionReceipt {
        // Override the kernel trace config
        execution_config = execution_config.with_kernel_trace(self.trace);

        if executable
            .costing_parameters()
            .free_credit_in_xrd
            .is_positive()
        {
            self.xrd_free_credits_used = true;
        }

        let vm = Vm {
            scrypto_vm: &self.scrypto_vm,
            native_vm: self.native_vm.clone(),
        };

        let transaction_receipt = execute_transaction_with_system::<_, _, T>(
            &mut self.database,
            vm,
            &costing_parameters,
            &execution_config,
            &executable,
            init,
        );
        if let TransactionResult::Commit(commit) = &transaction_receipt.result {
            let database_updates = commit
                .state_updates
                .create_database_updates::<SpreadPrefixKeyMapper>();
            self.database.commit(&database_updates);
            self.collected_events
                .push(commit.application_events.clone());

            if !self.skip_receipt_check {
                assert_receipt_substate_changes_can_be_typed(commit);
            }
        }
        transaction_receipt
    }

    pub fn preview(
        &mut self,
        preview_intent: PreviewIntentV1,
        network: &NetworkDefinition,
    ) -> Result<TransactionReceipt, PreviewError> {
        let vm = Vm {
            scrypto_vm: &self.scrypto_vm,
            native_vm: self.native_vm.clone(),
        };

        execute_preview(&self.database, vm, network, preview_intent, self.trace)
    }

    pub fn preview_manifest(
        &mut self,
        manifest: TransactionManifestV1,
        signer_public_keys: Vec<PublicKey>,
        tip_percentage: u16,
        flags: PreviewFlags,
    ) -> TransactionReceipt {
        let epoch = self.get_current_epoch();
        let vm = Vm {
            scrypto_vm: &self.scrypto_vm,
            native_vm: self.native_vm.clone(),
        };
        execute_preview(
            &mut self.database,
            vm,
            &NetworkDefinition::simulator(),
            PreviewIntentV1 {
                intent: IntentV1 {
                    header: TransactionHeaderV1 {
                        network_id: NetworkDefinition::simulator().id,
                        start_epoch_inclusive: epoch,
                        end_epoch_exclusive: epoch.after(10).unwrap(),
                        nonce: 0,
                        notary_public_key: PublicKey::Secp256k1(Secp256k1PublicKey([0u8; 33])),
                        notary_is_signatory: false,
                        tip_percentage,
                    },
                    instructions: InstructionsV1(manifest.instructions),
                    blobs: BlobsV1 {
                        blobs: manifest.blobs.values().map(|x| BlobV1(x.clone())).collect(),
                    },
                    message: MessageV1::default(),
                },
                signer_public_keys,
                flags,
            },
            self.trace,
        )
        .unwrap()
    }

    /// Calls a package blueprint function with the given arguments, paying the fee from the faucet.
    ///
    /// The arguments should be one of:
    /// * A tuple, such as `()`, `(x,)` or `(x, y, z)`
    ///   * IMPORTANT: If calling with a single argument, you must include a trailing comma
    ///     in the tuple declaration. This ensures that the rust compiler knows it's a singleton tuple,
    ///     rather than just some brackets around the inner value.
    /// * A struct which implements `ManifestEncode` representing the arguments
    /// * `manifest_args!(x, y, z)`
    ///
    /// Notes:
    /// * Buckets and signatures are not supported - instead use `execute_manifest_ignoring_fee` and `ManifestBuilder` directly.
    /// * Call `.expect_commit_success()` on the receipt to get access to receipt details.
    pub fn call_function(
        &mut self,
        package_address: impl ResolvablePackageAddress,
        blueprint_name: impl Into<String>,
        function_name: impl Into<String>,
        arguments: impl ResolvableArguments,
    ) -> TransactionReceipt {
        self.execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_function(package_address, blueprint_name, function_name, arguments)
                .build(),
            vec![],
        )
    }

    /// Calls a package blueprint function with the given arguments, and assumes it constructs a single component successfully.
    /// It returns the address of the first created component.
    ///
    /// The arguments should be one of:
    /// * A tuple, such as `()`, `(x,)` or `(x, y, z)`
    ///   * IMPORTANT: If calling with a single argument, you must include a trailing comma
    ///     in the tuple declaration. This ensures that the rust compiler knows it's a singleton tuple,
    ///     rather than just some brackets around the inner value.
    /// * A struct which implements `ManifestEncode` representing the arguments
    /// * `manifest_args!(x, y, z)`
    ///
    /// Notes:
    /// * Buckets and signatures are not supported - instead use `execute_manifest_ignoring_fee` and `ManifestBuilder` directly.
    pub fn construct_new(
        &mut self,
        package_address: impl ResolvablePackageAddress,
        blueprint_name: impl Into<String>,
        function_name: impl Into<String>,
        arguments: impl ResolvableArguments,
    ) -> ComponentAddress {
        self.call_function(package_address, blueprint_name, function_name, arguments)
            .expect_commit_success()
            .new_component_addresses()[0]
    }

    /// Calls a component method with the given arguments, paying the fee from the faucet.
    ///
    /// The arguments should be one of:
    /// * A tuple, such as `()`, `(x,)` or `(x, y, z)`
    ///   * IMPORTANT: If calling with a single argument, you must include a trailing comma
    ///     in the tuple declaration. This ensures that the rust compiler knows it's a singleton tuple,
    ///     rather than just some brackets around the inner value.
    /// * A struct which implements `ManifestEncode` representing the arguments
    /// * `manifest_args!(x, y, z)`
    ///
    /// Notes:
    /// * Buckets and signatures are not supported - instead use `execute_manifest_ignoring_fee` and `ManifestBuilder` directly.
    /// * Call `.expect_commit_success()` on the receipt to get access to receipt details.
    pub fn call_method(
        &mut self,
        address: impl ResolvableGlobalAddress,
        method_name: impl Into<String>,
        args: impl ResolvableArguments,
    ) -> TransactionReceipt {
        self.execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(address, method_name, args)
                .build(),
            vec![],
        )
    }

    fn create_fungible_resource_and_deposit(
        &mut self,
        owner_role: OwnerRole,
        resource_roles: FungibleResourceRoles,
        to: ComponentAddress,
    ) -> ResourceAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                owner_role,
                true,
                0,
                resource_roles,
                metadata!(),
                Some(5.into()),
            )
            .try_deposit_entire_worktop_or_abort(to, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    pub fn create_restricted_token(
        &mut self,
        account: ComponentAddress,
    ) -> (
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
        ResourceAddress,
    ) {
        let mint_auth = self.create_fungible_resource(dec!(1), 0, account);
        let burn_auth = self.create_fungible_resource(dec!(1), 0, account);
        let withdraw_auth = self.create_fungible_resource(dec!(1), 0, account);
        let recall_auth = self.create_fungible_resource(dec!(1), 0, account);
        let update_metadata_auth = self.create_fungible_resource(dec!(1), 0, account);
        let freeze_auth = self.create_fungible_resource(dec!(1), 0, account);
        let admin_auth = self.create_fungible_resource(dec!(1), 0, account);

        let token_address = self.create_fungible_resource_and_deposit(
            OwnerRole::None,
            FungibleResourceRoles {
                mint_roles: mint_roles! {
                    minter => rule!(require(mint_auth));
                    minter_updater => rule!(require(admin_auth));
                },
                burn_roles: burn_roles! {
                    burner => rule!(require(burn_auth));
                    burner_updater => rule!(require(admin_auth));
                },
                freeze_roles: freeze_roles! {
                    freezer => rule!(require(freeze_auth));
                    freezer_updater => rule!(require(admin_auth));
                },
                recall_roles: recall_roles! {
                    recaller => rule!(require(recall_auth));
                    recaller_updater => rule!(require(admin_auth));
                },
                withdraw_roles: withdraw_roles! {
                    withdrawer => rule!(require(withdraw_auth));
                    withdrawer_updater => rule!(require(admin_auth));
                },
                deposit_roles: deposit_roles! {
                    depositor => rule!(allow_all);
                    depositor_updater => rule!(require(admin_auth));
                },
            },
            account,
        );

        (
            token_address,
            mint_auth,
            burn_auth,
            withdraw_auth,
            recall_auth,
            update_metadata_auth,
            freeze_auth,
            admin_auth,
        )
    }

    pub fn create_everything_allowed_non_fungible_resource(
        &mut self,
        owner_role: OwnerRole,
    ) -> ResourceAddress {
        let receipt = self.execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .create_non_fungible_resource::<Vec<_>, ()>(
                    owner_role,
                    NonFungibleIdType::Integer,
                    false,
                    NonFungibleResourceRoles {
                        mint_roles: mint_roles! {
                            minter => rule!(allow_all);
                            minter_updater => rule!(allow_all);
                        },
                        burn_roles: burn_roles! {
                            burner => rule!(allow_all);
                            burner_updater => rule!(allow_all);
                        },
                        freeze_roles: freeze_roles! {
                            freezer => rule!(allow_all);
                            freezer_updater => rule!(allow_all);
                        },
                        recall_roles: recall_roles! {
                            recaller => rule!(allow_all);
                            recaller_updater => rule!(allow_all);
                        },
                        withdraw_roles: withdraw_roles! {
                            withdrawer => rule!(allow_all);
                            withdrawer_updater => rule!(allow_all);
                        },
                        deposit_roles: deposit_roles! {
                            depositor => rule!(allow_all);
                            depositor_updater => rule!(allow_all);
                        },
                        non_fungible_data_update_roles: non_fungible_data_update_roles! {
                            non_fungible_data_updater => rule!(allow_all);
                            non_fungible_data_updater_updater => rule!(allow_all);
                        },
                    },
                    metadata!(),
                    None,
                )
                .build(),
            vec![],
        );
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    pub fn create_freezeable_token(&mut self, account: ComponentAddress) -> ResourceAddress {
        self.create_fungible_resource_and_deposit(
            OwnerRole::None,
            FungibleResourceRoles {
                burn_roles: burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                },
                recall_roles: recall_roles! {
                    recaller => rule!(allow_all);
                    recaller_updater => rule!(deny_all);
                },
                freeze_roles: freeze_roles! {
                    freezer => rule!(allow_all);
                    freezer_updater => rule!(deny_all);
                },
                ..Default::default()
            },
            account,
        )
    }

    pub fn create_freezeable_non_fungible(&mut self, account: ComponentAddress) -> ResourceAddress {
        self.create_non_fungible_resource_with_roles(
            NonFungibleResourceRoles {
                burn_roles: burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                },
                recall_roles: recall_roles! {
                    recaller => rule!(allow_all);
                    recaller_updater => rule!(deny_all);
                },
                freeze_roles: freeze_roles! {
                    freezer => rule!(allow_all);
                    freezer_updater => rule!(deny_all);
                },
                ..Default::default()
            },
            account,
        )
    }

    pub fn create_recallable_token(&mut self, account: ComponentAddress) -> ResourceAddress {
        self.create_fungible_resource_and_deposit(
            OwnerRole::None,
            FungibleResourceRoles {
                recall_roles: recall_roles! {
                    recaller => rule!(allow_all);
                    recaller_updater => rule!(deny_all);
                },
                ..Default::default()
            },
            account,
        )
    }

    pub fn create_restricted_burn_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_fungible_resource(dec!(1), 0, account);

        let resource_address = self.create_fungible_resource_and_deposit(
            OwnerRole::None,
            FungibleResourceRoles {
                burn_roles: burn_roles! {
                    burner => rule!(require(auth_resource_address));
                    burner_updater => rule!(deny_all);
                },
                ..Default::default()
            },
            account,
        );

        (auth_resource_address, resource_address)
    }

    pub fn create_restricted_transfer_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_non_fungible_resource(account);

        let resource_address = self.create_fungible_resource_and_deposit(
            OwnerRole::None,
            FungibleResourceRoles {
                withdraw_roles: withdraw_roles! {
                    withdrawer => rule!(require(auth_resource_address));
                    withdrawer_updater => rule!(deny_all);
                },
                ..Default::default()
            },
            account,
        );

        (auth_resource_address, resource_address)
    }

    pub fn create_non_fungible_resource(&mut self, account: ComponentAddress) -> ResourceAddress {
        self.create_non_fungible_resource_advanced(NonFungibleResourceRoles::default(), account, 3)
    }
    pub fn create_non_fungible_resource_with_roles(
        &mut self,
        resource_roles: NonFungibleResourceRoles,
        account: ComponentAddress,
    ) -> ResourceAddress {
        self.create_non_fungible_resource_advanced(resource_roles, account, 3)
    }

    pub fn create_non_fungible_resource_advanced(
        &mut self,
        resource_roles: NonFungibleResourceRoles,
        account: ComponentAddress,
        n: usize,
    ) -> ResourceAddress {
        let mut entries = BTreeMap::new();
        for i in 1..n + 1 {
            entries.insert(
                NonFungibleLocalId::integer(i as u64),
                EmptyNonFungibleData {},
            );
        }

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                false,
                resource_roles,
                metadata!(),
                Some(entries),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    pub fn create_fungible_resource(
        &mut self,
        amount: Decimal,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                divisibility,
                FungibleResourceRoles::default(),
                metadata!(),
                Some(amount),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    pub fn create_mintable_burnable_fungible_resource(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let admin_auth = self.create_non_fungible_resource(account);

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                1u8,
                FungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(require(admin_auth));
                        minter_updater => rule!(deny_all);
                    },
                    burn_roles: burn_roles! {
                        burner => rule!(require(admin_auth));
                        burner_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata!(),
                None,
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        let resource_address = receipt.expect_commit(true).new_resource_addresses()[0];
        (admin_auth, resource_address)
    }

    pub fn create_freely_mintable_fungible_resource(
        &mut self,
        owner_role: OwnerRole,
        amount: Option<Decimal>,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                owner_role,
                true,
                divisibility,
                FungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata!(),
                amount,
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    pub fn create_freely_mintable_and_burnable_fungible_resource(
        &mut self,
        owner_role: OwnerRole,
        amount: Option<Decimal>,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                owner_role,
                true,
                divisibility,
                FungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    },
                    burn_roles: burn_roles! {
                        burner => rule!(allow_all);
                        burner_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata!(),
                amount,
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    pub fn create_freely_mintable_and_burnable_non_fungible_resource<T, V>(
        &mut self,
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        initial_supply: Option<T>,
        account: ComponentAddress,
    ) -> ResourceAddress
    where
        T: IntoIterator<Item = (NonFungibleLocalId, V)>,
        V: ManifestEncode + NonFungibleData,
    {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                owner_role,
                id_type,
                true,
                NonFungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    },
                    burn_roles: burn_roles! {
                        burner => rule!(allow_all);
                        burner_updater => rule!(deny_all);
                    },
                    ..Default::default()
                },
                metadata!(),
                initial_supply,
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    pub fn create_one_resource_pool(
        &mut self,
        resource_address: ResourceAddress,
        pool_manager_rule: AccessRule,
    ) -> (ComponentAddress, ResourceAddress) {
        let manifest = ManifestBuilder::new()
            .call_function(
                POOL_PACKAGE,
                ONE_RESOURCE_POOL_BLUEPRINT_IDENT,
                ONE_RESOURCE_POOL_INSTANTIATE_IDENT,
                OneResourcePoolInstantiateManifestInput {
                    resource_address,
                    pool_manager_rule,
                    owner_role: OwnerRole::None,
                    address_reservation: None,
                },
            )
            .build();
        let receipt = self.execute_manifest_ignoring_fee(manifest, vec![]);
        let commit_result = receipt.expect_commit_success();

        (
            commit_result.new_component_addresses()[0],
            commit_result.new_resource_addresses()[0],
        )
    }

    pub fn new_component<F>(
        &mut self,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
        handler: F,
    ) -> ComponentAddress
    where
        F: FnOnce(ManifestBuilder) -> ManifestBuilder,
    {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .then(handler)
            .build();

        let receipt = self.execute_manifest(manifest, initial_proofs);
        receipt.expect_commit(true).new_component_addresses()[0]
    }

    pub fn set_current_epoch(&mut self, epoch: Epoch) {
        let reader = SystemDatabaseReader::new(&self.database);
        let mut substate = reader
            .read_typed_object_field::<ConsensusManagerStateFieldPayload>(
                CONSENSUS_MANAGER.as_node_id(),
                ModuleId::Main,
                ConsensusManagerField::State.field_index(),
            )
            .unwrap()
            .into_latest();

        substate.epoch = epoch;

        let mut writer = SystemDatabaseWriter::new(&mut self.database);
        writer
            .write_typed_object_field(
                CONSENSUS_MANAGER.as_node_id(),
                ModuleId::Main,
                ConsensusManagerField::State.field_index(),
                ConsensusManagerStateFieldPayload::from_content_source(substate),
            )
            .unwrap();
    }

    pub fn get_current_epoch(&mut self) -> Epoch {
        let receipt = self.execute_system_transaction(
            vec![InstructionV1::CallMethod {
                address: CONSENSUS_MANAGER.into(),
                method_name: CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&ConsensusManagerGetCurrentEpochInput),
            }],
            btreeset![AuthAddresses::validator_role()],
        );
        receipt.expect_commit(true).output(0)
    }

    pub fn execute_system_transaction_with_preallocation(
        &mut self,
        instructions: Vec<InstructionV1>,
        proofs: BTreeSet<NonFungibleGlobalId>,
        pre_allocated_addresses: Vec<PreAllocatedAddress>,
    ) -> TransactionReceipt {
        let nonce = self.next_transaction_nonce();

        self.execute_transaction(
            SystemTransactionV1 {
                instructions: InstructionsV1(instructions),
                blobs: BlobsV1 { blobs: vec![] },
                hash_for_execution: hash(format!("Test runner txn: {}", nonce)),
                pre_allocated_addresses,
            }
            .prepare()
            .expect("expected transaction to be preparable")
            .get_executable(proofs),
            CostingParameters::default(),
            ExecutionConfig::for_system_transaction(NetworkDefinition::simulator()),
        )
    }

    pub fn execute_validator_transaction(
        &mut self,
        instructions: Vec<InstructionV1>,
    ) -> TransactionReceipt {
        self.execute_system_transaction(instructions, btreeset![AuthAddresses::validator_role()])
    }

    pub fn execute_system_transaction_with_preallocated_addresses(
        &mut self,
        instructions: Vec<InstructionV1>,
        pre_allocated_addresses: Vec<PreAllocatedAddress>,
        mut proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> TransactionReceipt {
        let nonce = self.next_transaction_nonce();
        proofs.insert(AuthAddresses::system_role());
        self.execute_transaction(
            SystemTransactionV1 {
                instructions: InstructionsV1(instructions),
                blobs: BlobsV1 { blobs: vec![] },
                hash_for_execution: hash(format!("Test runner txn: {}", nonce)),
                pre_allocated_addresses,
            }
            .prepare()
            .expect("expected transaction to be preparable")
            .get_executable(proofs),
            CostingParameters::default(),
            ExecutionConfig::for_system_transaction(NetworkDefinition::simulator()),
        )
    }

    pub fn execute_system_transaction(
        &mut self,
        instructions: Vec<InstructionV1>,
        proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> TransactionReceipt {
        let nonce = self.next_transaction_nonce();

        self.execute_transaction(
            SystemTransactionV1 {
                instructions: InstructionsV1(instructions),
                blobs: BlobsV1 { blobs: vec![] },
                hash_for_execution: hash(format!("Test runner txn: {}", nonce)),
                pre_allocated_addresses: vec![],
            }
            .prepare()
            .expect("expected transaction to be preparable")
            .get_executable(proofs),
            CostingParameters::default(),
            ExecutionConfig::for_system_transaction(NetworkDefinition::simulator()),
        )
    }

    /// Executes a "start round number `round` at timestamp `timestamp_ms`" system transaction, as
    /// if it was proposed by the first validator from the validator set, after `round - 1` missed
    /// rounds by that validator.
    pub fn advance_to_round_at_timestamp(
        &mut self,
        round: Round,
        proposer_timestamp_ms: i64,
    ) -> TransactionReceipt {
        let expected_round_number = self.get_consensus_manager_state().round.number() + 1;
        self.execute_system_transaction(
            vec![InstructionV1::CallMethod {
                address: CONSENSUS_MANAGER.into(),
                method_name: CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&ConsensusManagerNextRoundInput {
                    round,
                    proposer_timestamp_ms,
                    leader_proposal_history: LeaderProposalHistory {
                        gap_round_leaders: (expected_round_number..round.number())
                            .map(|_| 0)
                            .collect(),
                        current_leader: 0,
                        is_fallback: false,
                    },
                }),
            }],
            btreeset![AuthAddresses::validator_role()],
        )
    }

    /// Performs an [`advance_to_round_at_timestamp()`] with an unchanged timestamp.
    pub fn advance_to_round(&mut self, round: Round) -> TransactionReceipt {
        let current_timestamp_ms = self.get_current_proposer_timestamp_ms();
        self.advance_to_round_at_timestamp(round, current_timestamp_ms)
    }

    /// Reads out the substate holding the "epoch milli" timestamp reported by the proposer on the
    /// most recent round change.
    pub fn get_current_proposer_timestamp_ms(&mut self) -> i64 {
        let reader = SystemDatabaseReader::new(self.substate_db());
        reader
            .read_typed_object_field::<ConsensusManagerProposerMilliTimestampFieldPayload>(
                CONSENSUS_MANAGER.as_node_id(),
                ModuleId::Main,
                ConsensusManagerField::ProposerMilliTimestamp.field_index(),
            )
            .unwrap()
            .into_latest()
            .epoch_milli
    }

    pub fn get_consensus_manager_state(&mut self) -> ConsensusManagerSubstate {
        let reader = SystemDatabaseReader::new(self.substate_db());
        reader
            .read_typed_object_field::<ConsensusManagerStateFieldPayload>(
                CONSENSUS_MANAGER.as_node_id(),
                ModuleId::Main,
                ConsensusManagerField::State.field_index(),
            )
            .unwrap()
            .into_latest()
    }

    pub fn get_current_time(&mut self, precision: TimePrecision) -> Instant {
        let receipt = self.execute_system_transaction(
            vec![InstructionV1::CallMethod {
                address: CONSENSUS_MANAGER.into(),
                method_name: CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&ConsensusManagerGetCurrentTimeInputV2 {
                    precision
                }),
            }],
            btreeset![AuthAddresses::validator_role()],
        );
        receipt.expect_commit(true).output(0)
    }

    pub fn event_schema(
        &self,
        event_type_identifier: &EventTypeIdentifier,
    ) -> (LocalTypeId, VersionedScryptoSchema) {
        let (blueprint_id, name) = match event_type_identifier {
            EventTypeIdentifier(Emitter::Method(node_id, node_module), event_name) => {
                let blueprint_id = match node_module {
                    ModuleId::Main => {
                        let reader = SystemDatabaseReader::new(self.substate_db());
                        let type_info = reader.get_type_info(node_id).unwrap();
                        match type_info {
                            TypeInfoSubstate::Object(ObjectInfo {
                                blueprint_info: BlueprintInfo { blueprint_id, .. },
                                ..
                            }) => blueprint_id,
                            _ => {
                                panic!("No event schema.")
                            }
                        }
                    }
                    module @ _ => module.static_blueprint().unwrap(),
                };
                (blueprint_id, event_name.clone())
            }
            EventTypeIdentifier(Emitter::Function(blueprint_id), event_name) => {
                (blueprint_id.clone(), event_name.clone())
            }
        };

        let system_reader = SystemDatabaseReader::new(self.substate_db());
        let definition = system_reader
            .get_blueprint_definition(&blueprint_id)
            .unwrap();
        let schema_pointer = definition
            .interface
            .get_event_payload_def(name.as_str())
            .unwrap();

        match schema_pointer {
            BlueprintPayloadDef::Static(type_identifier) => {
                let reader = SystemDatabaseReader::new(self.substate_db());
                let schema = reader
                    .get_schema(
                        blueprint_id.package_address.as_node_id(),
                        &type_identifier.0,
                    )
                    .unwrap();
                (type_identifier.1, schema.as_ref().clone())
            }
            BlueprintPayloadDef::Generic(_instance_index) => {
                todo!()
            }
        }
    }

    pub fn event_name(&self, event_type_identifier: &EventTypeIdentifier) -> String {
        let (local_type_id, schema) = self.event_schema(event_type_identifier);
        schema
            .v1()
            .resolve_type_metadata(local_type_id)
            .unwrap()
            .get_name_string()
            .unwrap()
    }

    pub fn is_event_name_equal<T: ScryptoDescribe>(
        &self,
        event_type_identifier: &EventTypeIdentifier,
    ) -> bool {
        let expected_type_name = {
            let (local_type_id, schema) =
                sbor::generate_full_schema_from_single_type::<T, ScryptoCustomSchema>();
            schema
                .v1()
                .resolve_type_metadata(local_type_id)
                .unwrap()
                .get_name_string()
                .unwrap()
        };
        let actual_type_name = self.event_name(event_type_identifier);
        expected_type_name == actual_type_name
    }

    pub fn extract_events_of_type<T: ScryptoEvent>(&self, result: &CommitResult) -> Vec<T> {
        result
            .application_events
            .iter()
            .filter(|(id, _data)| self.is_event_name_equal::<T>(id))
            .map(|(_id, data)| scrypto_decode::<T>(data).unwrap())
            .collect::<Vec<_>>()
    }

    pub fn check_db<A: ApplicationChecker + Default>(
        &self,
    ) -> Result<
        (SystemDatabaseCheckerResults, A::ApplicationCheckerResults),
        SystemDatabaseCheckError,
    > {
        let mut checker = SystemDatabaseChecker::<A>::default();
        checker.check_db(&self.database)
    }

    pub fn check_events<A: ApplicationEventChecker>(
        &self,
    ) -> Result<A::ApplicationEventCheckerResults, SystemEventCheckerError> {
        let mut event_checker = SystemEventChecker::<A>::new();
        event_checker.check_all_events(&self.database, self.collected_events())
    }

    pub fn check_database(&self) {
        let mut kernel_checker = KernelDatabaseChecker::new();
        kernel_checker
            .check_db(&self.database)
            .expect("Database should be consistent");

        // Defining a composite checker of all of the application db checkers we have.
        radix_engine::define_composite_checker! {
            CompositeApplicationDatabaseChecker,
            [
                ResourceDatabaseChecker,
                RoleAssignmentDatabaseChecker,
                PackageRoyaltyDatabaseChecker<F: Fn(&BlueprintId, &str) -> bool>,
                ComponentRoyaltyDatabaseChecker,
            ]
        }
        let db_results = {
            let reader = SystemDatabaseReader::new(&self.database);
            let mut checker = SystemDatabaseChecker::new(CompositeApplicationDatabaseChecker::new(
                Default::default(),
                Default::default(),
                PackageRoyaltyDatabaseChecker::new(|blueprint_id, func_name| {
                    reader
                        .get_blueprint_definition(blueprint_id)
                        .map(|bp_def| bp_def.interface.functions.contains_key(func_name))
                        .unwrap_or(false)
                }),
                Default::default(),
            ));
            checker
                .check_db(&self.database)
                .expect("Database should be consistent after running test")
        };
        println!("{:#?}", db_results);

        if !db_results.1 .1.is_empty() {
            panic!("Role assignment violations: {:?}", db_results.1 .1);
        }

        let event_results = SystemEventChecker::<ResourceEventChecker>::new()
            .check_all_events(&self.database, &self.collected_events)
            .expect("Events should be consistent");
        println!("{:#?}", event_results);

        // If free credits (xrd from thin air) have been used then reconciliation will fail
        // due to missing mint events
        if !self.xrd_free_credits_used {
            ResourceReconciler::reconcile(&db_results.1 .0, &event_results)
                .expect("Resource reconciliation failed");
        }
    }
}

impl<E: NativeVmExtension, D: TestDatabase> TestRunner<E, HashTreeUpdatingDatabase<D>> {
    pub fn get_state_hash(&self) -> Hash {
        self.database.get_current_root_hash()
    }

    pub fn assert_state_hash_tree_matches_substate_store(&mut self) {
        let hashes_from_tree = self.database.list_substate_hashes();
        assert_eq!(
            hashes_from_tree.keys().cloned().collect::<HashSet<_>>(),
            self.database.list_partition_keys().collect::<HashSet<_>>(),
            "partitions captured in the state hash tree should match those in the substate store"
        );
        for (db_partition_key, by_db_sort_key) in hashes_from_tree {
            assert_eq!(
                by_db_sort_key.into_iter().collect::<HashMap<_, _>>(),
                self.database.list_entries(&db_partition_key)
                    .map(|(db_sort_key, substate_value)| (db_sort_key, hash(substate_value)))
                    .collect::<HashMap<_, _>>(),
                "partition's {:?} substates in the state hash tree should match those in the substate store",
                db_partition_key,
            )
        }
    }
}

pub struct SubtreeVaults<'d, D> {
    database: &'d D,
}

impl<'d, D: SubstateDatabase> SubtreeVaults<'d, D> {
    pub fn new(database: &'d D) -> Self {
        Self { database }
    }

    pub fn get_all(&self, node_id: &NodeId) -> IndexMap<ResourceAddress, Vec<NodeId>> {
        let mut vault_finder = VaultFinder::new();
        let mut traverser = StateTreeTraverser::new(self.database, &mut vault_finder, 100);
        traverser.traverse_subtree(None, *node_id);
        vault_finder.to_vaults()
    }

    pub fn sum_balance_changes(
        &self,
        node_id: &NodeId,
        vault_balance_changes: &IndexMap<NodeId, (ResourceAddress, BalanceChange)>,
    ) -> IndexMap<ResourceAddress, BalanceChange> {
        self.get_all(node_id)
            .into_iter()
            .filter_map(|(traversed_resource, vault_ids)| {
                vault_ids
                    .into_iter()
                    .filter_map(|vault_id| vault_balance_changes.get(&vault_id).cloned())
                    .map(|(reported_resource, change)| {
                        assert_eq!(reported_resource, traversed_resource);
                        change
                    })
                    .reduce(|mut left, right| {
                        left.add_assign(right);
                        left
                    })
                    .map(|change| (traversed_resource, change))
            })
            .collect()
    }
}

pub fn is_auth_error(e: &RuntimeError) -> bool {
    matches!(
        e,
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(_))
    )
}

pub fn is_costing_error(e: &RuntimeError) -> bool {
    matches!(
        e,
        RuntimeError::SystemModuleError(SystemModuleError::CostingError(_))
    )
}

pub fn is_wasm_error(e: &RuntimeError) -> bool {
    matches!(e, RuntimeError::VmError(VmError::Wasm(..)))
}
pub fn wat2wasm(wat: &str) -> Vec<u8> {
    let mut features = wabt::Features::new();
    features.enable_sign_extension();

    wabt::wat2wasm_with_features(
        wat.replace("${memcpy}", include_str!("snippets/memcpy.wat"))
            .replace("${memmove}", include_str!("snippets/memmove.wat"))
            .replace("${memset}", include_str!("snippets/memset.wat")),
        features,
    )
    .expect("Failed to compiled WAT into WASM")
}

/// Gets the default cargo directory for the given crate.
/// This respects whether the crate is in a workspace.
pub fn get_cargo_target_directory(manifest_path: impl AsRef<OsStr>) -> String {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--manifest-path")
        .arg(manifest_path.as_ref())
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .output()
        .expect("Failed to call cargo metadata");
    if output.status.success() {
        let parsed = serde_json::from_slice::<serde_json::Value>(&output.stdout)
            .expect("Failed to parse cargo metadata");
        let target_directory = parsed
            .as_object()
            .and_then(|o| o.get("target_directory"))
            .and_then(|o| o.as_str())
            .expect("Failed to parse target_directory from cargo metadata");
        target_directory.to_owned()
    } else {
        panic!("Cargo metadata call was not successful");
    }
}

pub fn single_function_package_definition(
    blueprint_name: &str,
    function_name: &str,
) -> PackageDefinition {
    PackageDefinition::new_single_function_test_definition(blueprint_name, function_name)
}

#[derive(ScryptoSbor, NonFungibleData, ManifestSbor)]
pub struct EmptyNonFungibleData {}

pub struct TransactionParams {
    pub start_epoch_inclusive: Epoch,
    pub end_epoch_exclusive: Epoch,
}

pub fn create_notarized_transaction(
    params: TransactionParams,
    manifest: TransactionManifestV1,
) -> NotarizedTransactionV1 {
    // create key pairs
    let sk1 = Secp256k1PrivateKey::from_u64(1).unwrap();
    let sk2 = Secp256k1PrivateKey::from_u64(2).unwrap();
    let sk_notary = Secp256k1PrivateKey::from_u64(3).unwrap();

    TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: params.start_epoch_inclusive,
            end_epoch_exclusive: params.end_epoch_exclusive,
            nonce: 5,
            notary_public_key: sk_notary.public_key().into(),
            notary_is_signatory: false,
            tip_percentage: 5,
        })
        .manifest(manifest)
        .sign(&sk1)
        .sign(&sk2)
        .notarize(&sk_notary)
        .build()
}

pub fn create_notarized_transaction_advanced<S: Signer>(
    test_runner: &mut DefaultTestRunner,
    network: &NetworkDefinition,
    manifest: TransactionManifestV1,
    signers: Vec<&S>,
    notary: &S,
    notary_is_signatory: bool,
) -> NotarizedTransactionV1 {
    let notarized_transaction = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: network.id,
            start_epoch_inclusive: Epoch::zero(),
            end_epoch_exclusive: Epoch::of(99),
            nonce: test_runner.next_transaction_nonce(),
            notary_public_key: notary.public_key().into(),
            notary_is_signatory: notary_is_signatory,
            tip_percentage: DEFAULT_TIP_PERCENTAGE,
        })
        .manifest(manifest)
        .multi_sign(&signers)
        .notarize(notary)
        .build();
    notarized_transaction
}

pub fn validate_notarized_transaction<'a>(
    network: &'a NetworkDefinition,
    transaction: &'a NotarizedTransactionV1,
) -> ValidatedNotarizedTransactionV1 {
    NotarizedTransactionValidator::new(ValidationConfig::default(network.id))
        .validate(transaction.prepare().unwrap())
        .unwrap()
}

pub fn assert_receipt_substate_changes_can_be_typed(commit_result: &CommitResult) {
    let system_updates = commit_result
        .state_updates
        .clone()
        .into_legacy()
        .system_updates;
    for ((node_id, partition_num), partition_updates) in (&system_updates).into_iter() {
        for (substate_key, database_update) in partition_updates.into_iter() {
            let typed_substate_key =
                to_typed_substate_key(node_id.entity_type().unwrap(), *partition_num, substate_key)
                    .expect("Substate key should be typeable");
            if !typed_substate_key.value_is_mappable() {
                continue;
            }
            match database_update {
                DatabaseUpdate::Set(raw_value) => {
                    // Check that typed value mapping works
                    to_typed_substate_value(&typed_substate_key, raw_value)
                        .expect("Substate value should be typeable");
                }
                DatabaseUpdate::Delete => {}
            }
        }
    }
}

pub fn assert_receipt_events_can_be_typed(commit_result: &CommitResult) {
    for (event_type_identifier, event_data) in &commit_result.application_events {
        match event_type_identifier.0 {
            Emitter::Function(BlueprintId {
                package_address, ..
            }) if ![
                PACKAGE_PACKAGE,
                RESOURCE_PACKAGE,
                ACCOUNT_PACKAGE,
                IDENTITY_PACKAGE,
                CONSENSUS_MANAGER_PACKAGE,
                ACCESS_CONTROLLER_PACKAGE,
                POOL_PACKAGE,
                TRANSACTION_PROCESSOR_PACKAGE,
                METADATA_MODULE_PACKAGE,
                ROYALTY_MODULE_PACKAGE,
                ROLE_ASSIGNMENT_MODULE_PACKAGE,
                TEST_UTILS_PACKAGE,
                GENESIS_HELPER_PACKAGE,
                FAUCET_PACKAGE,
                TRANSACTION_TRACKER_PACKAGE,
            ]
            .contains(&package_address) =>
            {
                continue
            }
            Emitter::Method(node_id, ..)
                if node_id.entity_type().map_or(false, |item| {
                    matches!(
                        item,
                        EntityType::GlobalGenericComponent | EntityType::InternalGenericComponent
                    )
                }) =>
            {
                continue
            }
            _ => {}
        };
        let _ = to_typed_native_event(event_type_identifier, event_data).unwrap();
    }
}

pub enum PackagePublishingSource {
    CompileAndPublishFromSource(PathBuf),
    PublishExisting(Vec<u8>, PackageDefinition),
}

impl From<String> for PackagePublishingSource {
    fn from(value: String) -> Self {
        Self::CompileAndPublishFromSource(value.into())
    }
}

impl<'g> From<&'g str> for PackagePublishingSource {
    fn from(value: &'g str) -> Self {
        Self::CompileAndPublishFromSource(value.into())
    }
}

impl From<PathBuf> for PackagePublishingSource {
    fn from(value: PathBuf) -> Self {
        Self::CompileAndPublishFromSource(value)
    }
}

impl<'g> From<&'g Path> for PackagePublishingSource {
    fn from(value: &'g Path) -> Self {
        Self::CompileAndPublishFromSource(value.into())
    }
}

impl From<(Vec<u8>, PackageDefinition)> for PackagePublishingSource {
    fn from(value: (Vec<u8>, PackageDefinition)) -> Self {
        Self::PublishExisting(value.0, value.1)
    }
}

impl PackagePublishingSource {
    pub fn code_and_definition(self) -> (Vec<u8>, PackageDefinition) {
        match self {
            Self::CompileAndPublishFromSource(path) => Compile::compile(path),
            Self::PublishExisting(code, definition) => (code, definition),
        }
    }
}
