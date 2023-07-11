use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use radix_engine::blueprints::consensus_manager::*;
use radix_engine::errors::*;
use radix_engine::system::bootstrap::*;
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::system::system::{FieldSubstate, KeyValueEntrySubstate};
use radix_engine::transaction::{
    execute_preview, execute_transaction, CommitResult, ExecutionConfig, FeeReserveConfig,
    PreviewError, TransactionReceipt, TransactionResult,
};
use radix_engine::types::*;
use radix_engine::utils::*;
use radix_engine::vm::wasm::{DefaultWasmEngine, WasmValidatorConfigV1};
use radix_engine::vm::{NativeVm, NativeVmExtension, NoExtension, ScryptoVm, Vm};
use radix_engine_interface::api::node_modules::auth::ToRoleEntry;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::royalty::ComponentRoyaltySubstate;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerConfig, ConsensusManagerGetCurrentEpochInput,
    ConsensusManagerGetCurrentTimeInput, ConsensusManagerNextRoundInput, EpochChangeCondition,
    LeaderProposalHistory, TimePrecision, CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
    CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT, CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
};
use radix_engine_interface::blueprints::package::{
    BlueprintDefinitionInit, PackageDefinition, PackagePublishNativeManifestInput,
    PackagePublishWasmAdvancedManifestInput, PackageRoyaltyAccumulatorSubstate, TypePointer,
    PACKAGE_BLUEPRINT, PACKAGE_PUBLISH_NATIVE_IDENT, PACKAGE_PUBLISH_WASM_ADVANCED_IDENT,
    PACKAGE_SCHEMAS_PARTITION_OFFSET,
};
use radix_engine_interface::constants::CONSENSUS_MANAGER;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_interface::time::Instant;
use radix_engine_interface::{dec, freeze_roles, rule};
use radix_engine_queries::query::{ResourceAccounter, StateTreeTraverser, VaultFinder};
use radix_engine_queries::typed_substate_layout::{
    BlueprintDefinition, BlueprintVersionKey, PACKAGE_BLUEPRINTS_PARTITION_OFFSET,
};
use radix_engine_store_interface::db_key_mapper::DatabaseKeyMapper;
use radix_engine_store_interface::interface::{ListableSubstateDatabase, SubstateDatabase};
use radix_engine_store_interface::{
    db_key_mapper::{
        MappedCommittableSubstateDatabase, MappedSubstateDatabase, SpreadPrefixKeyMapper,
    },
    interface::{CommittableSubstateDatabase, DatabaseUpdate, DatabaseUpdates},
};
use radix_engine_stores::hash_tree::tree_store::{TypedInMemoryTreeStore, Version};
use radix_engine_stores::hash_tree::{put_at_next_version, SubstateHashChange};
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use scrypto::prelude::*;
use transaction::prelude::*;
use transaction::signing::secp256k1::Secp256k1PrivateKey;
use transaction::validation::{
    NotarizedTransactionValidator, TransactionValidator, ValidationConfig,
};

pub struct Compile;

