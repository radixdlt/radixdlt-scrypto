use crate::blueprints::access_controller::*;
use crate::blueprints::account::AccountNativePackage;
use crate::blueprints::consensus_manager::ConsensusManagerNativePackage;
use crate::blueprints::identity::IdentityNativePackage;
use crate::blueprints::package::PackageNativePackage;
use crate::blueprints::pool::PoolNativePackage;
use crate::blueprints::resource::ResourceManagerNativePackage;
use crate::blueprints::transaction_processor::TransactionProcessorNativePackage;
use crate::blueprints::transaction_tracker::{
    TransactionTrackerNativePackage, TRANSACTION_TRACKER_CREATE_IDENT,
};
use crate::system::node_modules::access_rules::AccessRulesNativePackage;
use crate::system::node_modules::metadata::MetadataNativePackage;
use crate::system::node_modules::royalty::RoyaltyNativePackage;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::transaction::{
    execute_transaction, ExecutionConfig, FeeReserveConfig, TransactionReceipt,
};
use crate::types::*;
use crate::vm::wasm::WasmEngine;
use crate::vm::ScryptoVm;
use lazy_static::lazy_static;
use radix_engine_common::crypto::Secp256k1PublicKey;
use radix_engine_common::types::ComponentAddress;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use radix_engine_interface::api::node_modules::metadata::{MetadataValue, Url};
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerConfig, ConsensusManagerCreateManifestInput, EpochChangeCondition,
    CONSENSUS_MANAGER_BLUEPRINT, CONSENSUS_MANAGER_CREATE_IDENT,
};
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::{metadata_init, metadata_init_set_entry, rule};
use radix_engine_store_interface::{
    db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper},
    interface::{CommittableSubstateDatabase, SubstateDatabase},
};
use radix_engine_store_interface::db_key_mapper::MappedCommittableSubstateDatabase;
use radix_engine_store_interface::interface::{DatabaseUpdate, DatabaseUpdates};
use transaction::model::{
    BlobsV1, InstructionV1, InstructionsV1, SystemTransactionV1, TransactionPayload,
};
use transaction::prelude::{BlobV1, PreAllocatedAddress};
use transaction::validation::ManifestIdAllocator;
use radix_engine_store_interface::db_key_mapper::DatabaseKeyMapper;

const XRD_SYMBOL: &str = "XRD";
const XRD_NAME: &str = "Radix";
const XRD_DESCRIPTION: &str = "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.";
const XRD_URL: &str = "https://tokens.radixdlt.com";
const XRD_ICON_URL: &str = "https://assets.radixdlt.com/icons/icon-xrd-32x32.png";

