use crate::blueprints::access_controller::*;
use crate::blueprints::account::{AccountNativePackage, AccountOwnerBadgeData};
use crate::blueprints::consensus_manager::ConsensusManagerNativePackage;
use crate::blueprints::identity::{IdentityNativePackage, IdentityOwnerBadgeData};
use crate::blueprints::package::{
    create_package_partition_substates, PackageCollection, PackageNativePackage,
    PackageOwnerBadgeData, SystemInstruction,
};
use crate::blueprints::pool::v1::package::{PoolNativePackage, PoolV1MinorVersion};
use crate::blueprints::resource::ResourceNativePackage;
use crate::blueprints::test_utils::TestUtilsNativePackage;
use crate::blueprints::transaction_processor::TransactionProcessorNativePackage;
use crate::blueprints::transaction_tracker::{
    TransactionTrackerNativePackage, TRANSACTION_TRACKER_CREATE_IDENT,
};
use crate::internal_prelude::*;
use crate::object_modules::metadata::MetadataNativePackage;
use crate::object_modules::role_assignment::RoleAssignmentNativePackage;
use crate::object_modules::royalty::RoyaltyNativePackage;
use crate::system::system_db_reader::SystemDatabaseReader;
use crate::system::type_info::TypeInfoSubstate;
use crate::track::{
    BatchPartitionStateUpdate, LegacyStateUpdates, NodeStateUpdates, PartitionStateUpdates,
    StateUpdates,
};
use crate::transaction::{
    execute_transaction, CommitResult, ExecutionConfig, StateUpdateSummary, SubstateSchemaMapper,
    SubstateSystemStructures, TransactionOutcome, TransactionReceipt, TransactionResult,
};
use crate::vm::wasm::WasmEngine;
use crate::vm::{NativeVmExtension, VmBoot, VmInit};
use lazy_static::lazy_static;
use radix_common::constants::AuthAddresses;
use radix_common::crypto::Secp256k1PublicKey;
use radix_common::math::traits::*;
use radix_common::types::ComponentAddress;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerConfig, ConsensusManagerCreateManifestInput, EpochChangeCondition,
    CONSENSUS_MANAGER_BLUEPRINT, CONSENSUS_MANAGER_CREATE_IDENT,
};
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::{MetadataValue, UncheckedUrl};
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::{
    burn_roles, metadata, metadata_init, mint_roles, rule, withdraw_roles,
};
use radix_substate_store_interface::interface::{
    DatabaseUpdate, DatabaseUpdates, DbPartitionKey, DbSortKey, DbSubstateValue,
    PartitionDatabaseUpdates, PartitionEntry,
};
use radix_substate_store_interface::{
    db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper},
    interface::{CommittableSubstateDatabase, SubstateDatabase},
};
use radix_transactions::model::{
    BlobsV1, InstructionV1, InstructionsV1, SystemTransactionV1, TransactionPayload,
};
use radix_transactions::prelude::{BlobV1, PreAllocatedAddress};
use radix_transactions::validation::ManifestIdAllocator;

lazy_static! {
    pub static ref DEFAULT_TESTING_FAUCET_SUPPLY: Decimal = dec!("100000000000000000");
    pub static ref DEFAULT_VALIDATOR_USD_COST: Decimal = dec!("100");
    pub static ref DEFAULT_VALIDATOR_XRD_COST: Decimal = DEFAULT_VALIDATOR_USD_COST
        .checked_mul(Decimal::try_from(USD_PRICE_IN_XRD).unwrap())
        .unwrap();  // NOTE: Decimal arithmetic operation safe unwrap.
                    // No chance to overflow.
                    // The chance to overflow will be decreasing over time since USD price in XRD will only get lower ;)
}

//==========================================================================================
// GENESIS CHUNK MODELS
// - These are used by the node (and in Java) so they need to implement ScryptoEncode so
//   that they can go over the JNI boundary
// - The models which use ManifestSbor are also included in the transaction itself, and must
//   match the corresponding models in the `genesis_helper` component
//==========================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoSbor)]
pub struct GenesisValidator {
    pub key: Secp256k1PublicKey,
    pub accept_delegated_stake: bool,
    pub is_registered: bool,
    pub fee_factor: Decimal,
    pub metadata: Vec<(String, MetadataValue)>,
    pub owner: ComponentAddress,
}

impl From<Secp256k1PublicKey> for GenesisValidator {
    fn from(key: Secp256k1PublicKey) -> Self {
        // Re-using the validator key for its owner
        let default_owner_address = ComponentAddress::virtual_account_from_public_key(&key);
        GenesisValidator {
            key,
            accept_delegated_stake: true,
            is_registered: true,
            fee_factor: Decimal::ONE,
            metadata: vec![(
                "url".to_string(),
                MetadataValue::Url(UncheckedUrl::of(format!(
                    "http://test.local?validator={:?}",
                    key
                ))),
            )],
            owner: default_owner_address,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoSbor)]
pub struct GenesisStakeAllocation {
    pub account_index: u32,
    pub xrd_amount: Decimal,
}

// Note - this gets mapped into the ManifestGenesisResource by replacing the reservation
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisResource {
    pub reserved_resource_address: ResourceAddress,
    pub metadata: Vec<(String, MetadataValue)>,
    pub owner: Option<ComponentAddress>,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoSbor)]
pub struct GenesisResourceAllocation {
    pub account_index: u32,
    pub amount: Decimal,
}

// Note - this gets mapped into the ManifestGenesisResource for inclusion in the transaction
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum GenesisDataChunk {
    Validators(Vec<GenesisValidator>),
    Stakes {
        accounts: Vec<ComponentAddress>,
        allocations: Vec<(Secp256k1PublicKey, Vec<GenesisStakeAllocation>)>,
    },
    Resources(Vec<GenesisResource>),
    ResourceBalances {
        accounts: Vec<ComponentAddress>,
        allocations: Vec<(ResourceAddress, Vec<GenesisResourceAllocation>)>,
    },
    XrdBalances(Vec<(ComponentAddress, Decimal)>),
}