impl Compile {
    pub fn compile<P: AsRef<Path>>(package_dir: P) -> (Vec<u8>, PackageDefinition) {
        // Build
        let status = Command::new("cargo")
            .current_dir(package_dir.as_ref())
            .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
            .status()
            .unwrap();
        if !status.success() {
            panic!("Failed to compile package: {:?}", package_dir.as_ref());
        }

        // Find wasm path
        let mut cargo = package_dir.as_ref().to_owned();
        cargo.push("Cargo.toml");
        let wasm_name = if cargo.exists() {
            let content = fs::read_to_string(&cargo).expect("Failed to read the Cargo.toml file");
            Self::extract_crate_name(&content)
                .expect("Failed to extract crate name from the Cargo.toml file")
                .replace("-", "_")
        } else {
            // file name
            package_dir
                .as_ref()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
                .replace("-", "_")
        };
        let mut path = PathBuf::from_str(&get_cargo_target_directory(&cargo)).unwrap(); // Infallible;
        path.push("wasm32-unknown-unknown");
        path.push("release");
        path.push(wasm_name);
        path.set_extension("wasm");

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
        staker_account: ComponentAddress,
        genesis_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
    ) -> CustomGenesis {
        let genesis_validator: GenesisValidator = validator_public_key.clone().into();
        let genesis_data_chunks = vec![
            GenesisDataChunk::Validators(vec![genesis_validator]),
            GenesisDataChunk::Stakes {
                accounts: vec![staker_account],
                allocations: vec![(
                    validator_public_key,
                    vec![GenesisStakeAllocation {
                        account_index: 0,
                        xrd_amount: stake_xrd_amount,
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

    pub fn two_validators_and_single_staker(
        validator1_public_key: Secp256k1PublicKey,
        validator2_public_key: Secp256k1PublicKey,
        stake_xrd_amount: (Decimal, Decimal),
        staker_account: ComponentAddress,
        genesis_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
    ) -> CustomGenesis {
        let genesis_validator1: GenesisValidator = validator1_public_key.clone().into();
        let genesis_validator2: GenesisValidator = validator2_public_key.clone().into();
        let genesis_data_chunks = vec![
            GenesisDataChunk::Validators(vec![genesis_validator1, genesis_validator2]),
            GenesisDataChunk::Stakes {
                accounts: vec![staker_account],
                allocations: vec![
                    (
                        validator1_public_key,
                        vec![GenesisStakeAllocation {
                            account_index: 0,
                            xrd_amount: stake_xrd_amount.0,
                        }],
                    ),
                    (
                        validator2_public_key,
                        vec![GenesisStakeAllocation {
                            account_index: 0,
                            xrd_amount: stake_xrd_amount.1,
                        }],
                    ),
                ],
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

pub struct TestRunnerBuilder {
    custom_genesis: Option<CustomGenesis>,
    trace: bool,
    state_hashing: bool,
}

impl TestRunnerBuilder {
    pub fn new() -> TestRunnerBuilder {
        TestRunnerBuilder {
            custom_genesis: None,
            #[cfg(not(feature = "resource_tracker"))]
            trace: true,
            #[cfg(feature = "resource_tracker")]
            trace: false,
            state_hashing: false,
        }
    }

    pub fn without_trace(mut self) -> Self {
        self.trace = false;
        self
    }

    pub fn with_state_hashing(mut self) -> Self {
        self.state_hashing = true;
        self
    }

    pub fn with_custom_genesis(mut self, genesis: CustomGenesis) -> Self {
        self.custom_genesis = Some(genesis);
        self
    }

    pub fn build_and_get_epoch_with_native_vm_extension<E: NativeVmExtension>(
        self,
        extension: E,
    ) -> (TestRunner<E>, ActiveValidatorSet) {
        let scrypto_vm = ScryptoVm {
            wasm_engine: DefaultWasmEngine::default(),
            wasm_validator_config: WasmValidatorConfigV1::new(),
        };
        let native_vm = NativeVm::new_with_extension(extension);
        let vm = Vm::new(&scrypto_vm, native_vm.clone());
        let mut substate_db = InMemorySubstateDatabase::standard();
        let mut bootstrapper = Bootstrapper::new(&mut substate_db, vm, false);
        let GenesisReceipts {
            wrap_up_receipt, ..
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

        // Note that 0 is not a valid private key
        let next_private_key = 100;

        // Starting from non-zero considering that bootstrap might have used a few.
        let next_transaction_nonce = 100;

        let runner = TestRunner {
            scrypto_vm,
            native_vm,
            substate_db,
            state_hash_support: Some(self.state_hashing)
                .filter(|x| *x)
                .map(|_| StateHashSupport::new()),
            next_private_key,
            next_transaction_nonce,
            trace: self.trace,
        };

        let next_epoch = wrap_up_receipt
            .expect_commit_success()
            .next_epoch()
            .unwrap();
        (runner, next_epoch.validator_set)
    }

    pub fn build_and_get_epoch(self) -> (TestRunner<NoExtension>, ActiveValidatorSet) {
        self.build_and_get_epoch_with_native_vm_extension(NoExtension)
    }

    pub fn build(self) -> TestRunner<NoExtension> {
        self.build_and_get_epoch().0
    }

    pub fn build_with_native_vm_extension<E: NativeVmExtension>(
        self,
        extension: E,
    ) -> TestRunner<E> {
        self.build_and_get_epoch_with_native_vm_extension(extension)
            .0
    }
}

pub struct TestRunner<E: NativeVmExtension> {
    scrypto_vm: ScryptoVm<DefaultWasmEngine>,
    native_vm: NativeVm<E>,
    substate_db: InMemorySubstateDatabase,
    next_private_key: u64,
    next_transaction_nonce: u32,
    trace: bool,
    state_hash_support: Option<StateHashSupport>,
}

#[derive(Clone)]
pub struct TestRunnerSnapshot {
    substate_db: InMemorySubstateDatabase,
    next_private_key: u64,
    next_transaction_nonce: u32,
    state_hash_support: Option<StateHashSupport>,
}

impl<E: NativeVmExtension> TestRunner<E> {
    pub fn create_snapshot(&self) -> TestRunnerSnapshot {
        TestRunnerSnapshot {
            substate_db: self.substate_db.clone(),
            next_private_key: self.next_private_key,
            next_transaction_nonce: self.next_transaction_nonce,
            state_hash_support: self.state_hash_support.clone(),
        }
    }

    pub fn restore_snapshot(&mut self, snapshot: TestRunnerSnapshot) {
        self.substate_db = snapshot.substate_db;
        self.next_private_key = snapshot.next_private_key;
        self.next_transaction_nonce = snapshot.next_transaction_nonce;
        self.state_hash_support = snapshot.state_hash_support;
    }

    pub fn faucet_component(&self) -> GlobalAddress {
        FAUCET.clone().into()
    }

    pub fn substate_db(&self) -> &InMemorySubstateDatabase {
        &self.substate_db
    }

    pub fn substate_db_mut(&mut self) -> &mut InMemorySubstateDatabase {
        &mut self.substate_db
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
        // TODO: Move this to system wrapper around substate_store
        let key = scrypto_encode(key).unwrap();

        let metadata_value = self
            .substate_db
            .get_mapped::<SpreadPrefixKeyMapper, KeyValueEntrySubstate<MetadataValue>>(
                address.as_node_id(),
                METADATA_KV_STORE_PARTITION,
                &SubstateKey::Map(key),
            )?
            .value;

        metadata_value
    }

    pub fn inspect_component_royalty(&mut self, component_address: ComponentAddress) -> Decimal {
        let accumulator = self
            .substate_db
            .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<ComponentRoyaltySubstate>>(
                component_address.as_node_id(),
                ROYALTY_FIELDS_PARTITION,
                &RoyaltyField::RoyaltyAccumulator.into(),
            )
            .unwrap()
            .value
            .0;
        self.substate_db
            .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<LiquidFungibleResource>>(
                accumulator.royalty_vault.0.as_node_id(),
                MAIN_BASE_PARTITION,
                &FungibleVaultField::LiquidFungible.into(),
            )
            .unwrap()
            .value
            .0
            .amount()
    }

    pub fn inspect_package_royalty(&mut self, package_address: PackageAddress) -> Option<Decimal> {
        let output = self
            .substate_db
            .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<PackageRoyaltyAccumulatorSubstate>>(
                package_address.as_node_id(),
                MAIN_BASE_PARTITION,
                &PackageField::Royalty.into(),
            )?
            .value
            .0;

        self.substate_db
            .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<LiquidFungibleResource>>(
                output.royalty_vault.0.as_node_id(),
                MAIN_BASE_PARTITION,
                &FungibleVaultField::LiquidFungible.into(),
            )
            .map(|r| r.value.0.amount())
    }

    pub fn account_balance(
        &mut self,
        account_address: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> Option<Decimal> {
        let vaults = self.get_component_vaults(account_address, resource_address);
        let index = if resource_address.eq(&XRD) {
            // To account for royalty vault
            1usize
        } else {
            0usize
        };
        vaults
            .get(index)
            .map_or(None, |vault_id| self.inspect_vault_balance(*vault_id))
    }

    pub fn find_all_nodes(&self) -> IndexSet<NodeId> {
        let mut node_ids = index_set_new();
        for pk in self.substate_db.list_partition_keys() {
            let (node_id, _) = SpreadPrefixKeyMapper::from_db_partition_key(&pk);
            node_ids.insert(node_id);
        }
        node_ids
    }

    pub fn find_all_components(&self) -> Vec<ComponentAddress> {
        self.find_all_nodes()
            .iter()
            .filter_map(|node_id| ComponentAddress::try_from(node_id.as_bytes()).ok())
            .collect()
    }

    pub fn find_all_packages(&self) -> Vec<PackageAddress> {
        self.find_all_nodes()
            .iter()
            .filter_map(|node_id| PackageAddress::try_from(node_id.as_bytes()).ok())
            .collect()
    }

    pub fn find_all_resources(&self) -> Vec<ResourceAddress> {
        self.find_all_nodes()
            .iter()
            .filter_map(|node_id| ResourceAddress::try_from(node_id.as_bytes()).ok())
            .collect()
    }

    pub fn get_package_scrypto_schemas(
        &self,
        package_address: &PackageAddress,
    ) -> IndexMap<Hash, ScryptoSchema> {
        let mut schemas = index_map_new();
        for entry in self
            .substate_db()
            .list_entries(&SpreadPrefixKeyMapper::to_db_partition_key(
                package_address.as_node_id(),
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_SCHEMAS_PARTITION_OFFSET)
                    .unwrap(),
            ))
        {
            let hash: Hash =
                scrypto_decode(&SpreadPrefixKeyMapper::map_from_db_sort_key(&entry.0)).unwrap();
            let value: KeyValueEntrySubstate<ScryptoSchema> = scrypto_decode(&entry.1).unwrap();
            match value.value {
                Some(schema) => {
                    schemas.insert(hash, schema);
                }
                None => {}
            }
        }

        schemas
    }

    pub fn get_package_blueprint_definitions(
        &self,
        package_address: &PackageAddress,
    ) -> IndexMap<BlueprintVersionKey, BlueprintDefinition> {
        let mut definitions = index_map_new();
        for entry in self
            .substate_db()
            .list_entries(&SpreadPrefixKeyMapper::to_db_partition_key(
                package_address.as_node_id(),
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET)
                    .unwrap(),
            ))
        {
            let key: BlueprintVersionKey =
                scrypto_decode(&SpreadPrefixKeyMapper::map_from_db_sort_key(&entry.0)).unwrap();
            let value: KeyValueEntrySubstate<BlueprintDefinition> =
                scrypto_decode(&entry.1).unwrap();
            match value.value {
                Some(definition) => {
                    definitions.insert(key, definition);
                }
                None => {}
            }
        }

        definitions
    }

    pub fn get_component_vaults(
        &mut self,
        component_address: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> Vec<NodeId> {
        let node_id = component_address.as_node_id();
        let mut vault_finder = VaultFinder::new(resource_address);
        let mut traverser = StateTreeTraverser::new(&self.substate_db, &mut vault_finder, 100);
        traverser.traverse_all_descendents(None, *node_id);
        vault_finder.to_vaults()
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
        self.substate_db()
            .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<LiquidFungibleResource>>(
                &vault_id,
                MAIN_BASE_PARTITION,
                &FungibleVaultField::LiquidFungible.into(),
            )
            .map(|output| output.value.0.amount())
    }

    pub fn inspect_non_fungible_vault(
        &mut self,
        vault_id: NodeId,
    ) -> Option<(Decimal, Option<NonFungibleLocalId>)> {
        let amount = self
            .substate_db()
            .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<LiquidNonFungibleVault>>(
                &vault_id,
                MAIN_BASE_PARTITION,
                &NonFungibleVaultField::LiquidNonFungible.into(),
            )
            .map(|vault| vault.value.0.amount);

        let mut substate_iter = self
            .substate_db()
            .list_mapped::<SpreadPrefixKeyMapper, NonFungibleLocalId, MapKey>(
                &vault_id,
                MAIN_BASE_PARTITION.at_offset(PartitionOffset(1u8)).unwrap(),
            );
        let id = substate_iter.next().map(|(_key, id)| id);

        amount.map(|amount| (amount, id))
    }

    pub fn get_component_resources(
        &mut self,
        component_address: ComponentAddress,
    ) -> HashMap<ResourceAddress, Decimal> {
        let node_id = component_address.as_node_id();
        let mut accounter = ResourceAccounter::new(&self.substate_db);
        accounter.traverse(node_id.clone());
        accounter.close().balances
    }

    pub fn load_account_from_faucet(&mut self, account_address: ComponentAddress) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .take_all_from_worktop(XRD, "free_xrd")
            .try_deposit_or_abort(account_address, "free_xrd")
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
    }

    pub fn new_account_advanced(&mut self, owner_role: OwnerRole) -> ComponentAddress {
        let manifest = ManifestBuilder::new()
            .new_account_advanced(owner_role)
            .build();
        let receipt = self.execute_manifest_ignoring_fee(manifest, vec![]);
        receipt.expect_commit_success();

        let account = receipt.expect_commit(true).new_component_addresses()[0];

        let manifest = ManifestBuilder::new()
            .get_free_xrd_from_faucet()
            .try_deposit_batch_or_abort(account)
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

    pub fn get_active_validator_info_by_key(&self, key: &Secp256k1PublicKey) -> ValidatorSubstate {
        let address = self.get_active_validator_with_key(key);
        self.get_validator_info(address)
    }

    pub fn get_validator_info(&self, address: ComponentAddress) -> ValidatorSubstate {
        self.substate_db()
            .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<ValidatorSubstate>>(
                address.as_node_id(),
                MAIN_BASE_PARTITION,
                &ValidatorField::Validator.into(),
            )
            .unwrap()
            .value
            .0
    }

    pub fn get_active_validator_with_key(&self, key: &Secp256k1PublicKey) -> ComponentAddress {
        let substate = self
            .substate_db()
            .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<CurrentValidatorSetSubstate>>(
                CONSENSUS_MANAGER.as_node_id(),
                MAIN_BASE_PARTITION,
                &ConsensusManagerField::CurrentValidatorSet.into(),
            )
            .unwrap()
            .value
            .0;

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

    pub fn new_identity(&mut self, pk: Secp256k1PublicKey, is_virtual: bool) -> ComponentAddress {
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
            .try_deposit_batch_or_abort(account)
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
            .try_deposit_batch_or_abort(account)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        let address = receipt.expect_commit(true).new_component_addresses()[0];
        address
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

    pub fn publish_package_at_address(
        &mut self,
        code: Vec<u8>,
        definition: PackageDefinition,
        address: PackageAddress,
    ) {
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
            FeeReserveConfig::default(),
            ExecutionConfig::for_system_transaction(),
        );

        receipt.expect_commit_success();
    }

    pub fn publish_package(
        &mut self,
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata: BTreeMap<String, MetadataValue>,
        owner_role: OwnerRole,
    ) -> PackageAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .publish_package_advanced(None, code, definition, metadata, owner_role)
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_package_addresses()[0]
    }

    pub fn publish_package_with_owner(
        &mut self,
        code: Vec<u8>,
        definition: PackageDefinition,
        owner_badge: NonFungibleGlobalId,
    ) -> PackageAddress {
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

    pub fn compile_and_publish<P: AsRef<Path>>(&mut self, package_dir: P) -> PackageAddress {
        let (code, definition) = Compile::compile(package_dir);
        self.publish_package(code, definition, BTreeMap::new(), OwnerRole::None)
    }

    pub fn compile_and_publish_at_address<P: AsRef<Path>>(
        &mut self,
        package_dir: P,
        address: PackageAddress,
    ) {
        let (code, definition) = Compile::compile(package_dir);
        self.publish_package_at_address(code, definition, address);
    }

    pub fn compile_and_publish_retain_blueprints<
        P: AsRef<Path>,
        F: FnMut(&String, &mut BlueprintDefinitionInit) -> bool,
    >(
        &mut self,
        package_dir: P,
        retain: F,
    ) -> PackageAddress {
        let (code, mut definition) = Compile::compile(package_dir);
        definition.blueprints.retain(retain);
        self.publish_package(code, definition, BTreeMap::new(), OwnerRole::None)
    }

    pub fn compile_and_publish_with_owner<P: AsRef<Path>>(
        &mut self,
        package_dir: P,
        owner_badge: NonFungibleGlobalId,
    ) -> PackageAddress {
        let (code, definition) = Compile::compile(package_dir);
        self.publish_package_with_owner(code, definition, owner_badge)
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
            FeeReserveConfig::default(),
            ExecutionConfig::for_notarized_transaction(),
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
        let nonce = self.next_transaction_nonce();
        self.execute_transaction(
            TestTransaction::new_from_nonce(manifest, nonce)
                .prepare()
                .expect("expected transaction to be preparable")
                .get_executable(initial_proofs.into_iter().collect()),
            FeeReserveConfig::default(),
            ExecutionConfig::for_test_transaction(),
        )
    }

    pub fn execute_manifest_with_cost_unit_limit<T>(
        &mut self,
        manifest: TransactionManifestV1,
        initial_proofs: T,
        cost_unit_limit: u32,
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
            FeeReserveConfig::default(),
            ExecutionConfig::for_test_transaction().with_cost_unit_limit(cost_unit_limit),
        )
    }

    pub fn execute_transaction(
        &mut self,
        executable: Executable,
        fee_reserve_config: FeeReserveConfig,
        mut execution_config: ExecutionConfig,
    ) -> TransactionReceipt {
        // Override the kernel trace config
        execution_config = execution_config.with_kernel_trace(self.trace);

        let vm = Vm {
            scrypto_vm: &self.scrypto_vm,
            native_vm: self.native_vm.clone(),
        };

        let transaction_receipt = execute_transaction(
            &mut self.substate_db,
            vm,
            &fee_reserve_config,
            &execution_config,
            &executable,
        );
        if let TransactionResult::Commit(commit) = &transaction_receipt.transaction_result {
            self.substate_db
                .commit(&commit.state_updates.database_updates);
            if let Some(state_hash_support) = &mut self.state_hash_support {
                state_hash_support.update_with(&commit.state_updates.database_updates);
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

        execute_preview(&self.substate_db, vm, network, preview_intent, self.trace)
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
            &mut self.substate_db,
            vm,
            &NetworkDefinition::simulator(),
            PreviewIntentV1 {
                intent: IntentV1 {
                    header: TransactionHeaderV1 {
                        network_id: NetworkDefinition::simulator().id,
                        start_epoch_inclusive: epoch,
                        end_epoch_exclusive: epoch.after(10),
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
        component_address: impl ResolvableGlobalAddress,
        method_name: impl Into<String>,
        args: impl ResolvableArguments,
    ) -> TransactionReceipt {
        self.execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_method(component_address, method_name, args)
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
            .try_deposit_batch_or_abort(to)
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
        self.create_non_fungible_resource_with_access_rules(
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
        self.create_non_fungible_resource_with_access_rules(
            NonFungibleResourceRoles::default(),
            account,
        )
    }

    pub fn create_non_fungible_resource_with_access_rules(
        &mut self,
        resource_roles: NonFungibleResourceRoles,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let mut entries = BTreeMap::new();
        entries.insert(NonFungibleLocalId::integer(1), EmptyNonFungibleData {});
        entries.insert(NonFungibleLocalId::integer(2), EmptyNonFungibleData {});
        entries.insert(NonFungibleLocalId::integer(3), EmptyNonFungibleData {});

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
            .try_deposit_batch_or_abort(account)
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
            .try_deposit_batch_or_abort(account)
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
            .try_deposit_batch_or_abort(account)
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
            .try_deposit_batch_or_abort(account)
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
            .try_deposit_batch_or_abort(account)
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
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
        let mut substate = self
            .substate_db
            .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<ConsensusManagerSubstate>>(
                &CONSENSUS_MANAGER.as_node_id(),
                MAIN_BASE_PARTITION,
                &ConsensusManagerField::ConsensusManager.into(),
            )
            .unwrap();
        substate.value.0.epoch = epoch;
        self.substate_db.put_mapped::<SpreadPrefixKeyMapper, _>(
            &CONSENSUS_MANAGER.as_node_id(),
            MAIN_BASE_PARTITION,
            &ConsensusManagerField::ConsensusManager.into(),
            &substate,
        );
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

    pub fn get_state_hash(&self) -> Hash {
        self.state_hash_support
            .as_ref()
            .expect("state hashing not enabled")
            .get_current()
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
            FeeReserveConfig::default(),
            ExecutionConfig::for_system_transaction(),
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
            FeeReserveConfig::default(),
            ExecutionConfig::for_system_transaction(),
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
            FeeReserveConfig::default(),
            ExecutionConfig::for_system_transaction(),
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
        self.substate_db()
            .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<ProposerMilliTimestampSubstate>>(
                CONSENSUS_MANAGER.as_node_id(),
                MAIN_BASE_PARTITION,
                &ConsensusManagerField::CurrentTime.into(),
            )
            .unwrap()
            .value
            .0
            .epoch_milli
    }

    pub fn get_consensus_manager_state(&mut self) -> ConsensusManagerSubstate {
        self.substate_db()
            .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<ConsensusManagerSubstate>>(
                CONSENSUS_MANAGER.as_node_id(),
                MAIN_BASE_PARTITION,
                &ConsensusManagerField::ConsensusManager.into(),
            )
            .unwrap()
            .value
            .0
    }

    pub fn get_current_time(&mut self, precision: TimePrecision) -> Instant {
        let receipt = self.execute_system_transaction(
            vec![InstructionV1::CallMethod {
                address: CONSENSUS_MANAGER.into(),
                method_name: CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&ConsensusManagerGetCurrentTimeInput {
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
    ) -> (LocalTypeIndex, ScryptoSchema) {
        let (package_address, schema_pointer) = match event_type_identifier {
            EventTypeIdentifier(Emitter::Method(node_id, node_module), schema_pointer) => {
                match node_module {
                    ObjectModuleId::AccessRules => {
                        (ACCESS_RULES_MODULE_PACKAGE, schema_pointer.clone())
                    }
                    ObjectModuleId::Royalty => (ROYALTY_MODULE_PACKAGE, schema_pointer.clone()),
                    ObjectModuleId::Metadata => (METADATA_MODULE_PACKAGE, schema_pointer.clone()),
                    ObjectModuleId::Main => {
                        let type_info = self
                            .substate_db()
                            .get_mapped::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
                                node_id,
                                TYPE_INFO_FIELD_PARTITION,
                                &TypeInfoField::TypeInfo.into(),
                            )
                            .unwrap();

                        match type_info {
                            TypeInfoSubstate::Object(NodeObjectInfo {
                                main_blueprint_id: blueprint,
                                ..
                            }) => (blueprint.package_address, *schema_pointer),
                            _ => {
                                panic!("No event schema.")
                            }
                        }
                    }
                }
            }
            EventTypeIdentifier(Emitter::Function(node_id, ..), schema_pointer) => (
                PackageAddress::new_or_panic(node_id.0),
                schema_pointer.clone(),
            ),
        };

        match schema_pointer {
            TypePointer::Package(schema_hash, index) => {
                let schema = self
                    .substate_db()
                    .get_mapped::<SpreadPrefixKeyMapper, KeyValueEntrySubstate<ScryptoSchema>>(
                        package_address.as_node_id(),
                        MAIN_BASE_PARTITION
                            .at_offset(PACKAGE_SCHEMAS_PARTITION_OFFSET)
                            .unwrap(),
                        &SubstateKey::Map(scrypto_encode(&schema_hash).unwrap()),
                    )
                    .unwrap()
                    .value
                    .unwrap();

                (index, schema)
            }
            TypePointer::Instance(_instance_index) => {
                todo!()
            }
        }
    }

    pub fn event_name(&self, event_type_identifier: &EventTypeIdentifier) -> String {
        let (local_type_index, schema) = self.event_schema(event_type_identifier);
        schema
            .resolve_type_metadata(local_type_index)
            .unwrap()
            .get_name_string()
            .unwrap()
    }

    pub fn is_event_name_equal<T: ScryptoDescribe>(
        &self,
        event_type_identifier: &EventTypeIdentifier,
    ) -> bool {
        let expected_type_name = {
            let (local_type_index, schema) =
                sbor::generate_full_schema_from_single_type::<T, ScryptoCustomSchema>();
            schema
                .resolve_type_metadata(local_type_index)
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
}

#[derive(Clone)]
pub struct StateHashSupport {
    tree_store: TypedInMemoryTreeStore,
    current_version: Version,
    current_hash: Hash,
}

impl StateHashSupport {
    fn new() -> Self {
        StateHashSupport {
            tree_store: TypedInMemoryTreeStore::new(),
            current_version: 0,
            current_hash: Hash([0; Hash::LENGTH]),
        }
    }

    pub fn update_with(&mut self, db_updates: &DatabaseUpdates) {
        let mut hash_changes = Vec::new();
        for (db_partition_key, partition_update) in db_updates {
            for (db_sort_key, db_update) in partition_update {
                let hash_change = SubstateHashChange::new(
                    (db_partition_key.clone(), db_sort_key.clone()),
                    match db_update {
                        DatabaseUpdate::Set(v) => Some(hash(v)),
                        DatabaseUpdate::Delete => None,
                    },
                );
                hash_changes.push(hash_change);
            }
        }

        self.current_hash = put_at_next_version(
            &mut self.tree_store,
            Some(self.current_version).filter(|version| *version > 0),
            hash_changes,
        );
        self.current_version += 1;
    }

    pub fn get_current(&self) -> Hash {
        self.current_hash
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
struct EmptyNonFungibleData {}

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