lazy_static! {
    pub static ref DEFAULT_TESTING_FAUCET_SUPPLY: Decimal = dec!("100000000000000000");
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
                MetadataValue::Url(Url(format!("http://test.local?validator={:?}", key))),
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

pub struct Bootstrapper<'s, 'i, S, W>
where
    S: SubstateDatabase + CommittableSubstateDatabase,
    W: WasmEngine,
{
    substate_db: &'s mut S,
    scrypto_vm: &'i ScryptoVm<W>,
    trace: bool,
}

impl<'s, 'i, S, W> Bootstrapper<'s, 'i, S, W>
where
    S: SubstateDatabase + CommittableSubstateDatabase,
    W: WasmEngine,
{
    pub fn new(
        substate_db: &'s mut S,
        scrypto_vm: &'i ScryptoVm<W>,
        trace: bool,
    ) -> Bootstrapper<'s, 'i, S, W> {
        Bootstrapper {
            substate_db,
            scrypto_vm,
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
            },
            1,
            Some(0),
            *DEFAULT_TESTING_FAUCET_SUPPLY,
        )
    }

    pub fn bootstrap_with_genesis_data(
        &mut self,
        genesis_data_chunks: Vec<GenesisDataChunk>,
        initial_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
        initial_time_ms: i64,
        initial_current_leader: Option<ValidatorIndex>,
        faucet_supply: Decimal,
    ) -> Option<GenesisReceipts> {
        let substate_flash = create_system_bootstrap_flash();

        // FIXME: use substate flash data
        let xrd_info = self
            .substate_db
            .get_mapped::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
                &RADIX_TOKEN.into(),
                TYPE_INFO_FIELD_PARTITION,
                &TypeInfoField::TypeInfo.into(),
            );

        if xrd_info.is_none() {
            self.flash_substates(substate_flash);

            let system_bootstrap_receipt = self.execute_system_bootstrap(
                initial_epoch,
                initial_config,
                initial_time_ms,
                initial_current_leader,
            );

            let mut data_ingestion_receipts = vec![];
            for (chunk_index, chunk) in genesis_data_chunks.into_iter().enumerate() {
                let receipt = self.ingest_genesis_data_chunk(chunk, chunk_index);
                data_ingestion_receipts.push(receipt);
            }

            let genesis_wrap_up_receipt = self.execute_genesis_wrap_up(faucet_supply);

            Some(GenesisReceipts {
                system_bootstrap_receipt,
                data_ingestion_receipts,
                wrap_up_receipt: genesis_wrap_up_receipt,
            })
        } else {
            None
        }
    }

    fn flash_substates(&mut self, substates: BTreeMap<(NodeId, PartitionNumber), BTreeMap<SubstateKey, Vec<u8>>>) {
        let mut updates = index_map_new();

        for ((node_id, partition_num), substates) in substates {
            let partition_key = SpreadPrefixKeyMapper::to_db_partition_key(&node_id, partition_num);
            let mut partition_updates = index_map_new();
            for (substate_key, value) in substates {
                let key = SpreadPrefixKeyMapper::to_db_sort_key(&substate_key);
                let update = DatabaseUpdate::Set(value);
                partition_updates.insert(key, update);
            }

            updates.insert(partition_key, partition_updates);
        }

        self.substate_db.commit(&updates);
    }

    fn execute_system_bootstrap(
        &mut self,
        initial_epoch: Epoch,
        initial_config: ConsensusManagerConfig,
        initial_time_ms: i64,
        initial_current_leader: Option<ValidatorIndex>,
    ) -> TransactionReceipt {
        let transaction = create_system_bootstrap_transaction(
            initial_epoch,
            initial_config,
            initial_time_ms,
            initial_current_leader,
        );

        let receipt = execute_transaction(
            self.substate_db,
            self.scrypto_vm,
            &FeeReserveConfig::default(),
            &ExecutionConfig::for_genesis_transaction().with_kernel_trace(self.trace),
            &transaction
                .prepare()
                .expect("Expected system bootstrap transaction to be preparable")
                .get_executable(btreeset![AuthAddresses::system_role()]),
        );

        let commit_result = receipt.expect_commit(true);

        self.substate_db
            .commit(&commit_result.state_updates.database_updates);

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
            self.scrypto_vm,
            &FeeReserveConfig::default(),
            &ExecutionConfig::for_genesis_transaction().with_kernel_trace(self.trace),
            &transaction
                .prepare()
                .expect("Expected genesis data chunk transaction to be preparable")
                .get_executable(btreeset![AuthAddresses::system_role()]),
        );

        let commit_result = receipt.expect_commit(true);
        self.substate_db
            .commit(&commit_result.state_updates.database_updates);

        receipt
    }

    fn execute_genesis_wrap_up(&mut self, faucet_supply: Decimal) -> TransactionReceipt {
        let transaction = create_genesis_wrap_up_transaction(faucet_supply);

        let receipt = execute_transaction(
            self.substate_db,
            self.scrypto_vm,
            &FeeReserveConfig::default(),
            &ExecutionConfig::for_genesis_transaction().with_kernel_trace(self.trace),
            &transaction
                .prepare()
                .expect("Expected genesis wrap up transaction to be preparable")
                .get_executable(btreeset![AuthAddresses::system_role()]),
        );

        let commit_result = receipt.expect_commit(true);
        self.substate_db
            .commit(&commit_result.state_updates.database_updates);

        receipt
    }
}

pub fn create_system_bootstrap_flash() -> BTreeMap<(NodeId, PartitionNumber), BTreeMap<SubstateKey, Vec<u8>>> {
    btreemap!()
}