//==========================================================================================
// MANIFEST-SPECIFIC GENESIS CHUNK MODELS
// - These must match the corresponding models in the `genesis_helper` component
//==========================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub enum ManifestGenesisDataChunk {
    Validators(Vec<GenesisValidator>),
    Stakes {
        accounts: Vec<ComponentAddress>,
        allocations: Vec<(Secp256k1PublicKey, Vec<GenesisStakeAllocation>)>,
    },
    Resources(Vec<ManifestGenesisResource>),
    ResourceBalances {
        accounts: Vec<ComponentAddress>,
        allocations: Vec<(ResourceAddress, Vec<GenesisResourceAllocation>)>,
    },
    XrdBalances(Vec<(ComponentAddress, Decimal)>),
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct ManifestGenesisResource {
    pub resource_address_reservation: ManifestAddressReservation,
    pub metadata: Vec<(String, MetadataValue)>,
    pub owner: Option<ComponentAddress>,
}

//==========================================================================================
// BOOTSTRAPPER
// Various helper utilities for constructing and executing genesis
//==========================================================================================

#[derive(Debug, Clone, ScryptoSbor)]
pub struct GenesisReceipts {
    pub system_bootstrap_receipt: TransactionReceipt,
    pub data_ingestion_receipts: Vec<TransactionReceipt>,
    pub wrap_up_receipt: TransactionReceipt,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct FlashReceipt {
    pub state_updates: StateUpdates,
    pub state_update_summary: StateUpdateSummary,
    pub substate_system_structures: SubstateSystemStructures,
}

impl From<FlashReceipt> for TransactionReceipt {
    fn from(value: FlashReceipt) -> Self {
        // This is used by the node for allowing the flash to execute before the
        // genesis bootstrap transaction
        let commit_result = CommitResult::empty_with_outcome(TransactionOutcome::Success(vec![]));
        let mut transaction_receipt = TransactionReceipt::empty_with_commit(commit_result);
        value.merge_genesis_flash_into_transaction_receipt(&mut transaction_receipt);
        transaction_receipt
    }
}

impl FlashReceipt {
    // Merge system_flash_receipt into system_bootstrap_receipt
    // This is currently a necessary hack in order to not change GenesisReceipt with
    // the addition of a new system_flash_receipt.
    pub fn merge_genesis_flash_into_transaction_receipt(self, receipt: &mut TransactionReceipt) {
        match &mut receipt.result {
            TransactionResult::Commit(result) => {
                let mut new_packages = self.state_update_summary.new_packages;
                new_packages.extend(result.state_update_summary.new_packages.drain(..));
                let mut new_components = self.state_update_summary.new_components;
                new_components.extend(result.state_update_summary.new_components.drain(..));
                let mut new_resources = self.state_update_summary.new_resources;
                new_resources.extend(result.state_update_summary.new_resources.drain(..));

                result.state_update_summary.new_packages = new_packages;
                result.state_update_summary.new_components = new_components;
                result.state_update_summary.new_resources = new_resources;

                merge_asserting_no_overlap(&mut result.state_updates, self.state_updates);

                let mut substate_system_structures = self.substate_system_structures;
                for (node_id, by_partition_num) in
                    result.system_structure.substate_system_structures.drain(..)
                {
                    let merged_by_partition_num = substate_system_structures
                        .entry(node_id)
                        .or_insert_with(|| index_map_new());
                    for (partition_num, by_substate_key) in by_partition_num {
                        merged_by_partition_num
                            .entry(partition_num)
                            .or_insert_with(|| index_map_new())
                            .extend(by_substate_key);
                    }
                }
                result.system_structure.substate_system_structures = substate_system_structures;
            }
            _ => {}
        }
    }
}

/// Merges the given `source` into a `target`, asserting no overlap.
/// This function is not a method on [`StateUpdates`], since it is only used here locally (called
/// from a method describing itself as "a hack").
/// Note: the system receipt should not be conflicting with the flash receipt, and this function
/// will panic if this invariant is broken.
fn merge_asserting_no_overlap(target: &mut StateUpdates, source: StateUpdates) {
    for (node_id, source_node_state_updates) in source.by_node {
        let target_node_state_updates = target.by_node.entry(node_id).or_default();
        let target_by_partition = match target_node_state_updates {
            NodeStateUpdates::Delta { by_partition } => by_partition,
        };
        match source_node_state_updates {
            NodeStateUpdates::Delta { by_partition } => {
                for (partition_num, partition_state_updates) in by_partition {
                    let previous_target_partition_state_updates =
                        target_by_partition.insert(partition_num, partition_state_updates);
                    if !is_noop_partition_state_updates(&previous_target_partition_state_updates) {
                        panic!("Invalid genesis creation: Transactions overwriting initial flash substates");
                    }
                }
            }
        }
    }
}

/// Returns true if the given update is effectively no-op (i.e. [`None`] or empty delta).
/// This check is required since under some circumstances, Track may end up with empty partition
/// record (in fact, this check was migrated from previous version of the code, from before
/// [`StateUpdates`] structure refactoring).
fn is_noop_partition_state_updates(opt_updates: &Option<PartitionStateUpdates>) -> bool {
    let Some(updates) = opt_updates else {
        return true;
    };
    match updates {
        PartitionStateUpdates::Delta { by_substate } => by_substate.is_empty(),
        PartitionStateUpdates::Batch(BatchPartitionStateUpdate::Reset { .. }) => false,
    }
}

pub struct Bootstrapper<'s, S, E, W>
where
    S: SubstateDatabase + CommittableSubstateDatabase,
    E: NativeVmExtension,
    W: WasmEngine,
{
    network_definition: NetworkDefinition,
    substate_db: &'s mut S,
    vm_init: VmInit<'s, W, E>,
    trace: bool,
}

impl<'s, S, E, W> Bootstrapper<'s, S, E, W>
where
    S: SubstateDatabase + CommittableSubstateDatabase,
    E: NativeVmExtension,
    W: WasmEngine,
{
    pub fn new(
        network_definition: NetworkDefinition,
        substate_db: &'s mut S,
        vm_init: VmInit<'s, W, E>,
        trace: bool,
    ) -> Bootstrapper<'s, S, E, W> {
        Bootstrapper {
            network_definition,
            substate_db,
            vm_init,
            trace,
        }
    }

