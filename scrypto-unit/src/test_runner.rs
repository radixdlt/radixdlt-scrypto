use std::convert::Infallible;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use radix_engine::blueprints::consensus_manager::*;
use radix_engine::errors::*;
use radix_engine::system::bootstrap::*;
use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::transaction::{
    execute_preview, execute_transaction, CommitResult, ExecutionConfig, FeeReserveConfig,
    PreviewError, TransactionReceipt, TransactionResult,
};
use radix_engine::types::*;
use radix_engine::utils::*;
use radix_engine::vm::wasm::{DefaultWasmEngine, WasmInstrumenter, WasmMeteringConfig};
use radix_engine::vm::ScryptoVm;
use radix_engine_interface::api::component::ComponentRoyaltyAccumulatorSubstate;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::node_modules::royalty::*;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerConfig, ConsensusManagerGetCurrentEpochInput,
    ConsensusManagerGetCurrentTimeInput, ConsensusManagerNextRoundInput, EpochChangeCondition,
    LeaderProposalHistory, TimePrecision, CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
    CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT, CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
};
use radix_engine_interface::blueprints::package::{
    PackageDefinition, PackageInfoSubstate, PackageRoyaltySubstate,
};
use radix_engine_interface::constants::CONSENSUS_MANAGER;
use radix_engine_interface::data::manifest::model::ManifestExpression;
use radix_engine_interface::data::manifest::to_manifest_value;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_interface::schema::{BlueprintSchema, FunctionSchema, PackageSchema};
use radix_engine_interface::time::Instant;
use radix_engine_interface::{dec, rule};
use radix_engine_queries::query::{ResourceAccounter, StateTreeTraverser, VaultFinder};
use radix_engine_store_interface::{
    db_key_mapper::{
        MappedCommittableSubstateDatabase, MappedSubstateDatabase, SpreadPrefixKeyMapper,
    },
    interface::{CommittableSubstateDatabase, DatabaseUpdate, DatabaseUpdates},
};
use radix_engine_stores::hash_tree::tree_store::{TypedInMemoryTreeStore, Version};
use radix_engine_stores::hash_tree::{put_at_next_version, SubstateHashChange};
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use sbor::basic_well_known_types::{ANY_ID, UNIT_ID};
use scrypto::modules::Mutability::*;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionManifestV1;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::{
    AttachmentsV1, BlobV1, Executable, InstructionV1, IntentV1, PreviewFlags, PreviewIntentV1,
    SystemTransactionV1, TestTransaction, TransactionHeaderV1, TransactionPayload,
};
use transaction::prelude::{BlobsV1, InstructionsV1};

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
    pub genesis_data_chunks: Vec<(Vec<(BlueprintId, GlobalAddress)>, GenesisDataChunk)>,
    pub initial_epoch: Epoch,
    pub initial_config: ConsensusManagerConfig,
    pub initial_time_ms: i64,
}

impl CustomGenesis {
    pub fn default(initial_epoch: Epoch, initial_config: ConsensusManagerConfig) -> CustomGenesis {
        let pub_key = EcdsaSecp256k1PrivateKey::from_u64(1u64)
            .unwrap()
            .public_key();
        Self::single_validator_and_staker(
            pub_key,
            Decimal::one(),
            ComponentAddress::virtual_account_from_public_key(&pub_key),
            initial_epoch,
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
        }
    }

    pub fn single_validator_and_staker(
        validator_public_key: EcdsaSecp256k1PublicKey,
        stake_xrd_amount: Decimal,
        staker_account: ComponentAddress,
        initial_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
    ) -> CustomGenesis {
        let genesis_validator: GenesisValidator = validator_public_key.clone().into();
        let genesis_data_chunks = vec![
            (
                vec![],
                GenesisDataChunk::Validators(vec![genesis_validator]),
            ),
            (
                vec![],
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
            ),
        ];
        CustomGenesis {
            genesis_data_chunks,
            initial_epoch,
            initial_config,
            initial_time_ms: 0,
        }
    }
}

pub struct TestRunnerBuilder {
    custom_genesis: Option<CustomGenesis>,
    trace: bool,
    state_hashing: bool,
}