pub fn create_system_bootstrap_transaction(
    initial_epoch: Epoch,
    initial_config: ConsensusManagerConfig,
    initial_time_ms: i64,
    initial_current_leader: Option<ValidatorIndex>,
) -> SystemTransactionV1 {
    // NOTES
    // * Create resources before packages to avoid circular dependencies.

    let mut id_allocator = ManifestIdAllocator::new();
    let mut instructions = Vec::new();
    let mut pre_allocated_addresses = vec![];
    let mut blobs = vec![];

    // Package Package
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(PACKAGE_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                native_package_code_id: PACKAGE_CODE_ID,
                setup: PackageNativePackage::definition(),
                metadata: BTreeMap::new(),
            }),
        });
    }

    // TransactionProcessor Package
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(TRANSACTION_PROCESSOR_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                setup: TransactionProcessorNativePackage::definition(),
                metadata: BTreeMap::new(),
                native_package_code_id: TRANSACTION_PROCESSOR_CODE_ID,
            }),
        });
    }

    // Metadata Package
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(METADATA_MODULE_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                native_package_code_id: METADATA_CODE_ID,
                setup: MetadataNativePackage::definition(),
                metadata: BTreeMap::new(),
            }),
        });
    }

    // Access Rules Package
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(ROYALTY_MODULE_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                native_package_code_id: ROYALTY_CODE_ID,
                setup: RoyaltyNativePackage::definition(),
                metadata: BTreeMap::new(),
            }),
        });
    }

    // Resource Package
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(ACCESS_RULES_MODULE_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                native_package_code_id: ACCESS_RULES_CODE_ID,
                setup: AccessRulesNativePackage::definition(),
                metadata: BTreeMap::new(),
            }),
        });
    }

    // Royalty Package
    {
        pre_allocated_addresses.push((
            BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
            GlobalAddress::from(RESOURCE_PACKAGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishNativeManifestInput {
                package_address: Some(id_allocator.new_address_reservation_id()),
                native_package_code_id: RESOURCE_MANAGER_CODE_ID,
                setup: ResourceManagerNativePackage::definition(),
                metadata: BTreeMap::new(),
            }),
        });
    }

    // XRD Token
    {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        access_rules.insert(
            Mint,
            (
                rule!(require(global_caller(CONSENSUS_MANAGER))),
                rule!(deny_all),
            ),
        );
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(RADIX_TOKEN),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT
                .to_string(),
            args: to_manifest_value_and_unwrap!(
                &FungibleResourceManagerCreateWithInitialSupplyAndAddressManifestInput {
                    track_total_supply: false,
                    divisibility: 18,
                    metadata: metadata_init! {
                        "symbol" => XRD_SYMBOL.to_owned(), locked;
                        "name" => XRD_NAME.to_owned(), locked;
                        "description" => XRD_DESCRIPTION.to_owned(), locked;
                        "url" => XRD_URL.to_owned(), locked;
                        "icon_url" => XRD_ICON_URL.to_owned(), locked;
                    },
                    access_rules,
                    initial_supply: Decimal::zero(),
                    resource_address: id_allocator.new_address_reservation_id(),
                }
            ),
        });
    }

    // Package Token
    {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(deny_all), rule!(deny_all)));
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateWithAddressManifestInput {
                    id_type: NonFungibleIdType::Bytes,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                    metadata: metadata_init!(),
                    access_rules,
                    resource_address: id_allocator.new_address_reservation_id(),
                }
            ),
        });
    }

    // Object Token
    {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(deny_all), rule!(deny_all)));
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(GLOBAL_CALLER_VIRTUAL_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateWithAddressManifestInput {
                    id_type: NonFungibleIdType::Bytes,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                    metadata: metadata_init!(),
                    access_rules,
                    resource_address: id_allocator.new_address_reservation_id(),
                }
            ),
        });
    }

    // Package Owner Token
    {
        // TODO: Integrate this into package instantiation to remove circular dependency
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            Mint,
            (
                rule!(require(package_of_direct_caller(PACKAGE_PACKAGE))),
                rule!(deny_all),
            ),
        );
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(PACKAGE_OWNER_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateWithAddressManifestInput {
                    id_type: NonFungibleIdType::RUID,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                    metadata: metadata_init!(),
                    access_rules,
                    resource_address: id_allocator.new_address_reservation_id(),
                }
            ),
        });
    }

    // Identity Package
    {
        // TODO: Integrate this into package instantiation to remove circular dependency
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            Mint,
            (
                rule!(require(package_of_direct_caller(IDENTITY_PACKAGE))),
                rule!(deny_all),
            ),
        );
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(IDENTITY_OWNER_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateWithAddressManifestInput {
                    id_type: NonFungibleIdType::RUID,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                    metadata: metadata_init!(),
                    access_rules,
                    resource_address: id_allocator.new_address_reservation_id(),
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
                setup: IdentityNativePackage::definition(),
                native_package_code_id: IDENTITY_CODE_ID,
                metadata: BTreeMap::new(),
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
                setup: ConsensusManagerNativePackage::definition(),
                native_package_code_id: CONSENSUS_MANAGER_CODE_ID,
                metadata: BTreeMap::new(),
            }),
        });
    }

    // Account Package
    {
        // TODO: Integrate this into package instantiation to remove circular dependency
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            Mint,
            (
                rule!(require(package_of_direct_caller(ACCOUNT_PACKAGE))),
                rule!(deny_all),
            ),
        );
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(ACCOUNT_OWNER_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateWithAddressManifestInput {
                    id_type: NonFungibleIdType::RUID,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                    metadata: metadata_init!(),
                    access_rules,
                    resource_address: id_allocator.new_address_reservation_id(),
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
                setup: AccountNativePackage::definition(),
                native_package_code_id: ACCOUNT_CODE_ID,
                metadata: BTreeMap::new(),
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
                setup: AccessControllerNativePackage::definition(),
                metadata: BTreeMap::new(),
                native_package_code_id: ACCESS_CONTROLLER_CODE_ID,
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
                setup: PoolNativePackage::definition(),
                metadata: BTreeMap::new(),
                native_package_code_id: POOL_CODE_ID,
            }),
        });
    }


    // ECDSA Secp256k1
    {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(SECP256K1_SIGNATURE_VIRTUAL_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateWithAddressManifestInput {
                    id_type: NonFungibleIdType::Bytes,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                    metadata: metadata_init!(),
                    access_rules,
                    resource_address: id_allocator.new_address_reservation_id(),
                }
            ),
        });
    }

    // Ed25519
    {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(ED25519_SIGNATURE_VIRTUAL_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateWithAddressManifestInput {
                    id_type: NonFungibleIdType::Bytes,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                    metadata: metadata_init!(),
                    access_rules,
                    resource_address: id_allocator.new_address_reservation_id(),
                }
            ),
        });
    }

    // System Token
    {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        pre_allocated_addresses.push((
            BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
            GlobalAddress::from(SYSTEM_TRANSACTION_BADGE),
        ));
        instructions.push(InstructionV1::CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(
                &NonFungibleResourceManagerCreateWithAddressManifestInput {
                    id_type: NonFungibleIdType::Bytes,
                    track_total_supply: false,
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                    metadata: metadata_init!(),
                    access_rules,
                    resource_address: id_allocator.new_address_reservation_id(),
                }
            ),
        });
    }

    // Faucet Package
    {
        let faucet_code = include_bytes!("../../../assets/faucet.wasm").to_vec();
        let faucet_abi = include_bytes!("../../../assets/faucet.schema").to_vec();
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
                setup: manifest_decode(&faucet_abi).unwrap(),
                metadata: BTreeMap::new(),
                owner_role: OwnerRole::None,
            }),
        });
    }

    // Genesis helper package
    {
        // FIXME: Add authorization rules around preventing anyone else from
        // calling genesis helper code
        let genesis_helper_code = include_bytes!("../../../assets/genesis_helper.wasm").to_vec();
        let genesis_helper_abi = include_bytes!("../../../assets/genesis_helper.schema").to_vec();
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
                setup: manifest_decode(&genesis_helper_abi).unwrap(),
                metadata: BTreeMap::new(),
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
            ),
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
                native_package_code_id: TRANSACTION_TRACKER_CODE_ID,
                setup: TransactionTrackerNativePackage::definition(),
                metadata: BTreeMap::new(),
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
            args: manifest_args!(id_allocator.new_address_reservation_id()),
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
        args: manifest_args!(chunk),
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