    pub fn bootstrap_test_default(&mut self) -> Option<GenesisReceipts> {
        self.bootstrap_with_genesis_data(
            vec![],
            Epoch::of(1),
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
                num_fee_increase_delay_epochs: 1,
                validator_creation_usd_cost: *DEFAULT_VALIDATOR_USD_COST,
            },
            1,
            Some(0),
            *DEFAULT_TESTING_FAUCET_SUPPLY,
        )
    }

    pub fn bootstrap_with_genesis_data(
        &mut self,
        genesis_data_chunks: Vec<GenesisDataChunk>,
        genesis_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
        initial_time_ms: i64,
        initial_current_leader: Option<ValidatorIndex>,
        faucet_supply: Decimal,
    ) -> Option<GenesisReceipts> {
        let flash_receipt = create_substate_flash_for_genesis();
        let first_package = flash_receipt.state_update_summary.new_packages[0];
        let first_typed_info = self
            .substate_db
            .get_mapped::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
                first_package.as_node_id(),
                TYPE_INFO_FIELD_PARTITION,
                &TypeInfoField::TypeInfo.into(),
            );

        if first_typed_info.is_none() {
            self.substate_db.commit(
                &flash_receipt
                    .state_updates
                    .create_database_updates::<SpreadPrefixKeyMapper>(),
            );

            let mut system_bootstrap_receipt = self.execute_system_bootstrap(
                genesis_epoch,
                initial_config,
                initial_time_ms,
                initial_current_leader,
                faucet_supply,
            );

            flash_receipt
                .merge_genesis_flash_into_transaction_receipt(&mut system_bootstrap_receipt);

            let mut data_ingestion_receipts = vec![];
            for (chunk_index, chunk) in genesis_data_chunks.into_iter().enumerate() {
                let receipt = self.ingest_genesis_data_chunk(chunk, chunk_index);
                data_ingestion_receipts.push(receipt);
            }

            let genesis_wrap_up_receipt = self.execute_genesis_wrap_up();

            Some(GenesisReceipts {
                system_bootstrap_receipt,
                data_ingestion_receipts,
                wrap_up_receipt: genesis_wrap_up_receipt,
            })
        } else {
            None
        }
    }

    fn execute_system_bootstrap(
        &mut self,
        genesis_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
        initial_time_ms: i64,
        initial_current_leader: Option<ValidatorIndex>,
        faucet_supply: Decimal,
    ) -> TransactionReceipt {
        let transaction = create_system_bootstrap_transaction(
            genesis_epoch,
            initial_config,
            initial_time_ms,
            initial_current_leader,
            faucet_supply,
        );

        let receipt = execute_transaction(
            self.substate_db,
            self.vm_init.clone(),
            &ExecutionConfig::for_genesis_transaction(self.network_definition.clone())
                .with_kernel_trace(self.trace),
            &transaction
                .prepare()
                .expect("Expected system bootstrap transaction to be preparable")
                .get_executable(btreeset![AuthAddresses::system_role()]),
        );

        let commit_result = receipt.expect_commit(true);

        self.substate_db.commit(
            &commit_result
                .state_updates
                .create_database_updates::<SpreadPrefixKeyMapper>(),
        );

        receipt
    }

    fn ingest_genesis_data_chunk(
        &mut self,
        chunk: GenesisDataChunk,
        chunk_number: usize,
    ) -> TransactionReceipt {
        let transaction =
            create_genesis_data_ingestion_transaction(&GENESIS_HELPER, chunk, chunk_number);
        let receipt = execute_transaction(
            self.substate_db,
            self.vm_init.clone(),
            &ExecutionConfig::for_genesis_transaction(self.network_definition.clone())
                .with_kernel_trace(self.trace),
            &transaction
                .prepare()
                .expect("Expected genesis data chunk transaction to be preparable")
                .get_executable(btreeset![AuthAddresses::system_role()]),
        );

        let commit_result = receipt.expect_commit(true);
        self.substate_db.commit(
            &commit_result
                .state_updates
                .create_database_updates::<SpreadPrefixKeyMapper>(),
        );

        receipt
    }

    fn execute_genesis_wrap_up(&mut self) -> TransactionReceipt {
        let transaction = create_genesis_wrap_up_transaction();

        let receipt = execute_transaction(
            self.substate_db,
            self.vm_init.clone(),
            &ExecutionConfig::for_genesis_transaction(self.network_definition.clone())
                .with_kernel_trace(self.trace),
            &transaction
                .prepare()
                .expect("Expected genesis wrap up transaction to be preparable")
                .get_executable(btreeset![AuthAddresses::system_role()]),
        );

        let commit_result = receipt.expect_commit(true);
        self.substate_db.commit(
            &commit_result
                .state_updates
                .create_database_updates::<SpreadPrefixKeyMapper>(),
        );

        receipt
    }
}

