use crate::blueprints::access_controller::v1::*;
use crate::blueprints::account::{AccountNativePackage, AccountOwnerBadgeData};
use crate::blueprints::consensus_manager::ConsensusManagerNativePackage;
use crate::blueprints::identity::{IdentityNativePackage, IdentityOwnerBadgeData};
use crate::blueprints::package::*;
use crate::blueprints::pool::v1::package::{PoolNativePackage, PoolV1MinorVersion};
use crate::blueprints::resource::ResourceNativePackage;
use crate::blueprints::test_utils::TestUtilsNativePackage;
use crate::blueprints::transaction_processor::TransactionProcessorNativePackage;
use crate::blueprints::transaction_tracker::*;
use crate::internal_prelude::*;
use crate::object_modules::metadata::MetadataNativePackage;
use crate::object_modules::role_assignment::RoleAssignmentNativePackage;
use crate::object_modules::royalty::RoyaltyNativePackage;
use crate::system::system_db_reader::SystemDatabaseReader;
use crate::transaction::*;
use crate::updates::*;
use crate::vm::VmBoot;
use lazy_static::lazy_static;
use radix_common::crypto::Secp256k1PublicKey;
use radix_common::math::traits::*;
use radix_common::types::ComponentAddress;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_tracker::*;
use radix_engine_interface::object_modules::metadata::{MetadataValue, UncheckedUrl};
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::*;
use radix_substate_store_interface::interface::*;
use radix_transactions::model::*;
use radix_transactions::prelude::*;

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
        let default_owner_address = ComponentAddress::preallocated_account_from_public_key(&key);
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

#[derive(Debug, Clone)]
pub struct GenesisReceipts {
    pub system_flash_receipt: TransactionReceipt,
    pub system_bootstrap_receipt: TransactionReceipt,
    pub data_ingestion_receipts: Vec<TransactionReceipt>,
    pub wrap_up_receipt: TransactionReceipt,
}

#[derive(Default)]
pub struct GenesisReceiptExtractionHooks {
    bootstrap_receipts: Vec<TransactionReceipt>,
    data_ingestion_receipts: Vec<TransactionReceipt>,
    wrap_up_receipts: Vec<TransactionReceipt>,
}

impl GenesisReceiptExtractionHooks {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn into_genesis_receipts(self) -> GenesisReceipts {
        let [system_flash_receipt, system_bootstrap_receipt] = self
            .bootstrap_receipts
            .try_into()
            .expect("Expected two bootstrap receipts (flash and transaction)");
        let [wrap_up_receipt] = self
            .wrap_up_receipts
            .try_into()
            .expect("Expected one wrap-up receipt");
        GenesisReceipts {
            system_flash_receipt,
            system_bootstrap_receipt,
            data_ingestion_receipts: self.data_ingestion_receipts,
            wrap_up_receipt,
        }
    }
}

impl ProtocolUpdateExecutionHooks for GenesisReceiptExtractionHooks {
    fn on_transaction_executed(&mut self, event: OnProtocolTransactionExecuted) {
        let OnProtocolTransactionExecuted {
            protocol_version,
            batch_group_index,
            receipt,
            ..
        } = event;
        if protocol_version == ProtocolVersion::GENESIS {
            match batch_group_index {
                0 => self.bootstrap_receipts.push(receipt.clone()),
                1 => self.data_ingestion_receipts.push(receipt.clone()),
                2 => self.wrap_up_receipts.push(receipt.clone()),
                _ => panic!("Unexpected bootstrap batch group index: {batch_group_index}"),
            }
        }
    }
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
        let mut commit_result =
            CommitResult::empty_with_outcome(TransactionOutcome::Success(vec![]));
        commit_result.state_updates = value.state_updates;
        commit_result.state_update_summary = value.state_update_summary;
        commit_result.system_structure.substate_system_structures =
            value.substate_system_structures;
        TransactionReceipt::empty_with_commit(commit_result)
    }
}