impl TestRunnerBuilder {
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

    pub fn build_and_get_epoch(self) -> (TestRunner, ActiveValidatorSet) {
        let scrypto_interpreter = ScryptoVm {
            wasm_engine: DefaultWasmEngine::default(),
            wasm_instrumenter: WasmInstrumenter::default(),
            wasm_metering_config: WasmMeteringConfig::V0,
        };
        let mut substate_db = InMemorySubstateDatabase::standard();

        let mut bootstrapper = Bootstrapper::new(&mut substate_db, &scrypto_interpreter, false);
        let GenesisReceipts {
            wrap_up_receipt, ..
        } = match self.custom_genesis {
            Some(custom_genesis) => bootstrapper
                .bootstrap_with_genesis_data(
                    custom_genesis.genesis_data_chunks,
                    custom_genesis.initial_epoch,
                    custom_genesis.initial_config,
                    custom_genesis.initial_time_ms,
                )
                .unwrap(),
            None => bootstrapper.bootstrap_test_default().unwrap(),
        };

        // Note that 0 is not a valid private key
        let next_private_key = 100;

        // Starting from non-zero considering that bootstrap might have used a few.
        let next_transaction_nonce = 100;

        let runner = TestRunner {
            scrypto_interpreter,
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

    pub fn build(self) -> TestRunner {
        self.build_and_get_epoch().0
    }
}

pub struct TestRunner {
    scrypto_interpreter: ScryptoVm<DefaultWasmEngine>,
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

impl TestRunner {
    pub fn builder() -> TestRunnerBuilder {
        TestRunnerBuilder {
            custom_genesis: None,
            #[cfg(not(feature = "resource_tracker"))]
            trace: true,
            #[cfg(feature = "resource_tracker")]
            trace: false,
            state_hashing: false,
        }
    }

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

    pub fn new_key_pair(&mut self) -> (EcdsaSecp256k1PublicKey, EcdsaSecp256k1PrivateKey) {
        let private_key = EcdsaSecp256k1PrivateKey::from_u64(self.next_private_key()).unwrap();
        let public_key = private_key.public_key();

        (public_key, private_key)
    }

    pub fn new_key_pair_with_auth_address(
        &mut self,
    ) -> (
        EcdsaSecp256k1PublicKey,
        EcdsaSecp256k1PrivateKey,
        NonFungibleGlobalId,
    ) {
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
            .lock_fee(self.faucet_component(), 100u32.into())
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
            .get_mapped::<SpreadPrefixKeyMapper, Option<MetadataValue>>(
                address.as_node_id(),
                METADATA_KV_STORE_PARTITION,
                &SubstateKey::Map(key),
            )?;

        metadata_value
    }

    pub fn inspect_component_royalty(
        &mut self,
        component_address: ComponentAddress,
    ) -> Option<Decimal> {
        if let Some(output) = self
            .substate_db
            .get_mapped::<SpreadPrefixKeyMapper, ComponentRoyaltyAccumulatorSubstate>(
                component_address.as_node_id(),
                ROYALTY_FIELD_PARTITION,
                &RoyaltyField::RoyaltyAccumulator.into(),
            )
        {
            output
                .royalty_vault
                .and_then(|vault| {
                    self.substate_db
                        .get_mapped::<SpreadPrefixKeyMapper, LiquidFungibleResource>(
                            vault.as_node_id(),
                            OBJECT_BASE_PARTITION,
                            &FungibleVaultField::LiquidFungible.into(),
                        )
                })
                .map(|r| r.amount())
        } else {
            None
        }
    }

    pub fn inspect_package_royalty(&mut self, package_address: PackageAddress) -> Option<Decimal> {
        if let Some(output) = self
            .substate_db
            .get_mapped::<SpreadPrefixKeyMapper, PackageRoyaltySubstate>(
                package_address.as_node_id(),
                OBJECT_BASE_PARTITION,
                &PackageField::Royalty.into(),
            )
        {
            output
                .royalty_vault
                .and_then(|vault| {
                    self.substate_db
                        .get_mapped::<SpreadPrefixKeyMapper, LiquidFungibleResource>(
                            vault.as_node_id(),
                            OBJECT_BASE_PARTITION,
                            &FungibleVaultField::LiquidFungible.into(),
                        )
                })
                .map(|r| r.amount())
        } else {
            None
        }
    }

    pub fn account_balance(
        &mut self,
        account_address: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> Option<Decimal> {
        let vaults = self.get_component_vaults(account_address, resource_address);
        vaults
            .get(0)
            .map_or(None, |vault_id| self.inspect_vault_balance(*vault_id))
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
            .get_mapped::<SpreadPrefixKeyMapper, LiquidFungibleResource>(
                &vault_id,
                OBJECT_BASE_PARTITION,
                &FungibleVaultField::LiquidFungible.into(),
            )
            .map(|output| output.amount())
    }

    pub fn inspect_non_fungible_vault(
        &mut self,
        vault_id: NodeId,
    ) -> Option<(Decimal, Option<NonFungibleLocalId>)> {
        let amount = self
            .substate_db()
            .get_mapped::<SpreadPrefixKeyMapper, LiquidNonFungibleVault>(
                &vault_id,
                OBJECT_BASE_PARTITION,
                &NonFungibleVaultField::LiquidNonFungible.into(),
            )
            .map(|vault| vault.amount);

        let mut substate_iter = self
            .substate_db()
            .list_mapped::<SpreadPrefixKeyMapper, NonFungibleLocalId, MapKey>(
                &vault_id,
                OBJECT_BASE_PARTITION
                    .at_offset(PartitionOffset(1u8))
                    .unwrap(),
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
            .lock_fee(self.faucet_component(), 100u32.into())
            .call_method(self.faucet_component(), "free", manifest_args!())
            .take_all_from_worktop(RADIX_TOKEN, |builder, bucket| {
                builder.call_method(
                    account_address,
                    ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
                    manifest_args!(bucket),
                )
            })
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
    }

    pub fn new_account_advanced(&mut self, owner_rule: OwnerRole) -> ComponentAddress {
        let manifest = ManifestBuilder::new()
            .new_account_advanced(owner_rule)
            .build();
        let receipt = self.execute_manifest_ignoring_fee(manifest, vec![]);
        receipt.expect_commit_success();

        let account = receipt.expect_commit(true).new_component_addresses()[0];

        let manifest = ManifestBuilder::new()
            .call_method(self.faucet_component(), "free", manifest_args!())
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.execute_manifest_ignoring_fee(manifest, vec![]);
        receipt.expect_commit_success();

        account
    }

    pub fn new_virtual_account(
        &mut self,
    ) -> (
        EcdsaSecp256k1PublicKey,
        EcdsaSecp256k1PrivateKey,
        ComponentAddress,
    ) {
        let (pub_key, priv_key) = self.new_key_pair();
        let account = ComponentAddress::virtual_account_from_public_key(
            &PublicKey::EcdsaSecp256k1(pub_key.clone()),
        );
        self.load_account_from_faucet(account);
        (pub_key, priv_key, account)
    }

    pub fn get_active_validator_info_by_key(
        &self,
        key: &EcdsaSecp256k1PublicKey,
    ) -> ValidatorSubstate {
        let address = self.get_active_validator_with_key(key);
        self.get_validator_info(address)
    }

    pub fn get_validator_info(&self, address: ComponentAddress) -> ValidatorSubstate {
        self.substate_db()
            .get_mapped::<SpreadPrefixKeyMapper, ValidatorSubstate>(
                address.as_node_id(),
                OBJECT_BASE_PARTITION,
                &ValidatorField::Validator.into(),
            )
            .unwrap()
    }

    pub fn get_active_validator_with_key(&self, key: &EcdsaSecp256k1PublicKey) -> ComponentAddress {
        let substate = self
            .substate_db()
            .get_mapped::<SpreadPrefixKeyMapper, CurrentValidatorSetSubstate>(
                CONSENSUS_MANAGER.as_node_id(),
                OBJECT_BASE_PARTITION,
                &ConsensusManagerField::CurrentValidatorSet.into(),
            )
            .unwrap();

        substate
            .validator_set
            .get_by_public_key(key)
            .unwrap()
            .0
            .clone()
    }

    pub fn new_allocated_account(
        &mut self,
    ) -> (
        EcdsaSecp256k1PublicKey,
        EcdsaSecp256k1PrivateKey,
        ComponentAddress,
    ) {
        let key_pair = self.new_key_pair();
        let withdraw_auth = rule!(require(NonFungibleGlobalId::from_public_key(&key_pair.0)));
        let account = self.new_account_advanced(OwnerRole::Fixed(withdraw_auth));
        (key_pair.0, key_pair.1, account)
    }

    pub fn new_account(
        &mut self,
        is_virtual: bool,
    ) -> (
        EcdsaSecp256k1PublicKey,
        EcdsaSecp256k1PrivateKey,
        ComponentAddress,
    ) {
        if is_virtual {
            self.new_virtual_account()
        } else {
            self.new_allocated_account()
        }
    }

    pub fn new_identity(
        &mut self,
        pk: EcdsaSecp256k1PublicKey,
        is_virtual: bool,
    ) -> ComponentAddress {
        if is_virtual {
            ComponentAddress::virtual_identity_from_public_key(&pk)
        } else {
            let owner_id = NonFungibleGlobalId::from_public_key(&pk);
            let manifest = ManifestBuilder::new()
                .lock_fee(self.faucet_component(), 10.into())
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
            .lock_fee(self.faucet_component(), 10.into())
            .create_identity()
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit_success();
        let component_address = receipt.expect_commit(true).new_component_addresses()[0];

        component_address
    }

    pub fn new_validator_with_pub_key(
        &mut self,
        pub_key: EcdsaSecp256k1PublicKey,
        account: ComponentAddress,
    ) -> ComponentAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 10.into())
            .create_validator(pub_key)
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        let address = receipt.expect_commit(true).new_component_addresses()[0];
        address
    }

    pub fn publish_package(
        &mut self,
        code: Vec<u8>,
        definition: PackageDefinition,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, MetadataValue>,
        owner_rule: OwnerRole,
    ) -> PackageAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 100u32.into())
            .publish_package_advanced(code, definition, royalty_config, metadata, owner_rule)
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
            .lock_fee(self.faucet_component(), 100u32.into())
            .publish_package_with_owner(code, definition, owner_badge)
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_package_addresses()[0]
    }

    pub fn compile_and_publish<P: AsRef<Path>>(&mut self, package_dir: P) -> PackageAddress {
        let (code, definition) = Compile::compile(package_dir);
        self.publish_package(
            code,
            definition,
            BTreeMap::new(),
            BTreeMap::new(),
            OwnerRole::None,
        )
    }

    pub fn compile_and_publish_retain_blueprints<
        P: AsRef<Path>,
        F: FnMut(&String, &mut BlueprintSchema) -> bool,
    >(
        &mut self,
        package_dir: P,
        retain: F,
    ) -> PackageAddress {
        let (code, mut definition) = Compile::compile(package_dir);
        definition.schema.blueprints.retain(retain);
        self.publish_package(
            code,
            definition,
            BTreeMap::new(),
            BTreeMap::new(),
            OwnerRole::None,
        )
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
                address: self.faucet_component(),
                method_name: "lock_fee".to_string(),
                args: manifest_args!(dec!("100")),
            },
        );
        self.execute_manifest(manifest, initial_proofs)
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
        self.execute_transaction_with_config(
            TestTransaction::new_from_nonce(manifest, nonce)
                .prepare()
                .expect("expected transaction to be preparable")
                .get_executable(initial_proofs.into_iter().collect()),
            &FeeReserveConfig::default(),
            &ExecutionConfig::default()
                .with_kernel_trace(self.trace)
                .with_cost_unit_limit(cost_unit_limit),
        )
    }