pub fn create_system_bootstrap_flash(
) -> BTreeMap<(NodeId, PartitionNumber), BTreeMap<SubstateKey, Vec<u8>>> {
    let package_flashes = [
        (
            PACKAGE_PACKAGE,
            PackageNativePackage::definition(),
            NativeCodeId::PackageCode1 as u64,
            metadata_init! {
                "name" => "Package Package".to_owned(), locked;
                "description" => "A native package that is called to create a new package on the network.".to_owned(), locked;
            },
            // Maps the application layer schema collection index to the system layer schema partition
            btreemap! {
                PACKAGE_BLUEPRINT.to_string() => vec![SystemInstruction::MapCollectionToPhysicalPartition {
                    collection_index: PackageCollection::SchemaKeyValue.collection_index(),
                    partition_num: SCHEMAS_PARTITION,
                }],
            },
        ),
        (
            TRANSACTION_PROCESSOR_PACKAGE,
            TransactionProcessorNativePackage::definition(),
            NativeCodeId::TransactionProcessorCode1 as u64,
            metadata_init! {
                "name" => "Transaction Processor Package".to_owned(), locked;
                "description" => "A native package that defines the logic of the processing of manifest instructions and transaction runtime.".to_owned(), locked;
            },
            btreemap!(),
        ),
        (
            METADATA_MODULE_PACKAGE,
            MetadataNativePackage::definition(),
            NativeCodeId::MetadataCode1 as u64,
            metadata_init! {
                "name" => "Metadata Package".to_owned(), locked;
                "description" => "A native package that defines the logic of the metadata module that is used by resources, components, and packages.".to_owned(), locked;
            },
            btreemap!(),
        ),
        (
            ROLE_ASSIGNMENT_MODULE_PACKAGE,
            RoleAssignmentNativePackage::definition(),
            NativeCodeId::RoleAssignmentCode1 as u64,
            metadata_init! {
                "name" => "Access Rules Package".to_owned(), locked;
                "description" => "A native package that defines the logic of the access rules module that is used by resources, components, and packages.".to_owned(), locked;
            },
            btreemap!(),
        ),
        (
            RESOURCE_PACKAGE,
            ResourceNativePackage::definition(),
            NativeCodeId::ResourceCode1 as u64,
            metadata_init! {
                "name" => "Resource Package".to_owned(), locked;
                "description" => "A native package that is called to create a new resource manager on the network.".to_owned(), locked;
            },
            btreemap!(),
        ),
        (
            ROYALTY_MODULE_PACKAGE,
            RoyaltyNativePackage::definition(),
            NativeCodeId::RoyaltyCode1 as u64,
            metadata_init! {
                "name" => "Royalty Package".to_owned(), locked;
                "description" => "A native package that defines the logic of the royalty module used by components.".to_owned(), locked;
            },
            btreemap!(),
        ),
        (
            TEST_UTILS_PACKAGE,
            TestUtilsNativePackage::definition(),
            NativeCodeId::TestUtilsCode1 as u64,
            metadata_init! {
                "name" => "Test Utils Package".to_owned(), locked;
                "description" => "A native package that contains a set of useful functions to use in testing.".to_owned(), locked;
            },
            btreemap!(),
        ),
    ];

    let mut to_flash = BTreeMap::new();

    for (address, definition, native_code_id, metadata_init, system_instructions) in package_flashes
    {
        let partitions = {
            let package_structure = PackageNativePackage::validate_and_build_package_structure(
                definition,
                VmType::Native,
                native_code_id.to_be_bytes().to_vec(),
                system_instructions,
                &VmBoot::babylon(),
            )
            .unwrap_or_else(|err| {
                panic!(
                    "Invalid flashed Package definition with native_code_id {}: {:?}",
                    native_code_id, err
                )
            });

            create_package_partition_substates(package_structure, metadata_init, None)
        };

        for (partition_num, partition_substates) in partitions {
            let mut substates = BTreeMap::new();
            for (key, value) in partition_substates {
                substates.insert(key, value.into());
            }
            to_flash.insert((address.into_node_id(), partition_num), substates);
        }
    }

    to_flash
}

pub fn create_substate_flash_for_genesis() -> FlashReceipt {
    let substate_flash = create_system_bootstrap_flash();
    let mut system_updates = index_map_new();
    let mut new_packages = index_set_new();
    let mut new_components = index_set_new();
    let mut new_resources = index_set_new();
    let mut new_vaults = index_set_new();

    for ((node_id, partition_num), substates) in substate_flash {
        let mut substate_updates = index_map_new();
        for (substate_key, value) in substates {
            substate_updates.insert(substate_key, DatabaseUpdate::Set(value));
        }

        system_updates.insert((node_id, partition_num), substate_updates);
        if node_id.is_global_package() {
            new_packages.insert(PackageAddress::new_or_panic(node_id.0));
        }
        if node_id.is_global_component() {
            new_components.insert(ComponentAddress::new_or_panic(node_id.0));
        }
        if node_id.is_global_resource_manager() {
            new_resources.insert(ResourceAddress::new_or_panic(node_id.0));
        }
        if node_id.is_internal_vault() {
            new_vaults.insert(InternalAddress::new_or_panic(node_id.0));
        }
    }

    let state_updates = StateUpdates::from(LegacyStateUpdates {
        partition_deletions: index_set_new(),
        system_updates,
    });
    let flashed_db = FlashedSubstateDatabase {
        flash_updates: state_updates.create_database_updates::<SpreadPrefixKeyMapper>(),
    };
    let mut substate_schema_mapper =
        SubstateSchemaMapper::new(SystemDatabaseReader::new(&flashed_db));
    substate_schema_mapper.add_for_all_individually_updated(&state_updates);
    let substate_system_structures = substate_schema_mapper.done();

    FlashReceipt {
        state_updates,
        state_update_summary: StateUpdateSummary {
            new_packages,
            new_components,
            new_resources,
            new_vaults,
            vault_balance_changes: index_map_new(),
        },
        substate_system_structures,
    }
}