impl FlashReceipt {
    pub fn from_state_updates(
        state_updates: StateUpdates,
        before_store: &impl SubstateDatabase,
    ) -> Self {
        let state_updates = state_updates.rebuild_without_empty_entries();
        let state_update_summary =
            StateUpdateSummary::new_from_state_updates_on_db(before_store, &state_updates);
        let substate_system_structures = {
            let after_store = SystemDatabaseReader::new_with_overlay(before_store, &state_updates);
            let mut substate_schema_mapper = SubstateSchemaMapper::new(after_store);
            substate_schema_mapper.add_for_all_individually_updated(&state_updates);
            substate_schema_mapper.done()
        };
        Self {
            state_updates,
            state_update_summary,
            substate_system_structures,
        }
    }
}

pub fn create_system_bootstrap_flash_state_updates() -> StateUpdates {
    // The slightly weird order is so that it matches the historic order when this
    // used to be ordered by a BTreeMap over the node ids.
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

    let mut to_flash = StateUpdates::empty();

    for (address, definition, native_code_id, metadata_init, system_instructions) in package_flashes
    {
        let partitions = {
            let package_structure = PackageNativePackage::validate_and_build_package_structure(
                definition,
                VmType::Native,
                native_code_id.to_be_bytes().to_vec(),
                system_instructions,
                false,
                &VmBoot::babylon_genesis(),
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
            let partition_updates: IndexMap<_, _> = partition_substates
                .into_iter()
                .map(|(key, value)| (key, DatabaseUpdate::Set(value.into())))
                .collect();

            // To avoid creating wasted structure in StateUpdates, only create this partition if a change exists.
            if partition_updates.len() > 0 {
                to_flash
                    .of_node(address)
                    .of_partition(partition_num)
                    .mut_update_substates(partition_updates);
            }
        }
    }

    to_flash
}

pub fn create_substate_flash_for_genesis() -> FlashReceipt {
    let state_updates = create_system_bootstrap_flash_state_updates();
    FlashReceipt::from_state_updates(state_updates, &EmptySubstateDatabase)
}

struct EmptySubstateDatabase;

impl SubstateDatabase for EmptySubstateDatabase {
    fn get_raw_substate_by_db_key(
        &self,
        _partition_key: &DbPartitionKey,
        _sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        None
    }

    fn list_raw_values_from_db_key(
        &self,
        _partition_key: &DbPartitionKey,
        _from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        Box::new(core::iter::empty())
    }
}

pub fn create_system_bootstrap_transaction(
    initial_epoch: Epoch,
    initial_config: ConsensusManagerConfig,
    initial_time_ms: i64,
    initial_current_leader: Option<ValidatorIndex>,
    faucet_supply: Decimal,
) -> SystemTransactionV1 {
    let mut manifest_builder = ManifestBuilder::new_system_v1();
    let lookup = manifest_builder.name_lookup();

    // XRD Token
    {
        let xrd_reservation = manifest_builder.use_preallocated_address(
            XRD,
            RESOURCE_PACKAGE,
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            RESOURCE_PACKAGE,
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
            FungibleResourceManagerCreateWithInitialSupplyManifestInput {
                owner_role: OwnerRole::Fixed(rule!(require(system_execution(SystemExecution::Protocol)))),
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
                address_reservation: Some(xrd_reservation),
            },
        );
    }

    // Package of Direct Caller
    {
        let reservation = manifest_builder.use_preallocated_address(
            PACKAGE_OF_DIRECT_CALLER_RESOURCE,
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            NonFungibleResourceManagerCreateManifestInput {
                owner_role: OwnerRole::Fixed(rule!(require(system_execution(SystemExecution::Protocol)))),
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
                address_reservation: Some(reservation),
            },
        );
    }

    // Global Caller Resource
    {
        let reservation = manifest_builder.use_preallocated_address(
            GLOBAL_CALLER_RESOURCE,
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            NonFungibleResourceManagerCreateManifestInput {
                owner_role: OwnerRole::Fixed(rule!(require(system_execution(SystemExecution::Protocol)))),
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
                address_reservation: Some(reservation),
            },
        );
    }

    // Package Owner Resource
    {
        let reservation = manifest_builder.use_preallocated_address(
            PACKAGE_OWNER_BADGE,
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            NonFungibleResourceManagerCreateManifestInput {
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
                address_reservation: Some(reservation),
            },
        );
    }

    // Identity
    {
        let badge_reservation = manifest_builder.use_preallocated_address(
            IDENTITY_OWNER_BADGE,
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            NonFungibleResourceManagerCreateManifestInput {
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
                address_reservation: Some(badge_reservation),
            },
        );

        let package_reservation = manifest_builder.use_preallocated_address(
            IDENTITY_PACKAGE,
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
            PACKAGE_PUBLISH_NATIVE_IDENT,
            PackagePublishNativeManifestInput {
                package_address: Some(package_reservation),
                definition: IdentityNativePackage::definition(),
                native_package_code_id: NativeCodeId::IdentityCode1 as u64,
                metadata: metadata_init! {
                    "name" => "Identity Package".to_owned(), locked;
                    "description" => "A native package that defines the logic of identity components.".to_owned(), locked;
                },
            },
        );
    }

    // ConsensusManager Package
    {
        let reservation = manifest_builder.use_preallocated_address(
            CONSENSUS_MANAGER_PACKAGE,
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
            PACKAGE_PUBLISH_NATIVE_IDENT,
            PackagePublishNativeManifestInput {
                package_address: Some(reservation),
                definition: ConsensusManagerNativePackage::definition(),
                native_package_code_id: NativeCodeId::ConsensusManagerCode1 as u64,
                metadata: metadata_init! {
                    "name" => "Consensus Manager Package".to_owned(), locked;
                    "description" => "A native package that may be used to get network consensus information.".to_owned(), locked;
                },
            },
        );
    }

    // Account Package
    {
        let badge_reservation = manifest_builder.use_preallocated_address(
            ACCOUNT_OWNER_BADGE,
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            NonFungibleResourceManagerCreateManifestInput {
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
                address_reservation: Some(badge_reservation),
            },
        );

        let package_reservation = manifest_builder.use_preallocated_address(
            ACCOUNT_PACKAGE,
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
            PACKAGE_PUBLISH_NATIVE_IDENT,
            PackagePublishNativeManifestInput {
                package_address: Some(package_reservation),
                definition: AccountNativePackage::definition(),
                native_package_code_id: NativeCodeId::AccountCode1 as u64,
                metadata: metadata_init! {
                    "name" => "Account Package".to_owned(), locked;
                    "description" => "A native package that defines the logic of account components.".to_owned(), locked;
                },
            },
        );
    }

    // AccessController Package
    {
        let reservation = manifest_builder.use_preallocated_address(
            ACCESS_CONTROLLER_PACKAGE,
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
            PACKAGE_PUBLISH_NATIVE_IDENT,
            PackagePublishNativeManifestInput {
                package_address: Some(reservation),
                definition: AccessControllerV1NativePackage::definition(),
                metadata: metadata_init! {
                    "name" => "Access Controller Package".to_owned(), locked;
                    "description" => "A native package that defines the logic of access controller components.".to_owned(), locked;
                },
                native_package_code_id: NativeCodeId::AccessControllerCode1 as u64,
            },
        );
    }

    // Pool Package
    {
        let reservation = manifest_builder.use_preallocated_address(
            POOL_PACKAGE,
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
            PACKAGE_PUBLISH_NATIVE_IDENT,
            PackagePublishNativeManifestInput {
                package_address: Some(reservation),
                definition: PoolNativePackage::definition(PoolV1MinorVersion::Zero),
                metadata: metadata_init! {
                    "name" => "Pool Package".to_owned(), locked;
                    "description" => "A native package that defines the logic for a selection of pool components.".to_owned(), locked;
                },
                native_package_code_id: NativeCodeId::PoolCode1 as u64,
            },
        );
    }

    // ECDSA Secp256k1
    {
        let reservation = manifest_builder.use_preallocated_address(
            SECP256K1_SIGNATURE_RESOURCE,
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            NonFungibleResourceManagerCreateManifestInput {
                owner_role: OwnerRole::Fixed(rule!(require(system_execution(SystemExecution::Protocol)))),
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
                address_reservation: Some(reservation),
            }
        );
    }

    // Ed25519
    {
        let reservation = manifest_builder.use_preallocated_address(
            ED25519_SIGNATURE_RESOURCE,
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            NonFungibleResourceManagerCreateManifestInput {
                owner_role: OwnerRole::Fixed(rule!(require(system_execution(SystemExecution::Protocol)))),
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
                address_reservation: Some(reservation),
            },
        );
    }

    // System Execution Resource
    {
        let reservation = manifest_builder.use_preallocated_address(
            SYSTEM_EXECUTION_RESOURCE,
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            NonFungibleResourceManagerCreateManifestInput {
                owner_role: OwnerRole::Fixed(rule!(require(system_execution(SystemExecution::Protocol)))),
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
                address_reservation: Some(reservation),
            },
        );
    }

    // Faucet Package
    {
        let reservation: ManifestAddressReservation = manifest_builder.use_preallocated_address(
            FAUCET_PACKAGE,
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
        );
        manifest_builder = manifest_builder.publish_package_advanced(
            reservation,
            include_bytes!("../../assets/faucet.wasm").to_vec(),
            manifest_decode(include_bytes!("../../assets/faucet.rpd")).unwrap(),
            metadata_init!{
                "name" => "Faucet Package".to_owned(), locked;
                "description" => "A package that defines the logic of a simple faucet component for testing purposes.".to_owned(), locked;
            },
            OwnerRole::None,
        );
    }

    // Genesis helper package
    {
        let reservation = manifest_builder.use_preallocated_address(
            GENESIS_HELPER_PACKAGE,
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
        );
        manifest_builder = manifest_builder.publish_package_advanced(
            reservation,
            include_bytes!("../../assets/genesis_helper.wasm").to_vec(),
            manifest_decode(include_bytes!("../../assets/genesis_helper.rpd")).unwrap(),
            metadata_init! {
                "name" => "Genesis Helper Package".to_owned(), locked;
                "description" => "A package that defines the logic of the genesis helper which includes various utility and helper functions used in the creation of the Babylon Genesis.".to_owned(), locked;
            },
            OwnerRole::None,
        );
    }

    // Create ConsensusManager
    {
        let badge_reservation = manifest_builder.use_preallocated_address(
            VALIDATOR_OWNER_BADGE,
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
        );
        let manager_reservation = manifest_builder.use_preallocated_address(
            CONSENSUS_MANAGER,
            CONSENSUS_MANAGER_PACKAGE,
            CONSENSUS_MANAGER_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            CONSENSUS_MANAGER_PACKAGE,
            CONSENSUS_MANAGER_BLUEPRINT,
            CONSENSUS_MANAGER_CREATE_IDENT,
            ConsensusManagerCreateManifestInput {
                validator_owner_token_address: badge_reservation,
                component_address: manager_reservation,
                initial_epoch,
                initial_config,
                initial_time_ms,
                initial_current_leader,
            },
        );
    }

    // Create GenesisHelper
    {
        let reservation = manifest_builder.use_preallocated_address(
            GENESIS_HELPER,
            GENESIS_HELPER_PACKAGE,
            GENESIS_HELPER_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            GENESIS_HELPER_PACKAGE,
            GENESIS_HELPER_BLUEPRINT,
            "new",
            (
                reservation,
                CONSENSUS_MANAGER,
                system_execution(SystemExecution::Protocol),
            ),
        );
    }

    // Transaction tracker package
    {
        let reservation = manifest_builder.use_preallocated_address(
            TRANSACTION_TRACKER_PACKAGE,
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
            PACKAGE_PUBLISH_NATIVE_IDENT,
            PackagePublishNativeManifestInput {
                package_address: Some(reservation),
                native_package_code_id: NativeCodeId::TransactionTrackerCode1 as u64,
                definition: TransactionTrackerNativePackage::definition(),
                metadata: metadata_init!(),
            },
        );
    }

    // Intent Hash Store component
    {
        let reservation = manifest_builder.use_preallocated_address(
            TRANSACTION_TRACKER,
            TRANSACTION_TRACKER_PACKAGE,
            TRANSACTION_TRACKER_BLUEPRINT,
        );
        manifest_builder = manifest_builder.call_function(
            TRANSACTION_TRACKER_PACKAGE,
            TRANSACTION_TRACKER_BLUEPRINT,
            TRANSACTION_TRACKER_CREATE_IDENT,
            (reservation,),
        );
    }

    // Faucet
    // Note - the faucet is now created as part of bootstrap instead of wrap-up, to enable
    // transaction scenarios to be injected into the ledger in the node before genesis wrap-up occurs
    {
        let reservation =
            manifest_builder.use_preallocated_address(FAUCET, FAUCET_PACKAGE, FAUCET_BLUEPRINT);
        // Mint XRD for the faucet, and then deposit it into the new faucet
        // Note - on production environments, the faucet will be empty
        manifest_builder = manifest_builder
            .mint_fungible(XRD, faucet_supply)
            .take_from_worktop(XRD, faucet_supply, "faucet_xrd")
            .call_function(
                FAUCET_PACKAGE,
                FAUCET_BLUEPRINT,
                "new",
                (reservation, lookup.bucket("faucet_xrd")),
            );
    }

    manifest_builder
        .build()
        .into_transaction(hash(format!("Genesis Bootstrap")))
}

pub fn create_genesis_data_ingestion_transaction(
    chunk: GenesisDataChunk,
    chunk_index: usize,
) -> SystemTransactionV1 {
    map_address_allocations_for_manifest(chunk)
        .into_transaction(hash(format!("Genesis Data Chunk: {}", chunk_index)))
}

fn map_address_allocations_for_manifest(
    genesis_data_chunk: GenesisDataChunk,
) -> SystemTransactionManifestV1 {
    let mut manifest_builder = SystemManifestV1Builder::new_system_v1();
    let data_chunk = match genesis_data_chunk {
        GenesisDataChunk::Validators(content) => ManifestGenesisDataChunk::Validators(content),
        GenesisDataChunk::Stakes {
            accounts,
            allocations,
        } => ManifestGenesisDataChunk::Stakes {
            accounts,
            allocations,
        },
        GenesisDataChunk::Resources(genesis_resources) => {
            let resources = genesis_resources
                .into_iter()
                .map(|genesis_resource| ManifestGenesisResource {
                    resource_address_reservation: manifest_builder.use_preallocated_address(
                        genesis_resource.reserved_resource_address,
                        RESOURCE_PACKAGE,
                        FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    ),
                    metadata: genesis_resource.metadata,
                    owner: genesis_resource.owner,
                })
                .collect();
            ManifestGenesisDataChunk::Resources(resources)
        }
        GenesisDataChunk::ResourceBalances {
            accounts,
            allocations,
        } => ManifestGenesisDataChunk::ResourceBalances {
            accounts,
            allocations,
        },
        GenesisDataChunk::XrdBalances(content) => ManifestGenesisDataChunk::XrdBalances(content),
    };
    manifest_builder
        .call_method(GENESIS_HELPER, "ingest_data_chunk", (data_chunk,))
        .build()
}

pub fn create_genesis_wrap_up_transaction() -> SystemTransactionV1 {
    let manifest = ManifestBuilder::new_system_v1()
        .call_method(GENESIS_HELPER, "wrap_up", ())
        .build();

    manifest.into_transaction(hash(format!("Genesis Wrap Up")))
}