    pub fn execute_transaction(&mut self, executable: Executable) -> TransactionReceipt {
        let fee_config = FeeReserveConfig::default();
        let execution_config = ExecutionConfig::default().with_kernel_trace(self.trace);

        self.execute_transaction_with_config(executable, &fee_config, &execution_config)
    }

    pub fn execute_transaction_with_config(
        &mut self,
        executable: Executable,
        fee_reserve_config: &FeeReserveConfig,
        execution_config: &ExecutionConfig,
    ) -> TransactionReceipt {
        let transaction_receipt = execute_transaction(
            &mut self.substate_db,
            &self.scrypto_interpreter,
            fee_reserve_config,
            execution_config,
            &executable,
        );
        if let TransactionResult::Commit(commit) = &transaction_receipt.result {
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
        execute_preview(
            &self.substate_db,
            &mut self.scrypto_interpreter,
            network,
            preview_intent,
            self.trace,
        )
    }

    pub fn preview_manifest(
        &mut self,
        manifest: TransactionManifestV1,
        signer_public_keys: Vec<PublicKey>,
        tip_percentage: u16,
        flags: PreviewFlags,
    ) -> TransactionReceipt {
        let epoch = self.get_current_epoch();
        execute_preview(
            &mut self.substate_db,
            &self.scrypto_interpreter,
            &NetworkDefinition::simulator(),
            PreviewIntentV1 {
                intent: IntentV1 {
                    header: TransactionHeaderV1 {
                        network_id: NetworkDefinition::simulator().id,
                        start_epoch_inclusive: epoch,
                        end_epoch_exclusive: epoch.after(10),
                        nonce: 0,
                        notary_public_key: PublicKey::EcdsaSecp256k1(EcdsaSecp256k1PublicKey(
                            [0u8; 33],
                        )),
                        notary_is_signatory: false,
                        tip_percentage,
                    },
                    instructions: InstructionsV1(manifest.instructions),
                    blobs: BlobsV1 {
                        blobs: manifest.blobs.values().map(|x| BlobV1(x.clone())).collect(),
                    },
                    attachments: AttachmentsV1 {},
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
    /// Notes:
    /// * Buckets and signatures are not supported - instead use `execute_manifest_ignoring_fee` and `ManifestBuilder` directly.
    /// * Call `.expect_commit_success()` on the receipt to get access to receipt details.
    pub fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: ManifestValue,
    ) -> TransactionReceipt {
        self.execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_function(package_address, blueprint_name, function_name, args)
                .build(),
            vec![],
        )
    }

    /// Calls a package blueprint function with the given arguments, and assumes it constructs a single component successfully.
    /// It returns the address of the first created component.
    ///
    /// Notes:
    /// * Buckets and signatures are not supported - instead use `execute_manifest_ignoring_fee` and `ManifestBuilder` directly.
    pub fn construct_new(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: ManifestValue,
    ) -> ComponentAddress {
        self.call_function(package_address, blueprint_name, function_name, args)
            .expect_commit_success()
            .new_component_addresses()[0]
    }

    /// Calls a component method with the given arguments, paying the fee from the faucet.
    ///
    /// Notes:
    /// * Buckets and signatures are not supported - instead use `execute_manifest_ignoring_fee` and `ManifestBuilder` directly.
    /// * Call `.expect_commit_success()` on the receipt to get access to receipt details.
    pub fn call_method(
        &mut self,
        component_address: ComponentAddress,
        method_name: &str,
        args: ManifestValue,
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
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
        to: ComponentAddress,
    ) -> ResourceAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 100u32.into())
            .create_fungible_resource(0, BTreeMap::new(), access_rules, Some(5.into()))
            .call_method(
                to,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
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
        ResourceAddress,
    ) {
        let mint_auth = self.create_non_fungible_resource(account);
        let burn_auth = self.create_non_fungible_resource(account);
        let withdraw_auth = self.create_non_fungible_resource(account);
        let recall_auth = self.create_non_fungible_resource(account);
        let update_metadata_auth = self.create_non_fungible_resource(account);
        let freeze_auth = self.create_non_fungible_resource(account);
        let unfreeze_auth = self.create_non_fungible_resource(account);
        let admin_auth = self.create_non_fungible_resource(account);

        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            Mint,
            (
                rule!(require(mint_auth)),
                MUTABLE(rule!(require(admin_auth))),
            ),
        );
        access_rules.insert(
            Burn,
            (
                rule!(require(burn_auth)),
                MUTABLE(rule!(require(admin_auth))),
            ),
        );
        access_rules.insert(
            Withdraw,
            (
                rule!(require(withdraw_auth)),
                MUTABLE(rule!(require(admin_auth))),
            ),
        );
        access_rules.insert(
            Recall,
            (
                rule!(require(recall_auth)),
                MUTABLE(rule!(require(admin_auth))),
            ),
        );
        access_rules.insert(
            UpdateMetadata,
            (
                rule!(require(update_metadata_auth)),
                MUTABLE(rule!(require(admin_auth))),
            ),
        );
        access_rules.insert(
            Freeze,
            (
                rule!(require(freeze_auth)),
                MUTABLE(rule!(require(admin_auth))),
            ),
        );
        access_rules.insert(
            Unfreeze,
            (
                rule!(require(unfreeze_auth)),
                MUTABLE(rule!(require(admin_auth))),
            ),
        );
        access_rules.insert(
            Deposit,
            (rule!(allow_all), MUTABLE(rule!(require(admin_auth)))),
        );

        let token_address = self.create_fungible_resource_and_deposit(access_rules, account);

        (
            token_address,
            mint_auth,
            burn_auth,
            withdraw_auth,
            recall_auth,
            update_metadata_auth,
            freeze_auth,
            unfreeze_auth,
            admin_auth,
        )
    }

    pub fn create_freezeable_token(&mut self, account: ComponentAddress) -> ResourceAddress {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(Deposit, (rule!(allow_all), LOCKED));
        access_rules.insert(Recall, (rule!(allow_all), LOCKED));
        access_rules.insert(Freeze, (rule!(allow_all), LOCKED));
        access_rules.insert(Unfreeze, (rule!(allow_all), LOCKED));

        self.create_fungible_resource_and_deposit(access_rules, account)
    }

    pub fn create_recallable_token(&mut self, account: ComponentAddress) -> ResourceAddress {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceMethodAuthKey::Recall, (rule!(allow_all), LOCKED));

        self.create_fungible_resource_and_deposit(access_rules, account)
    }