/// A [`SubstateDatabase`] implementation holding only the initial [`DatabaseUpdates`] from a system
/// bootstrap flash.
struct FlashedSubstateDatabase {
    flash_updates: DatabaseUpdates,
}

impl SubstateDatabase for FlashedSubstateDatabase {
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        let DbPartitionKey {
            node_key,
            partition_num,
        } = partition_key;
        self.flash_updates
            .node_updates
            .get(node_key)
            .and_then(|node_updates| node_updates.partition_updates.get(partition_num))
            .and_then(|partition_updates| match partition_updates {
                PartitionDatabaseUpdates::Delta { substate_updates } => substate_updates
                    .get(sort_key)
                    .and_then(|update| match update {
                        DatabaseUpdate::Set(value) => Some(value.clone()),
                        DatabaseUpdate::Delete => None,
                    }),
                PartitionDatabaseUpdates::Reset {
                    new_substate_values,
                } => new_substate_values.get(sort_key).cloned(),
            })
    }

    fn list_entries_from(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        let DbPartitionKey {
            node_key,
            partition_num,
        } = partition_key;
        let mut entries = self
            .flash_updates
            .node_updates
            .get(node_key)
            .and_then(|node_updates| node_updates.partition_updates.get(partition_num))
            .into_iter()
            .flat_map(|partition_updates| {
                let effective_entries = match partition_updates {
                    PartitionDatabaseUpdates::Delta { substate_updates } => {
                        Box::new(substate_updates.iter().filter_map(|(sort_key, update)| {
                            match update {
                                DatabaseUpdate::Set(value) => Some((sort_key, value)),
                                DatabaseUpdate::Delete => None,
                            }
                        })) as Box<dyn Iterator<Item = _>>
                    }
                    PartitionDatabaseUpdates::Reset {
                        new_substate_values,
                    } => Box::new(new_substate_values.iter()),
                };
                effective_entries.map(|(sort_key, value)| (sort_key.clone(), value.clone()))
            })
            // It is more performant to filter before sorting (than sort + skip-while)
            .filter(|(sort_key, _value)| Some(sort_key) >= from_sort_key)
            .collect::<Vec<_>>();
        // The method's contract requires the results to be in the key's order (ascending):
        entries.sort_by(|(left_key, _), (right_key, _)| left_key.cmp(right_key));
        Box::new(entries.into_iter())
    }
}

