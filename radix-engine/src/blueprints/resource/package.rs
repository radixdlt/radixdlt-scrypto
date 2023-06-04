use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::system_callback::SystemLockData;
use crate::system::system_modules::costing::{FIXED_HIGH_FEE, FIXED_LOW_FEE, FIXED_MEDIUM_FEE};
use crate::types::*;
use crate::{event_schema, method_auth_template};
use radix_engine_interface::api::node_modules::metadata::{
    METADATA_GET_IDENT, METADATA_REMOVE_IDENT, METADATA_SET_IDENT,
};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::{
    BlueprintSetup, BlueprintTemplate, PackageSetup,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::{BlueprintCollectionSchema, FeaturedSchema, FieldSchema, SchemaMethodKey, SchemaMethodPermission};
use radix_engine_interface::schema::{BlueprintIndexSchema, FunctionSchema};
use radix_engine_interface::schema::{BlueprintKeyValueStoreSchema, BlueprintSchema, TypeRef};
use radix_engine_interface::schema::{Receiver, ReceiverInfo, RefTypes};
use resources_tracker_macro::trace_resources;

const FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME: &str = "create_FungibleResourceManager";
const FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME: &str =
    "create_with_initial_supply_and_address_FungibleResourceManager";
const FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_EXPORT_NAME: &str =
    "create_with_initial_supply_FungibleResourceManager";
const FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME: &str = "burn_FungibleResourceManager";
const FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME: &str = "mint_FungibleResourceManager";
const FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME: &str =
    "create_empty_vault_FungibleResourceManager";
const FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME: &str =
    "create_empty_bucket_FungibleResourceManager";
const FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME: &str =
    "get_resource_type_FungibleResourceManager";
const FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME: &str =
    "get_total_supply_FungibleResourceManager";
const FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME: &str =
    "drop_empty_bucket_FungibleResourceManager";

const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME: &str = "create_NonFungibleResourceManager";
const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME: &str =
    "create_with_initial_supply_NonFungibleResourceManager";
const NON_FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME: &str = "burn_NonFungibleResourceManager";
const NON_FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME: &str = "mint_NonFungibleResourceManager";
const NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_EXPORT_NAME: &str =
    "mint_uuid_NonFungibleResourceManager";
const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME: &str =
    "create_empty_vault_NonFungibleResourceManager";
const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME: &str =
    "create_empty_bucket_NonFungibleResourceManager";
const NON_FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME: &str =
    "get_resource_type_NonFungibleResourceManager";
const NON_FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME: &str =
    "get_total_supply_NonFungibleResourceManager";
const NON_FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME: &str =
    "drop_empty_bucket_NonFungibleResourceManager";

const FUNGIBLE_VAULT_TAKE_EXPORT_NAME: &str = "take_FungibleVault";
const FUNGIBLE_VAULT_PUT_EXPORT_NAME: &str = "put_FungibleVault";
const FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_FungibleVault";
const FUNGIBLE_VAULT_RECALL_EXPORT_NAME: &str = "recall_FungibleVault";
const FUNGIBLE_VAULT_FREEZE_EXPORT_NAME: &str = "freeze_FungibleVault";
const FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME: &str = "unfreeze_FungibleVault";
const FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME: &str = "create_proof_FungibleVault";
const FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME: &str =
    "create_proof_of_amount_FungibleVault";
const FUNGIBLE_VAULT_LOCK_AMOUNT_EXPORT_NAME: &str = "lock_amount_FungibleVault";
const FUNGIBLE_VAULT_UNLOCK_AMOUNT_EXPORT_NAME: &str = "unlock_amount_FungibleVault";

const NON_FUNGIBLE_VAULT_TAKE_EXPORT_NAME: &str = "take_NonFungibleVault";
const NON_FUNGIBLE_VAULT_PUT_EXPORT_NAME: &str = "put_NonFungibleVault";
const NON_FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_NonFungibleVault";
const NON_FUNGIBLE_VAULT_RECALL_EXPORT_NAME: &str = "recall_NonFungibleVault";
const NON_FUNGIBLE_VAULT_FREEZE_EXPORT_NAME: &str = "freeze_NonFungibleVault";
const NON_FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME: &str = "unfreeze_NonFungibleVault";
const NON_FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME: &str = "create_proof_NonFungibleVault";
const NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME: &str =
    "create_proof_of_amount_NonFungibleVault";
const NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_EXPORT_NAME: &str = "unlock_fungibles_NonFungibleVault";
const NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_EXPORT_NAME: &str =
    "unlock_non_fungibles_NonFungibleVault";

const FUNGIBLE_BUCKET_TAKE_EXPORT_NAME: &str = "take_FungibleBucket";
const FUNGIBLE_BUCKET_PUT_EXPORT_NAME: &str = "put_FungibleBucket";
const FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_FungibleBucket";
const FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME: &str =
    "get_resource_address_FungibleBucket";
const FUNGIBLE_BUCKET_CREATE_PROOF_EXPORT_NAME: &str = "create_proof_FungibleBucket";
const FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME: &str =
    "create_proof_of_amount_FungibleBucket";
const FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME: &str = "create_proof_of_all_FungibleBucket";
const FUNGIBLE_BUCKET_LOCK_AMOUNT_EXPORT_NAME: &str = "lock_amount_FungibleBucket";
const FUNGIBLE_BUCKET_UNLOCK_AMOUNT_EXPORT_NAME: &str = "unlock_amount_FungibleBucket";

const NON_FUNGIBLE_BUCKET_TAKE_EXPORT_NAME: &str = "take_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_PUT_EXPORT_NAME: &str = "put_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME: &str =
    "get_resource_address_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_CREATE_PROOF_EXPORT_NAME: &str = "create_proof_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME: &str =
    "create_proof_of_amount_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_EXPORT_NAME: &str =
    "create_proof_of_non_fungibles_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME: &str =
    "create_proof_of_all_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_EXPORT_NAME: &str =
    "unlock_fungibles_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_EXPORT_NAME: &str =
    "unlock_non_fungibles_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_EXPORT_NAME: &str =
    "take_non_fungibles_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_EXPORT_NAME: &str =
    "get_non_fungible_local_ids_NonFungibleBucket";

const FUNGIBLE_PROOF_CLONE_EXPORT_NAME: &str = "clone_FungibleProof";
const FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_FungibleProof";
const FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME: &str = "get_resource_address_FungibleProof";
const FUNGIBLE_PROOF_DROP_EXPORT_NAME: &str = "drop_FungibleProof";

const NON_FUNGIBLE_PROOF_CLONE_EXPORT_NAME: &str = "clone_NonFungibleProof";
const NON_FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_NonFungibleProof";
const NON_FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME: &str =
    "get_resource_address_NonFungibleProof";
const NON_FUNGIBLE_PROOF_DROP_EXPORT_NAME: &str = "drop_NonFungibleProof";

pub struct ResourceManagerNativePackage;

impl ResourceManagerNativePackage {
    pub fn definition() -> PackageSetup {
        //====================================================================================

        let fungible_resource_manager_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(
                FieldSchema::normal(aggregator
                    .add_child_type_and_descendents::<FungibleResourceManagerDivisibilitySubstate>(
                    )),
            );
            fields.push(
                FieldSchema::Conditional {
                    feature: TRACK_TOTAL_SUPPLY_FEATURE.to_string(),
                    value: aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerTotalSupplySubstate>(),
                }
            );

            let mut functions = BTreeMap::new();
            functions.insert(
                FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                FunctionSchema {
                    receiver: None,
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME),
                },
            );
            functions.insert(
                FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT.to_string(),
                FunctionSchema {
                    receiver: None,
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateWithInitialSupplyInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateWithInitialSupplyOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME),
                },
            );
            functions.insert(
                FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT.to_string(),
                FunctionSchema {
                    receiver: None,
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateWithInitialSupplyAndAddressInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateWithInitialSupplyAndAddressOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_EXPORT_NAME),
                },
            );

            functions.insert(
                FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerMintInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerMintOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_BURN_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<ResourceManagerBurnInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerBurnOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME),
                },
            );

            functions.insert(
                RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME),
                },
            );

            functions.insert(
                RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetResourceTypeInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetResourceTypeOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyOutput>(),
                    export: FeaturedSchema::Conditional {
                        feature: TRACK_TOTAL_SUPPLY_FEATURE.to_string(),
                        value: FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME.to_string()
                    },
                },
            );
            functions.insert(
                RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME),
                },
            );

            let event_schema = event_schema! {
                aggregator,
                [
                    VaultCreationEvent,
                    MintFungibleResourceEvent,
                    BurnFungibleResourceEvent
                ]
            };

            let schema = generate_full_schema(aggregator);
            BlueprintSchema {
                outer_blueprint: None,
                schema,
                fields,
                collections: vec![],
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema,
                dependencies: btreeset!(),
            }
        };

        //====================================================================================

        let non_fungible_resource_manager_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(
                FieldSchema::normal(aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerIdTypeSubstate>()),
            );
            fields.push(
                FieldSchema::normal(aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerMutableFieldsSubstate>(
                    )),
            );
            fields.push(
                FieldSchema::Conditional {
                    feature: TRACK_TOTAL_SUPPLY_FEATURE.to_string(),
                    value: aggregator.add_child_type_and_descendents::<NonFungibleResourceManagerTotalSupplySubstate>(),
                }
            );

            let mut collections = Vec::new();
            collections.push(BlueprintCollectionSchema::KeyValueStore(
                BlueprintKeyValueStoreSchema {
                    key: TypeRef::Blueprint(
                        aggregator.add_child_type_and_descendents::<NonFungibleLocalId>(),
                    ),
                    value: TypeRef::Instance(0u8),
                    can_own: false,
                },
            ));

            let mut functions = BTreeMap::new();
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                FunctionSchema {
                    receiver: None,
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME),
                },
            );
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
                FunctionSchema {
                    receiver: None,
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateWithAddressInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateWithAddressOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT),
                },
            );
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT.to_string(),
                FunctionSchema {
                    receiver: None,
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateWithInitialSupplyInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateWithInitialSupplyOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME),
                },
            );
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_UUID_WITH_INITIAL_SUPPLY_IDENT.to_string(),
                FunctionSchema {
                    receiver: None,
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateUuidWithInitialSupplyInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateUuidWithInitialSupplyOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_UUID_WITH_INITIAL_SUPPLY_IDENT),
                },
            );

            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME),
                },
            );

            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerGetNonFungibleInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerGetNonFungibleOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT),
                },
            );

            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerUpdateDataInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerUpdateDataOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT),
                },
            );
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerExistsInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerExistsOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT),
                },
            );

            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintUuidInput>(
                        ),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintUuidOutput>(
                        ),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_EXPORT_NAME),
                },
            );
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintSingleUuidInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintSingleUuidOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT),
                },
            );

            functions.insert(
                RESOURCE_MANAGER_BURN_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<ResourceManagerBurnInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerBurnOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME),
                },
            );

            functions.insert(
                RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetResourceTypeInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetResourceTypeOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyOutput>(),
                    export: FeaturedSchema::Conditional {
                        feature: TRACK_TOTAL_SUPPLY_FEATURE.to_string(),
                        value: NON_FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME.to_string(),
                    },
                },
            );
            functions.insert(
                RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME),
                },
            );

            let event_schema = event_schema! {
                aggregator,
                [
                    VaultCreationEvent,
                    MintNonFungibleResourceEvent,
                    BurnNonFungibleResourceEvent
                ]
            };

            let schema = generate_full_schema(aggregator);
            BlueprintSchema {
                outer_blueprint: None,
                schema,
                fields,
                collections,
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema,
                dependencies: btreeset!(),
            }
        };

        //====================================================================================

        let fungible_vault_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
            let mut fields = Vec::new();
            fields
                .push(FieldSchema::normal(aggregator.add_child_type_and_descendents::<FungibleVaultBalanceSubstate>()));
            fields.push(FieldSchema::normal(aggregator.add_child_type_and_descendents::<LockedFungibleResource>()));

            let mut functions = BTreeMap::new();
            functions.insert(
                VAULT_TAKE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<VaultTakeInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultTakeOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_VAULT_TAKE_EXPORT_NAME),
                },
            );
            functions.insert(
                VAULT_PUT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<VaultPutInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultPutOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_VAULT_PUT_EXPORT_NAME),
                },
            );
            functions.insert(
                VAULT_GET_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator.add_child_type_and_descendents::<VaultGetAmountInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultGetAmountOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME),
                },
            );
            functions.insert(
                FUNGIBLE_VAULT_LOCK_FEE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<FungibleVaultLockFeeInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleVaultLockFeeOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_VAULT_LOCK_FEE_IDENT),
                },
            );
            functions.insert(
                VAULT_RECALL_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: aggregator.add_child_type_and_descendents::<VaultRecallInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultRecallOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_VAULT_RECALL_EXPORT_NAME),
                },
            );
            functions.insert(
                VAULT_FREEZE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: aggregator.add_child_type_and_descendents::<VaultFreezeInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultFreezeOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_VAULT_FREEZE_EXPORT_NAME),
                },
            );
            functions.insert(
                VAULT_UNFREEZE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: aggregator.add_child_type_and_descendents::<VaultUnfreezeInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultUnfreezeOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME),
                },
            );
            functions.insert(
                VAULT_CREATE_PROOF_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<VaultCreateProofInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultCreateProofOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME),
                },
            );
            functions.insert(
                VAULT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<VaultCreateProofOfAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<VaultCreateProofOfAmountOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME),
                },
            );
            functions.insert(
                FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleVaultLockFungibleAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleVaultLockFungibleAmountOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_VAULT_LOCK_AMOUNT_EXPORT_NAME),
                },
            );
            functions.insert(
                FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleVaultUnlockFungibleAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleVaultUnlockFungibleAmountOutput>(
                        ),
                    export: FeaturedSchema::normal(FUNGIBLE_VAULT_UNLOCK_AMOUNT_EXPORT_NAME),
                },
            );

            let event_schema = event_schema! {
                aggregator,
                [
                    LockFeeEvent,
                    WithdrawResourceEvent,
                    DepositResourceEvent,
                    RecallResourceEvent
                ]
            };

            let schema = generate_full_schema(aggregator);

            BlueprintSchema {
                outer_blueprint: Some(FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                schema,
                fields,
                collections: vec![],
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema,
                dependencies: btreeset!(),
            }
        };

        //====================================================================================

        let non_fungible_vault_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
            let mut fields = Vec::new();
            fields.push(
                FieldSchema::normal(aggregator.add_child_type_and_descendents::<NonFungibleVaultBalanceSubstate>()),
            );
            fields.push(FieldSchema::normal(aggregator.add_child_type_and_descendents::<LockedNonFungibleResource>()));

            let mut collections = Vec::new();
            collections.push(BlueprintCollectionSchema::Index(BlueprintIndexSchema {}));

            let mut functions = BTreeMap::new();
            functions.insert(
                VAULT_TAKE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<VaultTakeInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultTakeOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_TAKE_EXPORT_NAME),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultTakeNonFungiblesInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultTakeNonFungiblesOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT),
                },
            );
            functions.insert(
                VAULT_RECALL_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: aggregator.add_child_type_and_descendents::<VaultRecallInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultRecallOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_RECALL_EXPORT_NAME),
                },
            );
            functions.insert(
                VAULT_FREEZE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: aggregator.add_child_type_and_descendents::<VaultFreezeInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultFreezeOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_FREEZE_EXPORT_NAME),
                },
            );
            functions.insert(
                VAULT_UNFREEZE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: aggregator.add_child_type_and_descendents::<VaultUnfreezeInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultUnfreezeOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultRecallNonFungiblesInput>(
                        ),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultRecallNonFungiblesOutput>(
                        ),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT),
                },
            );
            functions.insert(
                VAULT_PUT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<VaultPutInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultPutOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_PUT_EXPORT_NAME),
                },
            );
            functions.insert(
                VAULT_GET_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator.add_child_type_and_descendents::<VaultGetAmountInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultGetAmountOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultGetNonFungibleLocalIdsInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultGetNonFungibleLocalIdsOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT),
                },
            );
            functions.insert(
                VAULT_CREATE_PROOF_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<VaultCreateProofInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultCreateProofOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME),
                },
            );
            functions.insert(
                VAULT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<VaultCreateProofOfAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<VaultCreateProofOfAmountOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultCreateProofOfNonFungiblesInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultCreateProofOfNonFungiblesOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultLockNonFungiblesInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultLockNonFungiblesOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_EXPORT_NAME),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultUnlockNonFungiblesInput>(
                        ),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultUnlockNonFungiblesOutput>(
                        ),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_EXPORT_NAME),
                },
            );

            let event_schema = event_schema! {
                aggregator,
                [
                    LockFeeEvent,
                    WithdrawResourceEvent,
                    DepositResourceEvent,
                    RecallResourceEvent
                ]
            };

            let schema = generate_full_schema(aggregator);

            BlueprintSchema {
                outer_blueprint: Some(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                schema,
                fields,
                collections,
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema,
                dependencies: btreeset!(),
            }
        };

        //====================================================================================

        let fungible_bucket_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::normal(aggregator.add_child_type_and_descendents::<LiquidFungibleResource>()));
            fields.push(FieldSchema::normal(aggregator.add_child_type_and_descendents::<LockedFungibleResource>()));

            let mut functions = BTreeMap::new();
            functions.insert(
                BUCKET_PUT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<BucketPutInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketPutOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_BUCKET_PUT_EXPORT_NAME),
                },
            );
            functions.insert(
                BUCKET_TAKE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<BucketTakeInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketTakeOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_BUCKET_TAKE_EXPORT_NAME),
                },
            );
            functions.insert(
                BUCKET_GET_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator.add_child_type_and_descendents::<BucketGetAmountInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketGetAmountOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME),
                },
            );
            functions.insert(
                BUCKET_GET_RESOURCE_ADDRESS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<BucketGetResourceAddressInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<BucketGetResourceAddressOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<BucketCreateProofInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketCreateProofOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_BUCKET_CREATE_PROOF_EXPORT_NAME),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<BucketCreateProofOfAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<BucketCreateProofOfAmountOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_OF_ALL_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<BucketCreateProofOfAllInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<BucketCreateProofOfAllOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME),
                },
            );
            functions.insert(
                FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleBucketLockAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleBucketLockAmountOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_BUCKET_LOCK_AMOUNT_EXPORT_NAME),
                },
            );
            functions.insert(
                FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleBucketUnlockAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleBucketUnlockAmountOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_BUCKET_UNLOCK_AMOUNT_EXPORT_NAME),
                },
            );

            let schema = generate_full_schema(aggregator);
            BlueprintSchema {
                outer_blueprint: Some(FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                schema,
                fields,
                collections: vec![],
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema: [].into(),
                dependencies: btreeset!(),
            }
        };

        //====================================================================================

        let non_fungible_bucket_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::normal(aggregator.add_child_type_and_descendents::<LiquidNonFungibleResource>()));
            fields.push(FieldSchema::normal(aggregator.add_child_type_and_descendents::<LockedNonFungibleResource>()));

            let mut functions = BTreeMap::new();
            functions.insert(
                BUCKET_PUT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<BucketPutInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketPutOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_BUCKET_PUT_EXPORT_NAME),
                },
            );
            functions.insert(
                BUCKET_TAKE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<BucketTakeInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketTakeOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_BUCKET_TAKE_EXPORT_NAME),
                },
            );

            functions.insert(
                BUCKET_GET_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator.add_child_type_and_descendents::<BucketGetAmountInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketGetAmountOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME),
                },
            );
            functions.insert(
                BUCKET_GET_RESOURCE_ADDRESS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<BucketGetResourceAddressInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<BucketGetResourceAddressOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<BucketCreateProofInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketCreateProofOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_BUCKET_CREATE_PROOF_EXPORT_NAME),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<BucketCreateProofOfAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<BucketCreateProofOfAmountOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<NonFungibleBucketCreateProofOfNonFungiblesInput>(),
                    output: aggregator.add_child_type_and_descendents::<NonFungibleBucketCreateProofOfNonFungiblesOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_EXPORT_NAME),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_OF_ALL_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<BucketCreateProofOfAllInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<BucketCreateProofOfAllOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<BucketTakeNonFungiblesInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<BucketTakeNonFungiblesOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_EXPORT_NAME),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<BucketGetNonFungibleLocalIdsInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<BucketGetNonFungibleLocalIdsOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_EXPORT_NAME),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleBucketLockNonFungiblesInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleBucketLockNonFungiblesOutput>(
                        ),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_EXPORT_NAME),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleBucketUnlockNonFungiblesInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleBucketUnlockNonFungiblesOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_EXPORT_NAME),
                },
            );

            let schema = generate_full_schema(aggregator);
            BlueprintSchema {
                outer_blueprint: Some(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                schema,
                fields,
                collections: vec![],
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema: [].into(),
                dependencies: btreeset!(),
            }
        };

        let fungible_proof_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::normal(aggregator.add_child_type_and_descendents::<ProofMoveableSubstate>()));
            fields.push(FieldSchema::normal(aggregator.add_child_type_and_descendents::<FungibleProofSubstate>()));

            let mut functions = BTreeMap::new();
            functions.insert(
                PROOF_DROP_IDENT.to_string(),
                FunctionSchema {
                    receiver: None,
                    input: aggregator.add_child_type_and_descendents::<ProofDropInput>(),
                    output: aggregator.add_child_type_and_descendents::<ProofDropOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_PROOF_DROP_EXPORT_NAME),
                },
            );
            functions.insert(
                PROOF_CLONE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<ProofCloneInput>(),
                    output: aggregator.add_child_type_and_descendents::<ProofCloneOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_PROOF_CLONE_EXPORT_NAME),
                },
            );
            functions.insert(
                PROOF_GET_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator.add_child_type_and_descendents::<ProofGetAmountInput>(),
                    output: aggregator.add_child_type_and_descendents::<ProofGetAmountOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME),
                },
            );
            functions.insert(
                PROOF_GET_RESOURCE_ADDRESS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<ProofGetResourceAddressInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ProofGetResourceAddressOutput>(),
                    export: FeaturedSchema::normal(FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME),
                },
            );

            let schema = generate_full_schema(aggregator);
            BlueprintSchema {
                outer_blueprint: Some(FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                schema,
                fields,
                collections: vec![],
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema: [].into(),
                dependencies: btreeset!(),
            }
        };

        let non_fungible_proof_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::normal(aggregator.add_child_type_and_descendents::<ProofMoveableSubstate>()));
            fields.push(FieldSchema::normal(aggregator.add_child_type_and_descendents::<NonFungibleProofSubstate>()));

            let mut functions = BTreeMap::new();
            functions.insert(
                PROOF_DROP_IDENT.to_string(),
                FunctionSchema {
                    receiver: None,
                    input: aggregator.add_child_type_and_descendents::<ProofDropInput>(),
                    output: aggregator.add_child_type_and_descendents::<ProofDropOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_PROOF_DROP_EXPORT_NAME),
                },
            );
            functions.insert(
                PROOF_CLONE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator.add_child_type_and_descendents::<ProofCloneInput>(),
                    output: aggregator.add_child_type_and_descendents::<ProofCloneOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_PROOF_CLONE_EXPORT_NAME),
                },
            );
            functions.insert(
                PROOF_GET_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator.add_child_type_and_descendents::<ProofGetAmountInput>(),
                    output: aggregator.add_child_type_and_descendents::<ProofGetAmountOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME),
                },
            );
            functions.insert(
                PROOF_GET_RESOURCE_ADDRESS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<ProofGetResourceAddressInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ProofGetResourceAddressOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME),
                },
            );

            functions.insert(
                NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleProofGetLocalIdsInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleProofGetLocalIdsOutput>(),
                    export: FeaturedSchema::normal(NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT),
                },
            );

            let schema = generate_full_schema(aggregator);
            BlueprintSchema {
                outer_blueprint: Some(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                schema,
                fields,
                collections: vec![],
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema: [].into(),
                dependencies: btreeset!(),
            }
        };

        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(FieldSchema::normal(aggregator.add_child_type_and_descendents::<WorktopSubstate>()));

        let mut functions = BTreeMap::new();
        functions.insert(
            WORKTOP_DROP_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<WorktopDropInput>(),
                output: aggregator.add_child_type_and_descendents::<WorktopDropOutput>(),
                export: FeaturedSchema::normal(WORKTOP_DROP_IDENT),
            },
        );
        functions.insert(
            WORKTOP_PUT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<WorktopPutInput>(),
                output: aggregator.add_child_type_and_descendents::<WorktopPutOutput>(),
                export: FeaturedSchema::normal(WORKTOP_PUT_IDENT),
            },
        );
        functions.insert(
            WORKTOP_TAKE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<WorktopTakeInput>(),
                output: aggregator.add_child_type_and_descendents::<WorktopTakeOutput>(),
                export: FeaturedSchema::normal(WORKTOP_TAKE_IDENT),
            },
        );
        functions.insert(
            WORKTOP_TAKE_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<WorktopTakeNonFungiblesInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<WorktopTakeNonFungiblesOutput>(),
                export: FeaturedSchema::normal(WORKTOP_TAKE_NON_FUNGIBLES_IDENT),
            },
        );
        functions.insert(
            WORKTOP_TAKE_ALL_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<WorktopTakeAllInput>(),
                output: aggregator.add_child_type_and_descendents::<WorktopTakeAllOutput>(),
                export: FeaturedSchema::normal(WORKTOP_TAKE_ALL_IDENT),
            },
        );
        functions.insert(
            WORKTOP_ASSERT_CONTAINS_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<WorktopAssertContainsInput>(),
                output: aggregator.add_child_type_and_descendents::<WorktopAssertContainsOutput>(),
                export: FeaturedSchema::normal(WORKTOP_ASSERT_CONTAINS_IDENT),
            },
        );
        functions.insert(
            WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<WorktopAssertContainsAmountInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<WorktopAssertContainsAmountOutput>(),
                export: FeaturedSchema::normal(WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT),
            },
        );
        functions.insert(
            WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<WorktopAssertContainsNonFungiblesInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<WorktopAssertContainsNonFungiblesOutput>(),
                export: FeaturedSchema::normal(WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT),
            },
        );
        functions.insert(
            WORKTOP_DRAIN_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<WorktopDrainInput>(),
                output: aggregator.add_child_type_and_descendents::<WorktopDrainOutput>(),
                export: FeaturedSchema::normal(WORKTOP_DRAIN_IDENT),
            },
        );
        let schema = generate_full_schema(aggregator);
        let worktop_schema = BlueprintSchema {
            outer_blueprint: None,
            schema,
            fields,
            collections: vec![],
            functions,
            virtual_lazy_load_functions: btreemap!(),
            event_schema: [].into(),
            dependencies: btreeset!(),
        };

        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(FieldSchema::normal(aggregator.add_child_type_and_descendents::<AuthZone>()));

        let mut functions = BTreeMap::new();
        functions.insert(
            AUTH_ZONE_POP_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AuthZonePopInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZonePopOutput>(),
                export: FeaturedSchema::normal(AUTH_ZONE_POP_EXPORT_NAME),
            },
        );
        functions.insert(
            AUTH_ZONE_PUSH_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AuthZonePushInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZonePushOutput>(),
                export: FeaturedSchema::normal(AUTH_ZONE_PUSH_EXPORT_NAME),
            },
        );
        functions.insert(
            AUTH_ZONE_CREATE_PROOF_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AuthZoneCreateProofInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZoneCreateProofOutput>(),
                export: FeaturedSchema::normal(AUTH_ZONE_CREATE_PROOF_EXPORT_NAME),
            },
        );
        functions.insert(
            AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AuthZoneCreateProofOfAmountInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AuthZoneCreateProofOfAmountOutput>(),
                export: FeaturedSchema::normal(AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME),
            },
        );
        functions.insert(
            AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AuthZoneCreateProofOfNonFungiblesInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AuthZoneCreateProofOfNonFungiblesOutput>(),
                export: FeaturedSchema::normal(AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_EXPORT_NAME),
            },
        );
        functions.insert(
            AUTH_ZONE_CREATE_PROOF_OF_ALL_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AuthZoneCreateProofOfAllInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AuthZoneCreateProofOfAllOutput>(),
                export: FeaturedSchema::normal(AUTH_ZONE_CREATE_PROOF_OF_ALL_EXPORT_NAME),
            },
        );
        functions.insert(
            AUTH_ZONE_CLEAR_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AuthZoneClearInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZoneClearOutput>(),
                export: FeaturedSchema::normal(AUTH_ZONE_CLEAR_EXPORT_NAME),
            },
        );
        functions.insert(
            AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AuthZoneClearVirtualProofsInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AuthZoneClearVirtualProofsOutput>(),
                export: FeaturedSchema::normal(AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_EXPORT_NAME),
            },
        );
        functions.insert(
            AUTH_ZONE_DRAIN_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AuthZoneDrainInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZoneDrainOutput>(),
                export: FeaturedSchema::normal(AUTH_ZONE_DRAIN_EXPORT_NAME),
            },
        );
        functions.insert(
            AUTH_ZONE_DROP_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<AuthZoneDropInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZoneDropOutput>(),
                export: FeaturedSchema::normal(AUTH_ZONE_DROP_EXPORT_NAME),
            },
        );

        let schema = generate_full_schema(aggregator);
        let auth_zone_schema = BlueprintSchema {
            outer_blueprint: None,
            schema,
            fields,
            collections: vec![],
            functions,
            event_schema: btreemap!(),
            virtual_lazy_load_functions: btreemap!(),
            dependencies: btreeset!(),
        };

        let blueprints = btreemap!(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string() => BlueprintSetup {
                schema: fungible_resource_manager_schema,
                function_auth: btreemap!(
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string() => rule!(allow_all),
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT.to_string() => rule!(allow_all),
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT.to_string() => rule!(allow_all),
                ),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template: method_auth_template! {
                        SchemaMethodKey::metadata(METADATA_GET_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::metadata(METADATA_SET_IDENT) => [SET_METADATA_ROLE];
                        SchemaMethodKey::metadata(METADATA_REMOVE_IDENT) => [SET_METADATA_ROLE];

                        SchemaMethodKey::main(FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT) => [MINT_ROLE];
                        SchemaMethodKey::main(RESOURCE_MANAGER_BURN_IDENT) => [BURN_ROLE];
                        SchemaMethodKey::main(RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT) => SchemaMethodPermission::Public;
                    },
                    outer_method_auth_template: btreemap!(),
                }
            },
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string() => BlueprintSetup {
                schema: non_fungible_resource_manager_schema,
                function_auth: btreemap!(
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string() => rule!(allow_all),
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string() => rule!(allow_all),
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT.to_string() => rule!(allow_all),
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_UUID_WITH_INITIAL_SUPPLY_IDENT.to_string() => rule!(allow_all),
                ),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template: method_auth_template! {
                        SchemaMethodKey::metadata(METADATA_GET_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::metadata(METADATA_SET_IDENT) => [SET_METADATA_ROLE];
                        SchemaMethodKey::metadata(METADATA_REMOVE_IDENT) => [SET_METADATA_ROLE];

                        SchemaMethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT) => [MINT_ROLE];
                        SchemaMethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT) => [MINT_ROLE];
                        SchemaMethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT) => [MINT_ROLE];
                        SchemaMethodKey::main(RESOURCE_MANAGER_BURN_IDENT) => [BURN_ROLE];
                        SchemaMethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT) => [UPDATE_NON_FUNGIBLE_DATA_ROLE];
                        SchemaMethodKey::main(RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT) => SchemaMethodPermission::Public;
                    },
                    outer_method_auth_template: btreemap!(),
                }
            },
            FUNGIBLE_VAULT_BLUEPRINT.to_string() => BlueprintSetup {
                schema: fungible_vault_schema,
                function_auth: btreemap!(),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template: method_auth_template! {
                        SchemaMethodKey::main(VAULT_GET_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_CREATE_PROOF_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_CREATE_PROOF_OF_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_RECALL_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_FREEZE_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_UNFREEZE_IDENT) => SchemaMethodPermission::Public;

                        SchemaMethodKey::main(VAULT_PUT_IDENT) => SchemaMethodPermission::Public;

                        SchemaMethodKey::main(FUNGIBLE_VAULT_LOCK_FEE_IDENT) => [VAULT_WITHDRAW_ROLE];
                        SchemaMethodKey::main(VAULT_TAKE_IDENT) => [VAULT_WITHDRAW_ROLE];
                    },
                    outer_method_auth_template: method_auth_template! {
                        SchemaMethodKey::main(VAULT_GET_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_CREATE_PROOF_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_CREATE_PROOF_OF_AMOUNT_IDENT) => SchemaMethodPermission::Public;

                        SchemaMethodKey::main(VAULT_FREEZE_IDENT) => [FREEZE_ROLE];
                        SchemaMethodKey::main(VAULT_UNFREEZE_IDENT) => [UNFREEZE_ROLE];
                        SchemaMethodKey::main(VAULT_TAKE_IDENT) => [WITHDRAW_ROLE];
                        SchemaMethodKey::main(FUNGIBLE_VAULT_LOCK_FEE_IDENT) => [WITHDRAW_ROLE];
                        SchemaMethodKey::main(VAULT_RECALL_IDENT) => [RECALL_ROLE];
                        SchemaMethodKey::main(VAULT_PUT_IDENT) => [DEPOSIT_ROLE];
                        SchemaMethodKey::main(FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT) => ["this_package"];
                        SchemaMethodKey::main(FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT) => ["this_package"];
                    },
                }
            },
            NON_FUNGIBLE_VAULT_BLUEPRINT.to_string() => BlueprintSetup {
                schema: non_fungible_vault_schema,
                function_auth: btreemap!(),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template: method_auth_template! {
                        SchemaMethodKey::main(VAULT_GET_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_CREATE_PROOF_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_CREATE_PROOF_OF_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_RECALL_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_FREEZE_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_UNFREEZE_IDENT) => SchemaMethodPermission::Public;

                        SchemaMethodKey::main(VAULT_PUT_IDENT) => SchemaMethodPermission::Public;

                        SchemaMethodKey::main(VAULT_TAKE_IDENT) => [VAULT_WITHDRAW_ROLE];
                        SchemaMethodKey::main(NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT) => [VAULT_WITHDRAW_ROLE];
                    },
                    outer_method_auth_template: method_auth_template! {
                        SchemaMethodKey::main(VAULT_GET_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_CREATE_PROOF_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(VAULT_CREATE_PROOF_OF_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT) => SchemaMethodPermission::Public;

                        SchemaMethodKey::main(VAULT_TAKE_IDENT) => [WITHDRAW_ROLE];
                        SchemaMethodKey::main(NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT) => [WITHDRAW_ROLE];
                        SchemaMethodKey::main(VAULT_RECALL_IDENT) => [RECALL_ROLE];
                        SchemaMethodKey::main(VAULT_FREEZE_IDENT) => [FREEZE_ROLE];
                        SchemaMethodKey::main(VAULT_UNFREEZE_IDENT) => [UNFREEZE_ROLE];
                        SchemaMethodKey::main(NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT) => [RECALL_ROLE];
                        SchemaMethodKey::main(VAULT_PUT_IDENT) => [DEPOSIT_ROLE];
                        SchemaMethodKey::main(NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT) => ["this_package"];
                        SchemaMethodKey::main(NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT) => ["this_package"];
                    },
                }
            },
            FUNGIBLE_BUCKET_BLUEPRINT.to_string() => BlueprintSetup {
                schema: fungible_bucket_schema,
                function_auth: btreemap!(),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template: btreemap!(),
                    outer_method_auth_template: method_auth_template! {
                        SchemaMethodKey::main(BUCKET_GET_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(BUCKET_GET_RESOURCE_ADDRESS_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(BUCKET_CREATE_PROOF_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(BUCKET_CREATE_PROOF_OF_ALL_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(BUCKET_PUT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(BUCKET_TAKE_IDENT) => SchemaMethodPermission::Public;

                        SchemaMethodKey::main(FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT) => ["this_package"];
                        SchemaMethodKey::main(FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT) => ["this_package"];
                    }
                }
            },
            NON_FUNGIBLE_BUCKET_BLUEPRINT.to_string() => BlueprintSetup {
                schema: non_fungible_bucket_schema,
                function_auth: btreemap!(),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template: btreemap!(),
                    outer_method_auth_template: method_auth_template! {
                        SchemaMethodKey::main(BUCKET_GET_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(BUCKET_GET_RESOURCE_ADDRESS_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(BUCKET_CREATE_PROOF_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(BUCKET_CREATE_PROOF_OF_ALL_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(BUCKET_PUT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(BUCKET_TAKE_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT) => SchemaMethodPermission::Public;

                        SchemaMethodKey::main(NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT) => ["this_package"];
                        SchemaMethodKey::main(NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_IDENT) => ["this_package"];
                    }
                }
            },
            FUNGIBLE_PROOF_BLUEPRINT.to_string() => BlueprintSetup {
                schema: fungible_proof_schema,
                function_auth: btreemap!(
                    PROOF_DROP_IDENT.to_string() => rule!(allow_all),
                ),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template: btreemap!(),
                    outer_method_auth_template: method_auth_template!(
                        SchemaMethodKey::main(PROOF_GET_RESOURCE_ADDRESS_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(PROOF_CLONE_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(PROOF_DROP_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(PROOF_GET_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                    )
                }
            },
            NON_FUNGIBLE_PROOF_BLUEPRINT.to_string() => BlueprintSetup {
                schema: non_fungible_proof_schema,
                function_auth: btreemap!(
                    PROOF_DROP_IDENT.to_string() => rule!(allow_all),
                ),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template: btreemap!(),
                    outer_method_auth_template: method_auth_template!(
                        SchemaMethodKey::main(PROOF_GET_RESOURCE_ADDRESS_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(PROOF_CLONE_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(PROOF_DROP_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(PROOF_GET_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                        SchemaMethodKey::main(NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT) => SchemaMethodPermission::Public;
                    )
                }
            },
            WORKTOP_BLUEPRINT.to_string() => BlueprintSetup {
                schema: worktop_schema,
                function_auth: btreemap!(
                    WORKTOP_DROP_IDENT.to_string() => rule!(allow_all),
                ),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template: btreemap!(),
                    outer_method_auth_template: btreemap!(),
                },
            },
            AUTH_ZONE_BLUEPRINT.to_string() => BlueprintSetup {
                schema: auth_zone_schema,
                function_auth: btreemap!(),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template: btreemap!(),
                    outer_method_auth_template: btreemap!(),
                },
            }
        );

        PackageSetup { blueprints }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<&NodeId>,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        match export_name {
            FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: FungibleResourceManagerCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::create(
                    input.divisibility,
                    input.metadata,
                    input.access_rules,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: FungibleResourceManagerCreateWithInitialSupplyInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = FungibleResourceManagerBlueprint::create_with_initial_supply(
                    input.divisibility,
                    input.metadata,
                    input.access_rules,
                    input.initial_supply,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: FungibleResourceManagerCreateWithInitialSupplyAndAddressInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = FungibleResourceManagerBlueprint::create_with_initial_supply_and_address(
                    input.features,
                    input.divisibility,
                    input.metadata,
                    input.access_rules,
                    input.initial_supply,
                    input.resource_address,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: FungibleResourceManagerMintInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::mint(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: ResourceManagerBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::burn(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: ResourceManagerDropEmptyBucketInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::drop_empty_bucket(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerCreateEmptyVaultInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = FungibleResourceManagerBlueprint::create_empty_vault(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerCreateEmptyBucketInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;

                let rtn = FungibleResourceManagerBlueprint::create_empty_bucket(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerGetResourceTypeInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = FungibleResourceManagerBlueprint::get_resource_type(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerGetTotalSupplyInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::get_total_supply(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: NonFungibleResourceManagerCreateInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::create(
                    input.id_type,
                    input.non_fungible_schema,
                    input.metadata,
                    input.access_rules,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: NonFungibleResourceManagerCreateWithAddressInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::create_with_address(
                    input.id_type,
                    input.non_fungible_schema,
                    input.metadata,
                    input.access_rules,
                    input.resource_address,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                let input: NonFungibleResourceManagerCreateWithInitialSupplyInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::create_with_initial_supply(
                    input.id_type,
                    input.non_fungible_schema,
                    input.metadata,
                    input.access_rules,
                    input.entries,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_UUID_WITH_INITIAL_SUPPLY_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let input: NonFungibleResourceManagerCreateUuidWithInitialSupplyInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;

                let rtn = NonFungibleResourceManagerBlueprint::create_uuid_with_initial_supply(
                    input.non_fungible_schema,
                    input.metadata,
                    input.access_rules,
                    input.entries,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleResourceManagerMintInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn =
                    NonFungibleResourceManagerBlueprint::mint_non_fungible(input.entries, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleResourceManagerMintUuidInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::mint_uuid_non_fungible(
                    input.entries,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleResourceManagerMintSingleUuidInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::mint_single_uuid_non_fungible(
                    input.entry,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: ResourceManagerBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleResourceManagerBlueprint::burn(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: ResourceManagerDropEmptyBucketInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn =
                    NonFungibleResourceManagerBlueprint::drop_empty_bucket(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerCreateEmptyBucketInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;

                let rtn = NonFungibleResourceManagerBlueprint::create_empty_bucket(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerCreateEmptyVaultInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::create_empty_vault(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleResourceManagerUpdateDataInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::update_non_fungible_data(
                    input.id,
                    input.field_name,
                    input.data,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleResourceManagerExistsInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::non_fungible_exists(input.id, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerGetResourceTypeInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::get_resource_type(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerGetTotalSupplyInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleResourceManagerBlueprint::get_total_supply(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleResourceManagerGetNonFungibleInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::get_non_fungible(input.id, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            FUNGIBLE_VAULT_LOCK_FEE_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: FungibleVaultLockFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                FungibleVaultBlueprint::lock_fee(receiver, input.amount, input.contingent, api)
            }
            FUNGIBLE_VAULT_TAKE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultTakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::take(&input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_RECALL_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultRecallInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::recall(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_FREEZE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: VaultFreezeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::freeze(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: VaultUnfreezeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::unfreeze(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_PUT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultPutInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::put(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: VaultGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::get_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: VaultCreateProofInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::create_proof(receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: VaultCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn =
                    FungibleVaultBlueprint::create_proof_of_amount(receiver, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_LOCK_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: FungibleVaultLockFungibleAmountInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = FungibleVaultBlueprint::lock_amount(receiver, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_UNLOCK_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: FungibleVaultUnlockFungibleAmountInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = FungibleVaultBlueprint::unlock_amount(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            NON_FUNGIBLE_VAULT_TAKE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultTakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::take(&input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleVaultTakeNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::take_non_fungibles(
                    input.non_fungible_local_ids,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_RECALL_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultRecallInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::recall(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_FREEZE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: VaultFreezeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::freeze(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: VaultUnfreezeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::unfreeze(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleVaultRecallNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::recall_non_fungibles(
                    input.non_fungible_local_ids,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_PUT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultPutInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::put(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: VaultGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::get_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: NonFungibleVaultGetNonFungibleLocalIdsInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::get_non_fungible_local_ids(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: VaultCreateProofInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::create_proof(receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: VaultCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn =
                    NonFungibleVaultBlueprint::create_proof_of_amount(receiver, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: NonFungibleVaultCreateProofOfNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::create_proof_of_non_fungibles(
                    receiver, input.ids, api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: NonFungibleVaultLockNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn =
                    NonFungibleVaultBlueprint::lock_non_fungibles(receiver, input.local_ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleVaultUnlockNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::unlock_non_fungibles(input.local_ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            FUNGIBLE_PROOF_CLONE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let _input: ProofCloneInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleProofBlueprint::clone(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let _input: ProofGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleProofBlueprint::get_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ProofGetResourceAddressInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = FungibleProofBlueprint::get_resource_address(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_PROOF_DROP_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ProofDropInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleProofBlueprint::drop(input.proof, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_CLONE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let _input: ProofCloneInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleProofBlueprint::clone(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let _input: ProofGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleProofBlueprint::get_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let _input: NonFungibleProofGetLocalIdsInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleProofBlueprint::get_local_ids(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            NON_FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ProofGetResourceAddressInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = NonFungibleProofBlueprint::get_resource_address(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_DROP_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ProofDropInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleProofBlueprint::drop(input.proof, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            FUNGIBLE_BUCKET_PUT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                FungibleBucketBlueprint::put(input, api)
            }
            FUNGIBLE_BUCKET_TAKE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                FungibleBucketBlueprint::take(input, api)
            }
            FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: BucketGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let amount = FungibleBucketBlueprint::get_amount(api)?;

                Ok(IndexedScryptoValue::from_typed(&amount))
            }
            FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                FungibleBucketBlueprint::get_resource_address(input, api)
            }
            FUNGIBLE_BUCKET_CREATE_PROOF_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: BucketCreateProofInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleBucketBlueprint::create_proof(receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: BucketCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn =
                    FungibleBucketBlueprint::create_proof_of_amount(receiver, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: BucketCreateProofOfAllInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleBucketBlueprint::create_proof_of_all(receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_BUCKET_LOCK_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                FungibleBucketBlueprint::lock_amount(receiver, input, api)
            }
            FUNGIBLE_BUCKET_UNLOCK_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                FungibleBucketBlueprint::unlock_amount(input, api)
            }

            NON_FUNGIBLE_BUCKET_PUT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                NonFungibleBucketBlueprint::put(input, api)
            }
            NON_FUNGIBLE_BUCKET_TAKE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                NonFungibleBucketBlueprint::take(input, api)
            }
            NON_FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: BucketGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let amount = NonFungibleBucketBlueprint::get_amount(api)?;

                Ok(IndexedScryptoValue::from_typed(&amount))
            }
            NON_FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                NonFungibleBucketBlueprint::get_resource_address(input, api)
            }
            NON_FUNGIBLE_BUCKET_CREATE_PROOF_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: BucketCreateProofInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleBucketBlueprint::create_proof(receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: BucketCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleBucketBlueprint::create_proof_of_amount(
                    receiver,
                    input.amount,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: NonFungibleBucketCreateProofOfNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleBucketBlueprint::create_proof_of_non_fungibles(
                    receiver, input.ids, api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: BucketCreateProofOfAllInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleBucketBlueprint::create_proof_of_all(receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                NonFungibleBucketBlueprint::get_non_fungible_local_ids(input, api)
            }
            NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                NonFungibleBucketBlueprint::take_non_fungibles(input, api)
            }
            NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                NonFungibleBucketBlueprint::lock_non_fungibles(receiver, input, api)
            }
            NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                NonFungibleBucketBlueprint::unlock_non_fungibles(input, api)
            }

            WORKTOP_DROP_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                WorktopBlueprint::drop(input, api)
            }
            WORKTOP_PUT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                WorktopBlueprint::put(input, api)
            }
            WORKTOP_TAKE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                WorktopBlueprint::take(input, api)
            }
            WORKTOP_TAKE_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                WorktopBlueprint::take_non_fungibles(input, api)
            }
            WORKTOP_TAKE_ALL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                WorktopBlueprint::take_all(input, api)
            }
            WORKTOP_ASSERT_CONTAINS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                WorktopBlueprint::assert_contains(input, api)
            }
            WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                WorktopBlueprint::assert_contains_amount(input, api)
            }
            WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                WorktopBlueprint::assert_contains_non_fungibles(input, api)
            }
            WORKTOP_DRAIN_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                WorktopBlueprint::drain(input, api)
            }
            AUTH_ZONE_POP_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: AuthZonePopInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let proof = AuthZoneBlueprint::pop(api)?;

                Ok(IndexedScryptoValue::from_typed(&proof))
            }
            AUTH_ZONE_PUSH_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AuthZonePushInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                AuthZoneBlueprint::push(input.proof, api)?;

                Ok(IndexedScryptoValue::from_typed(&()))
            }
            AUTH_ZONE_CREATE_PROOF_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: AuthZoneCreateProofInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let proof = AuthZoneBlueprint::create_proof(input.resource_address, api)?;

                Ok(IndexedScryptoValue::from_typed(&proof))
            }
            AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: AuthZoneCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let proof = AuthZoneBlueprint::create_proof_of_amount(
                    input.resource_address,
                    input.amount,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&proof))
            }
            AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: AuthZoneCreateProofOfNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;

                let proof = AuthZoneBlueprint::create_proof_of_non_fungibles(
                    input.resource_address,
                    input.ids,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&proof))
            }
            AUTH_ZONE_CREATE_PROOF_OF_ALL_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: AuthZoneCreateProofOfAllInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let proof = AuthZoneBlueprint::create_proof_of_all(input.resource_address, api)?;

                Ok(IndexedScryptoValue::from_typed(&proof))
            }
            AUTH_ZONE_CLEAR_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let _input: AuthZoneClearInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                AuthZoneBlueprint::clear(api)?;

                Ok(IndexedScryptoValue::from_typed(&()))
            }
            AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let _input: AuthZoneClearInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                AuthZoneBlueprint::clear_signature_proofs(api)?;

                Ok(IndexedScryptoValue::from_typed(&()))
            }
            AUTH_ZONE_DRAIN_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let _input: AuthZoneDrainInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let proofs = AuthZoneBlueprint::drain(api)?;

                Ok(IndexedScryptoValue::from_typed(&proofs))
            }
            AUTH_ZONE_DROP_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                AuthZoneBlueprint::drop(input, api)
            }
            _ => Err(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