    pub fn create_restricted_burn_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_non_fungible_resource(account);

        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));
        access_rules.insert(Burn, (rule!(require(auth_resource_address)), LOCKED));
        let resource_address = self.create_fungible_resource_and_deposit(access_rules, account);

        (auth_resource_address, resource_address)
    }

    pub fn create_restricted_transfer_token(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let auth_resource_address = self.create_non_fungible_resource(account);

        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(require(auth_resource_address)), LOCKED),
        );
        access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));
        let resource_address = self.create_fungible_resource_and_deposit(access_rules, account);

        (auth_resource_address, resource_address)
    }

    pub fn create_non_fungible_resource(&mut self, account: ComponentAddress) -> ResourceAddress {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));

        let mut entries = BTreeMap::new();
        entries.insert(NonFungibleLocalId::integer(1), EmptyNonFungibleData {});
        entries.insert(NonFungibleLocalId::integer(2), EmptyNonFungibleData {});
        entries.insert(NonFungibleLocalId::integer(3), EmptyNonFungibleData {});

        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 100u32.into())
            .create_non_fungible_resource(
                NonFungibleIdType::Integer,
                BTreeMap::new(),
                access_rules,
                Some(entries),
            )
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
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
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(ResourceMethodAuthKey::Deposit, (rule!(allow_all), LOCKED));
        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 100u32.into())
            .create_fungible_resource(divisibility, BTreeMap::new(), access_rules, Some(amount))
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    pub fn create_mintable_burnable_fungible_resource(
        &mut self,
        account: ComponentAddress,
    ) -> (ResourceAddress, ResourceAddress) {
        let admin_auth = self.create_non_fungible_resource(account);

        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(Deposit, (rule!(allow_all), LOCKED));
        access_rules.insert(Mint, (rule!(require(admin_auth)), LOCKED));
        access_rules.insert(Burn, (rule!(require(admin_auth)), LOCKED));
        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 100u32.into())
            .create_fungible_resource(1u8, BTreeMap::new(), access_rules, None)
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        let resource_address = receipt.expect_commit(true).new_resource_addresses()[0];
        (admin_auth, resource_address)
    }

    pub fn create_freely_mintable_fungible_resource(
        &mut self,
        amount: Option<Decimal>,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(Deposit, (rule!(allow_all), LOCKED));
        access_rules.insert(Mint, (rule!(allow_all), LOCKED));
        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 100u32.into())
            .create_fungible_resource(divisibility, BTreeMap::new(), access_rules, amount)
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build();
        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    pub fn create_freely_mintable_and_burnable_fungible_resource(
        &mut self,
        amount: Option<Decimal>,
        divisibility: u8,
        account: ComponentAddress,
    ) -> ResourceAddress {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(Deposit, (rule!(allow_all), LOCKED));
        access_rules.insert(Mint, (rule!(allow_all), LOCKED));
        access_rules.insert(Burn, (rule!(allow_all), LOCKED));
        let manifest = ManifestBuilder::new()
            .lock_fee(self.faucet_component(), 100u32.into())
            .create_fungible_resource(divisibility, BTreeMap::new(), access_rules, amount)
            .call_method(
                account,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
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
        F: FnOnce(&mut ManifestBuilder) -> &mut ManifestBuilder,
    {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.faucet_component(),
                "lock_fee",
                manifest_args!(dec!("10")),
            )
            .borrow_mut(|builder| Result::<_, Infallible>::Ok(handler(builder)))
            .unwrap()
            .build();

        let receipt = self.execute_manifest(manifest, initial_proofs);
        receipt.expect_commit(true).new_component_addresses()[0]
    }

    pub fn set_current_epoch(&mut self, epoch: Epoch) {
        let mut substate = self
            .substate_db
            .get_mapped::<SpreadPrefixKeyMapper, ConsensusManagerSubstate>(
                &CONSENSUS_MANAGER.as_node_id(),
                OBJECT_BASE_PARTITION,
                &ConsensusManagerField::ConsensusManager.into(),
            )
            .unwrap();
        substate.epoch = epoch;
        self.substate_db.put_mapped::<SpreadPrefixKeyMapper, _>(
            &CONSENSUS_MANAGER.as_node_id(),
            OBJECT_BASE_PARTITION,
            &ConsensusManagerField::ConsensusManager.into(),
            &substate,
        );
    }

    pub fn get_current_epoch(&mut self) -> Epoch {
        let receipt = self.execute_system_transaction(
            vec![InstructionV1::CallMethod {
                address: CONSENSUS_MANAGER.into(),
                method_name: CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT.to_string(),
                args: to_manifest_value(&ConsensusManagerGetCurrentEpochInput),
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
        pre_allocated_addresses: Vec<(BlueprintId, GlobalAddress)>,
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
        pre_allocated_addresses: Vec<(BlueprintId, GlobalAddress)>,
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
        )
    }

    /// Executes a "start round number `round` at timestamp `timestamp_ms`" system transaction, as
    /// if it was proposed by the first validator from the validator set, after `round - 1` missed
    /// rounds by that validator.
    /// Please note that this assumes that state is right at the beginning of an epoch.
    pub fn advance_to_round_at_timestamp(
        &mut self,
        round: Round,
        proposer_timestamp_ms: i64,
    ) -> TransactionReceipt {
        self.execute_system_transaction(
            vec![InstructionV1::CallMethod {
                address: CONSENSUS_MANAGER.into(),
                method_name: CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
                args: to_manifest_value(&ConsensusManagerNextRoundInput {
                    round,
                    proposer_timestamp_ms,
                    leader_proposal_history: LeaderProposalHistory {
                        gap_round_leaders: (1..round.number()).map(|_| 0).collect(),
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
            .get_mapped::<SpreadPrefixKeyMapper, ProposerMilliTimestampSubstate>(
                CONSENSUS_MANAGER.as_node_id(),
                OBJECT_BASE_PARTITION,
                &ConsensusManagerField::CurrentTime.into(),
            )
            .unwrap()
            .epoch_milli
    }

    pub fn get_current_time(&mut self, precision: TimePrecision) -> Instant {
        let receipt = self.execute_system_transaction(
            vec![InstructionV1::CallMethod {
                address: CONSENSUS_MANAGER.into(),
                method_name: CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT.to_string(),
                args: to_manifest_value(&ConsensusManagerGetCurrentTimeInput { precision }),
            }],
            btreeset![AuthAddresses::validator_role()],
        );
        receipt.expect_commit(true).output(0)
    }

    pub fn event_schema(
        &self,
        event_type_identifier: &EventTypeIdentifier,
    ) -> (LocalTypeIndex, ScryptoSchema) {
        let (package_address, blueprint_name, local_type_index) = match event_type_identifier {
            EventTypeIdentifier(Emitter::Method(node_id, node_module), local_type_index) => {
                match node_module {
                    ObjectModuleId::AccessRules => (
                        ACCESS_RULES_MODULE_PACKAGE,
                        ACCESS_RULES_BLUEPRINT.into(),
                        local_type_index.clone(),
                    ),
                    ObjectModuleId::Royalty => (
                        ROYALTY_MODULE_PACKAGE,
                        COMPONENT_ROYALTY_BLUEPRINT.into(),
                        local_type_index.clone(),
                    ),
                    ObjectModuleId::Metadata => (
                        METADATA_MODULE_PACKAGE,
                        METADATA_BLUEPRINT.into(),
                        local_type_index.clone(),
                    ),
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
                            TypeInfoSubstate::Object(ObjectInfo { blueprint, .. }) => (
                                blueprint.package_address,
                                blueprint.blueprint_name,
                                *local_type_index,
                            ),
                            _ => {
                                panic!("No event schema.")
                            }
                        }
                    }
                }
            }
            EventTypeIdentifier(
                Emitter::Function(node_id, _, blueprint_name),
                local_type_index,
            ) => (
                PackageAddress::new_or_panic(node_id.0),
                blueprint_name.to_owned(),
                local_type_index.clone(),
            ),
        };

        (
            local_type_index,
            self.substate_db()
                .get_mapped::<SpreadPrefixKeyMapper, PackageInfoSubstate>(
                    package_address.as_node_id(),
                    OBJECT_BASE_PARTITION,
                    &PackageField::Info.into(),
                )
                .unwrap()
                .schema
                .blueprints
                .remove(&blueprint_name)
                .unwrap()
                .schema,
        )
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
    matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(_)))
}

pub fn is_costing_error(e: &RuntimeError) -> bool {
    matches!(e, RuntimeError::ModuleError(ModuleError::CostingError(_)))
}

pub fn is_wasm_error(e: &RuntimeError) -> bool {
    matches!(
        e,
        RuntimeError::KernelError(KernelError::WasmRuntimeError(..))
    )
}

pub fn wat2wasm(wat: &str) -> Vec<u8> {
    wabt::wat2wasm(
        wat.replace("${memcpy}", include_str!("snippets/memcpy.wat"))
            .replace("${memmove}", include_str!("snippets/memmove.wat"))
            .replace("${memset}", include_str!("snippets/memset.wat")),
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
    let mut package_schema = PackageSchema::default();
    package_schema.blueprints.insert(
        blueprint_name.to_string(),
        BlueprintSchema {
            outer_blueprint: None,
            schema: ScryptoSchema {
                type_kinds: vec![],
                type_metadata: vec![],
                type_validations: vec![],
            },
            fields: vec![LocalTypeIndex::WellKnown(UNIT_ID)],
            collections: vec![],
            functions: btreemap!(
                function_name.to_string() => FunctionSchema {
                    receiver: Option::None,
                    input: LocalTypeIndex::WellKnown(ANY_ID),
                    output: LocalTypeIndex::WellKnown(ANY_ID),
                    export_name: format!("{}_{}", blueprint_name, function_name),
                }
            ),
            virtual_lazy_load_functions: btreemap!(),
            event_schema: [].into(),
            dependencies: btreeset!(),
            method_auth_template: btreemap!(),
            outer_method_auth_template: btreemap!(),
        },
    );
    PackageDefinition {
        schema: package_schema,
        function_access_rules: btreemap!(
            blueprint_name.to_string() => btreemap!(function_name.to_string() => rule!(allow_all))
        ),
    }
}

#[derive(ScryptoSbor, NonFungibleData, ManifestSbor)]
struct EmptyNonFungibleData {}