pub fn create_system_bootstrap_transaction(
    initial_epoch: Epoch,
    initial_config: ConsensusManagerConfig,
    initial_time_ms: i64,
    initial_current_leader: Option<ValidatorIndex>,
    faucet_supply: Decimal,
) -> SystemTransactionV1 {
    let mut id_allocator = ManifestIdAllocator::new();
    let mut instructions = Vec::new();
    let mut pre_allocated_addresses = vec![];
    let mut blobs = vec![];

    // XRD Token
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(XRD),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                .to_string(),
            args: to_manifest_value_and_unwrap!(
                &FungibleResourceManagerCreateWithInitialSupplyManifestInput {
                    owner_role: OwnerRole::Fixed(rule!(require(AuthAddresses::system_role()))),
                    track_total_supply: false,
                    divisibility: 18,
                    resource_roles: FungibleResourceRoles {
                        mint_roles: mint_roles! {
                            minter => rule!(require(global_caller(CONSENSUS_MANAGER)));
                            minter_updater => rule!(deny_all);
                        },
                        burn_roles: burn_roles! {
                            burner => rule!(require(global_caller(CONSENSUS_MANAGER)));
                            burner_updater => rule!(deny_all);
                        },
                        ..Default::default()
                    },
                    metadata: metadata! {
                        init {
                            "symbol" => "XRD".to_owned(), locked;
                            "name" => "Radix".to_owned(), locked;
                            "description" => "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.".to_owned(), locked;
                            "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-xrd-32x32.png".to_owned()), locked;
                            "info_url" => UncheckedUrl::of("https://tokens.radixdlt.com".to_owned()), locked;
                            "tags" => Vec::<String>::new(), locked;
                        }
                    },
                    initial_supply: Decimal::zero(),
                    address_reservation: Some(id_allocator.new_address_reservation_id()),
                }
            ),
        });
    }

    // Package Token
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateManifestInput {
                    owner_role: OwnerRole::Fixed(rule!(require(AuthAddresses::system_role()))),
                    id_type: NonFungibleIdType::Bytes,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_local_without_self_package_replacement::<()>(),
                    resource_roles: NonFungibleResourceRoles {
                        withdraw_roles: withdraw_roles! {
                            withdrawer => rule!(deny_all);
                            withdrawer_updater => rule!(deny_all);
                        },
                        ..Default::default()
                    },
                    metadata: metadata! {
                        init {
                            "name" => "Package Virtual Badges".to_owned(), locked;
                            "description" => "Virtual badges generated automatically by the Radix system to represent the authority of the package for a direct caller. These badges cease to exist at the end of their transaction.".to_owned(), locked;
                            "tags" => vec!["badge".to_owned()], locked;
                            "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-package_of_direct_caller_virtual_badge.png".to_owned()), locked;
                        }
                    },
                    address_reservation: Some(id_allocator.new_address_reservation_id()),
                }
            ),
        });
    }

    // Object Token
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(GLOBAL_CALLER_VIRTUAL_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateManifestInput {
                    owner_role: OwnerRole::Fixed(rule!(require(AuthAddresses::system_role()))),
                    id_type: NonFungibleIdType::Bytes,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_local_without_self_package_replacement::<()>(),
                    resource_roles: NonFungibleResourceRoles {
                        withdraw_roles: withdraw_roles! {
                            withdrawer => rule!(deny_all);
                            withdrawer_updater => rule!(deny_all);
                        },
                        ..Default::default()
                    },
                    metadata: metadata! {
                        init {
                            "name" => "Global Caller Virtual Badges".to_owned(), locked;
                            "description" => "Virtual badges generated automatically by the Radix system to represent the authority of a global caller. These badges cease to exist at the end of their transaction.".to_owned(), locked;
                            "tags" => vec!["badge".to_owned()], locked;
                            "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-global_caller_virtual_badge.png".to_owned()), locked;
                        }
                    },
                    address_reservation: Some(id_allocator.new_address_reservation_id()),
                }
            ),
        });
    }

    // Package Owner Token
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(PACKAGE_OWNER_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateManifestInput {
                    owner_role: OwnerRole::Fixed(rule!(require(global_caller(PACKAGE_PACKAGE)))),
                    id_type: NonFungibleIdType::Bytes,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_local_without_self_package_replacement::<PackageOwnerBadgeData>(),
                    resource_roles: NonFungibleResourceRoles {
                        mint_roles: mint_roles! {
                            minter => rule!(require(package_of_direct_caller(PACKAGE_PACKAGE)));
                            minter_updater => rule!(deny_all);
                        },
                        ..Default::default()
                    },
                    metadata: metadata! {
                        init {
                            "name" => "Package Owner Badges".to_owned(), locked;
                            "description" => "Badges created by the Radix system that provide individual control over blueprint packages deployed by developers.".to_owned(), locked;
                            "tags" => vec!["badge".to_owned(), "package".to_owned()], locked;
                            "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-package_owner_badge.png".to_owned()), locked;
                        }
                    },
                    address_reservation: Some(id_allocator.new_address_reservation_id()),
                }
            ),
        });
    }

    // Identity Package
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(IDENTITY_OWNER_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateManifestInput {
                    owner_role: OwnerRole::Fixed(rule!(require(global_caller(IDENTITY_PACKAGE)))),
                    id_type: NonFungibleIdType::Bytes,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_local_without_self_package_replacement::<IdentityOwnerBadgeData>(),
                    resource_roles: NonFungibleResourceRoles {
                        mint_roles: mint_roles! {
                            minter => rule!(require(package_of_direct_caller(IDENTITY_PACKAGE)));
                            minter_updater => rule!(deny_all);
                        },
                        ..Default::default()
                    },
                    metadata: metadata! {
                        init {
                            "name" => "Identity Owner Badges".to_owned(), locked;
                            "description" => "Badges created by the Radix system that provide individual control over identity components.".to_owned(), locked;
                            "tags" => vec!["badge".to_owned(), "identity".to_owned()], locked;
                            "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-identity_owner_badge.png".to_owned()), locked;
                        }
                    },
                    address_reservation: Some(id_allocator.new_address_reservation_id()),
                }
            ),
        });

        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(IDENTITY_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                definition: IdentityNativePackage::definition(),
                native_package_code_id: NativeCodeId::IdentityCode1 as u64,
                metadata: metadata_init! {
                    "name" => "Identity Package".to_owned(), locked;
                    "description" => "A native package that defines the logic of identity components.".to_owned(), locked;
                },
            }),
        });
    }

    // ConsensusManager Package
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(CONSENSUS_MANAGER_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                definition: ConsensusManagerNativePackage::definition(),
                native_package_code_id: NativeCodeId::ConsensusManagerCode1 as u64,
                metadata: metadata_init! {
                    "name" => "Consensus Manager Package".to_owned(), locked;
                    "description" => "A native package that may be used to get network consensus information.".to_owned(), locked;
                },
            }),
        });
    }

    // Account Package
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(ACCOUNT_OWNER_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateManifestInput {
                    owner_role: OwnerRole::Fixed(rule!(require(global_caller(ACCOUNT_PACKAGE)))),
                    id_type: NonFungibleIdType::Bytes,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_local_without_self_package_replacement::<AccountOwnerBadgeData>(),
                    resource_roles: NonFungibleResourceRoles {
                        mint_roles: mint_roles! {
                            minter => rule!(require(package_of_direct_caller(ACCOUNT_PACKAGE)));
                            minter_updater => rule!(deny_all);
                        },
                        ..Default::default()
                    },
                    metadata: metadata! {
                        init {
                            "name" => "Account Owner Badges".to_owned(), locked;
                            "description" => "Badges created by the Radix system that provide individual control over account components.".to_owned(), locked;
                            "tags" => vec![
                                "badge".to_owned(),
                                "account".to_owned(),
                            ], locked;
                            "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-account_owner_badge.png".to_owned()), locked;
                        }
                    },
                    address_reservation: Some(id_allocator.new_address_reservation_id()),
                }
            ),
        });

        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(ACCOUNT_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                definition: AccountNativePackage::definition(),
                native_package_code_id: NativeCodeId::AccountCode1 as u64,
                metadata: metadata_init! {
                    "name" => "Account Package".to_owned(), locked;
                    "description" => "A native package that defines the logic of account components.".to_owned(), locked;
                },
            }),
        });
    }

    // AccessController Package
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(ACCESS_CONTROLLER_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                definition: AccessControllerNativePackage::definition(),
                metadata: metadata_init! {
                    "name" => "Access Controller Package".to_owned(), locked;
                    "description" => "A native package that defines the logic of access controller components.".to_owned(), locked;
                },
                native_package_code_id: NativeCodeId::AccessControllerCode1 as u64,
            }),
        });
    }

    // Pool Package
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(POOL_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                definition: PoolNativePackage::definition(PoolV1MinorVersion::Zero),
                metadata: metadata_init! {
                    "name" => "Pool Package".to_owned(), locked;
                    "description" => "A native package that defines the logic for a selection of pool components.".to_owned(), locked;
                },
                native_package_code_id: NativeCodeId::PoolCode1 as u64,
            }),
        });
    }

    // ECDSA Secp256k1
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(SECP256K1_SIGNATURE_VIRTUAL_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateManifestInput {
                    owner_role: OwnerRole::Fixed(rule!(require(AuthAddresses::system_role()))),
                    id_type: NonFungibleIdType::Bytes,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_local_without_self_package_replacement::<()>(),
                    resource_roles: NonFungibleResourceRoles::default(),
                    metadata: metadata! {
                        init {
                            "name" => "ECDSA secp256k1 Virtual Badges".to_owned(), locked;
                            "description" => "Virtual badges generated automatically by the Radix system to represent ECDSA secp256k1 signatures applied to transactions. These badges cease to exist at the end of their transaction.".to_owned(), locked;
                            "tags" => vec!["badge".to_owned()], locked;
                            "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-ecdsa_secp256k1_signature_virtual_badge.png".to_owned()), locked;
                        }
                    },
                    address_reservation: Some(id_allocator.new_address_reservation_id()),
                }
            ),
        });
    }

    // Ed25519
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(ED25519_SIGNATURE_VIRTUAL_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateManifestInput {
                    owner_role: OwnerRole::Fixed(rule!(require(AuthAddresses::system_role()))),
                    id_type: NonFungibleIdType::Bytes,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_local_without_self_package_replacement::<()>(),
                    resource_roles: NonFungibleResourceRoles::default(),
                    metadata: metadata! {
                        init {
                            "name" => "EdDSA Ed25519 Virtual Badges".to_owned(), locked;
                            "description" => "Virtual badges generated automatically by the Radix system to represent EdDSA Ed25519 signatures applied to transactions. These badges cease to exist at the end of their transaction.".to_owned(), locked;
                            "tags" => vec!["badge".to_owned()], locked;
                            "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-eddsa_ed25519_signature_virtual_badge.png".to_owned()), locked;
                        }
                    },
                    address_reservation: Some(id_allocator.new_address_reservation_id()),
                }
            ),
        });
    }

    // System Token
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(SYSTEM_TRANSACTION_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateManifestInput {
                    owner_role: OwnerRole::Fixed(rule!(require(AuthAddresses::system_role()))),
                    id_type: NonFungibleIdType::Integer,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_local_without_self_package_replacement::<()>(),
                    resource_roles: NonFungibleResourceRoles::default(),
                    metadata: metadata! {
                        init {
                            "name" => "System Transaction Badge".to_owned(), locked;
                            "description" => "Virtual badges are created under this resource to represent the Radix system's authority at genesis and to affect changes to system entities during protocol updates, or to represent the Radix system's authority in the regularly occurring system transactions including round and epoch changes.".to_owned(), locked;
                            "tags" => vec!["badge".to_owned(), "system badge".to_owned()], locked;
                            "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-system_transaction_badge.png".to_owned()), locked;
                        }
                    },
                    address_reservation: Some(id_allocator.new_address_reservation_id()),
                }
            ),
        });
    }

    // Faucet Package
    {
        let faucet_code = include_bytes!("../../assets/faucet.wasm").to_vec();
        let faucet_abi = include_bytes!("../../assets/faucet.rpd").to_vec();
        let faucet_code_hash = hash(&faucet_code);
        blobs.push(BlobV1(faucet_code));
        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(FAUCET_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishWasmAdvancedManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                code: ManifestBlobRef(faucet_code_hash.0),
                definition: manifest_decode(&faucet_abi).unwrap(),
                metadata: metadata_init!{
                    "name" => "Faucet Package".to_owned(), locked;
                    "description" => "A package that defines the logic of a simple faucet component for testing purposes.".to_owned(), locked;
                },
                owner_role: OwnerRole::None,
            }),
        });
    }

    // Genesis helper package
    {
        let genesis_helper_code = include_bytes!("../../assets/genesis_helper.wasm").to_vec();
        let genesis_helper_abi = include_bytes!("../../assets/genesis_helper.rpd").to_vec();
        let genesis_helper_code_hash = hash(&genesis_helper_code);
        blobs.push(BlobV1(genesis_helper_code));
        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(GENESIS_HELPER_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishWasmAdvancedManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                code: ManifestBlobRef(genesis_helper_code_hash.0),
                definition: manifest_decode(&genesis_helper_abi).unwrap(),
                metadata: metadata_init! {
                    "name" => "Genesis Helper Package".to_owned(), locked;
                    "description" => "A package that defines the logic of the genesis helper which includes various utility and helper functions used in the creation of the Babylon Genesis.".to_owned(), locked;
                },
                owner_role: OwnerRole::None,
            }),
        });
    }

    // Create ConsensusManager
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(VALIDATOR_OWNER_BADGE),
        ));
        pre_allocated_addresses.push((
            BlueprintId::new(&CONSENSUS_MANAGER_PACKAGE, CONSENSUS_MANAGER_BLUEPRINT),
            GlobalAddress::from(CONSENSUS_MANAGER),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: CONSENSUS_MANAGER_PACKAGE.into(),
            blueprint_name: CONSENSUS_MANAGER_BLUEPRINT.to_string(),
            function_name: CONSENSUS_MANAGER_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ConsensusManagerCreateManifestInput {
                validator_owner_token_address: id_allocator.new_address_reservation_id(),
                component_address: id_allocator.new_address_reservation_id(),
                initial_epoch,
                initial_config,
                initial_time_ms,
                initial_current_leader,
            }),
        });
    }

    // Create GenesisHelper
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&GENESIS_HELPER_PACKAGE, GENESIS_HELPER_BLUEPRINT),
            GlobalAddress::from(GENESIS_HELPER),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: GENESIS_HELPER_PACKAGE.into(),
            blueprint_name: GENESIS_HELPER_BLUEPRINT.to_string(),
            function_name: "new".to_string(),
            args: manifest_args!(
                id_allocator.new_address_reservation_id(),
                CONSENSUS_MANAGER,
                AuthAddresses::system_role()
            )
            .into(),
        });
    }

    // Intent Hash Store package
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(TRANSACTION_TRACKER_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                native_package_code_id: NativeCodeId::TransactionTrackerCode1 as u64,
                definition: TransactionTrackerNativePackage::definition(),
                metadata: metadata_init!(),
            }),
        });
    }

    // Intent Hash Store component
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&TRANSACTION_TRACKER_PACKAGE, TRANSACTION_TRACKER_BLUEPRINT),
            GlobalAddress::from(TRANSACTION_TRACKER),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: TRANSACTION_TRACKER_PACKAGE.into(),
            blueprint_name: TRANSACTION_TRACKER_BLUEPRINT.to_string(),
            function_name: TRANSACTION_TRACKER_CREATE_IDENT.to_string(),
            args: manifest_args!(id_allocator.new_address_reservation_id()).into(),
        });
    }

    // Faucet
    // Note - the faucet is now created as part of bootstrap instead of wrap-up, to enable
    // transaction scenarios to be injected into the ledger in the node before genesis wrap-up occurs
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&FAUCET_PACKAGE, FAUCET_BLUEPRINT),
            GlobalAddress::from(FAUCET),
        ));

        // Mint XRD for the faucet, and then deposit it into the new faucet
        // Note - on production environments, the faucet will be empty
        let faucet_xrd_bucket = id_allocator.new_bucket_id();
        instructions.push(
            InstructionV1::CallMethod {
                address: XRD.clone().into(),
                method_name: FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
                args: manifest_args!(faucet_supply).into(),
            }
            .into(),
        );
        instructions.push(
            InstructionV1::TakeFromWorktop {
                resource_address: XRD,
                amount: faucet_supply,
            }
            .into(),
        );
        instructions.push(InstructionV1::CallFunction {
            package_address: FAUCET_PACKAGE.into(),
            blueprint_name: FAUCET_BLUEPRINT.to_string(),
            function_name: "new".to_string(),
            args: manifest_args!(id_allocator.new_address_reservation_id(), faucet_xrd_bucket)
                .into(),
        });
    }

    SystemTransactionV1 {
        instructions: InstructionsV1(instructions),
        pre_allocated_addresses: pre_allocated_addresses
            .into_iter()
            .map(|allocation_pair| allocation_pair.into())
            .collect(),
        blobs: BlobsV1 { blobs },
        hash_for_execution: hash(format!("Genesis Bootstrap")),
    }
}