pub fn create_genesis_wrap_up_transaction(faucet_supply: Decimal) -> SystemTransactionV1 {
    let mut id_allocator = ManifestIdAllocator::new();
    let mut instructions = Vec::new();

    instructions.push(InstructionV1::CallMethod {
        address: GENESIS_HELPER.clone().into(),
        method_name: "wrap_up".to_string(),
        args: manifest_args!(),
    });

    instructions.push(
        InstructionV1::CallMethod {
            address: RADIX_TOKEN.clone().into(),
            method_name: FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
            args: manifest_args!(faucet_supply),
        }
        .into(),
    );

    instructions.push(
        InstructionV1::TakeAllFromWorktop {
            resource_address: RADIX_TOKEN,
        }
        .into(),
    );

    let bucket = id_allocator.new_bucket_id();

    instructions.push(InstructionV1::CallFunction {
        package_address: FAUCET_PACKAGE.into(),
        blueprint_name: FAUCET_BLUEPRINT.to_string(),
        function_name: "new".to_string(),
        args: manifest_args!(ManifestAddressReservation(0), bucket),
    });

    SystemTransactionV1 {
        instructions: InstructionsV1(instructions),
        pre_allocated_addresses: vec![PreAllocatedAddress {
            blueprint_id: BlueprintId::new(&FAUCET_PACKAGE, FAUCET_BLUEPRINT),
            address: FAUCET.into(),
        }],
        blobs: BlobsV1 { blobs: vec![] },
        hash_for_execution: hash(format!("Genesis Wrap Up")),
    }
}