pub fn create_genesis_data_ingestion_transaction(
    genesis_helper: &ComponentAddress,
    chunk: GenesisDataChunk,
    chunk_number: usize,
) -> SystemTransactionV1 {
    let mut instructions = Vec::new();

    let (chunk, pre_allocated_addresses) = map_address_allocations_for_manifest(chunk);

    instructions.push(InstructionV1::CallMethod {
        address: genesis_helper.clone().into(),
        method_name: "ingest_data_chunk".to_string(),
        args: manifest_args!(chunk).into(),
    });

    SystemTransactionV1 {
        instructions: InstructionsV1(instructions),
        pre_allocated_addresses,
        blobs: BlobsV1 { blobs: vec![] },
        hash_for_execution: hash(format!("Genesis Data Chunk: {}", chunk_number)),
    }
}

fn map_address_allocations_for_manifest(
    genesis_data_chunk: GenesisDataChunk,
) -> (ManifestGenesisDataChunk, Vec<PreAllocatedAddress>) {
    match genesis_data_chunk {
        GenesisDataChunk::Validators(content) => {
            (ManifestGenesisDataChunk::Validators(content), vec![])
        }
        GenesisDataChunk::Stakes {
            accounts,
            allocations,
        } => (
            ManifestGenesisDataChunk::Stakes {
                accounts,
                allocations,
            },
            vec![],
        ),
        GenesisDataChunk::Resources(resources) => {
            let (resources, allocations): (Vec<_>, Vec<_>) = resources
                .into_iter()
                .enumerate()
                .map(|(index, resource)| {
                    let manifest_resource = ManifestGenesisResource {
                        resource_address_reservation: ManifestAddressReservation(index as u32),
                        metadata: resource.metadata,
                        owner: resource.owner,
                    };
                    let address_allocation = PreAllocatedAddress {
                        blueprint_id: BlueprintId {
                            package_address: RESOURCE_PACKAGE,
                            blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                        },
                        address: resource.reserved_resource_address.into(),
                    };
                    (manifest_resource, address_allocation)
                })
                .unzip();
            (ManifestGenesisDataChunk::Resources(resources), allocations)
        }
        GenesisDataChunk::ResourceBalances {
            accounts,
            allocations,
        } => (
            ManifestGenesisDataChunk::ResourceBalances {
                accounts,
                allocations,
            },
            vec![],
        ),
        GenesisDataChunk::XrdBalances(content) => {
            (ManifestGenesisDataChunk::XrdBalances(content), vec![])
        }
    }
}

pub fn create_genesis_wrap_up_transaction() -> SystemTransactionV1 {
    let mut instructions = Vec::new();

    instructions.push(InstructionV1::CallMethod {
        address: GENESIS_HELPER.clone().into(),
        method_name: "wrap_up".to_string(),
        args: manifest_args!().into(),
    });

    SystemTransactionV1 {
        instructions: InstructionsV1(instructions),
        pre_allocated_addresses: vec![],
        blobs: BlobsV1 { blobs: vec![] },
        hash_for_execution: hash(format!("Genesis Wrap Up")),
    }
}
