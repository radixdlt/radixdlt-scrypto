use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::system_callback::SystemLockData;
use crate::system::system_modules::costing::{FIXED_HIGH_FEE, FIXED_LOW_FEE, FIXED_MEDIUM_FEE};
use crate::types::*;
use crate::{event_schema, method_auth_template};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::node_modules::metadata::{METADATA_GET_IDENT, METADATA_REMOVE_IDENT, METADATA_SET_IDENT, METADATA_SETTER_ROLE};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, MethodAuthTemplate, PackageDefinition,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::{
    BlueprintCollectionSchema, BlueprintSchemaInit, FieldSchema, Generic,
};
use radix_engine_interface::schema::{
    BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit, BlueprintIndexSchema,
    FunctionSchemaInit,
};
use radix_engine_interface::schema::{
    BlueprintKeyValueStoreSchema, BlueprintStateSchemaInit, TypeRef,
};
use radix_engine_interface::schema::{Receiver, ReceiverInfo, RefTypes};
use resources_tracker_macro::trace_resources;

const FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME: &str = "create_FungibleResourceManager";
const FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME: &str =
    "create_with_initial_supply_and_address_FungibleResourceManager";
const FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_EXPORT_NAME: &str =
    "create_with_initial_supply_FungibleResourceManager";
const FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME: &str = "burn_FungibleResourceManager";
const FUNGIBLE_RESOURCE_MANAGER_PACKAGE_BURN_EXPORT_NAME: &str =
    "package_burn_FungibleResourceManager";
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
const NON_FUNGIBLE_RESOURCE_MANAGER_PACKAGE_BURN_EXPORT_NAME: &str =
    "package_burn_NonFungibleResourceManager";
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
const FUNGIBLE_VAULT_BURN_EXPORT_NAME: &str = "burn_FungibleVault";

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
const NON_FUNGIBLE_VAULT_BURN_EXPORT_NAME: &str = "burn_NonFungibleVault";

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
    pub fn definition() -> PackageDefinition {
        //====================================================================================

        let fungible_resource_manager_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator
                    .add_child_type_and_descendents::<FungibleResourceManagerDivisibilitySubstate>(
                    ),
            ));
            fields.push(FieldSchema::if_feature(
                aggregator
                    .add_child_type_and_descendents::<FungibleResourceManagerTotalSupplySubstate>(),
                TRACK_TOTAL_SUPPLY_FEATURE,
            ));

            let mut functions = BTreeMap::new();
            functions.insert(
                FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<FungibleResourceManagerCreateInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<FungibleResourceManagerCreateOutput>(
                            ),
                    ),
                    export: FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateWithInitialSupplyInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateWithInitialSupplyOutput>()),
                    export: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateWithInitialSupplyAndAddressInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateWithInitialSupplyAndAddressOutput>()),
                    export: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<FungibleResourceManagerMintInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<FungibleResourceManagerMintOutput>(),
                    ),
                    export: FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_BURN_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ResourceManagerBurnInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ResourceManagerBurnOutput>(),
                    ),
                    export: FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_PACKAGE_BURN_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerPackageBurnInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerPackageBurnOutput>(),
                    ),
                    export: FUNGIBLE_RESOURCE_MANAGER_PACKAGE_BURN_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultOutput>()),
                    export: FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketOutput>()),
                    export: FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerGetResourceTypeInput>(
                            ),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerGetResourceTypeOutput>(
                            ),
                    ),
                    export: FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyOutput>(
                            ),
                    ),
                    export: FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketInput>(
                            ),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketOutput>(
                            ),
                    ),
                    export: FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME.to_string(),
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

            BlueprintDefinitionInit {
                outer_blueprint: None,
                dependencies: btreeset!(),
                feature_set: btreeset!(TRACK_TOTAL_SUPPLY_FEATURE.to_string()),
                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: event_schema,
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },
                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: btreemap!(
                        FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string() => rule!(allow_all),
                        FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT.to_string() => rule!(allow_all),
                        FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT.to_string() => rule!(allow_all),
                    ),
                    method_auth: MethodAuthTemplate::Static {
                        auth: method_auth_template! {
                            FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT => [MINT_ROLE];
                            RESOURCE_MANAGER_BURN_IDENT => [BURN_ROLE];
                            RESOURCE_MANAGER_PACKAGE_BURN_IDENT => [RESOURCE_PACKAGE_ROLE];
                            RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT => MethodPermission::Public;
                            RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT => MethodPermission::Public;
                            RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT => MethodPermission::Public;
                            RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT => MethodPermission::Public;
                            RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT => MethodPermission::Public;
                        },
                        outer_auth: btreemap!(),
                    },
                },
            }
        };

        //====================================================================================

        let non_fungible_resource_manager_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerIdTypeSubstate>(),
            ));
            fields.push(
                FieldSchema::static_field(aggregator
                    .add_child_type_and_descendents::<NonFungibleResourceManagerMutableFieldsSubstate>(
                    )),
            );
            fields.push(
                FieldSchema::if_feature(
                    aggregator.add_child_type_and_descendents::<NonFungibleResourceManagerTotalSupplySubstate>(),
                    TRACK_TOTAL_SUPPLY_FEATURE,
                )
            );

            let mut collections = Vec::new();
            collections.push(BlueprintCollectionSchema::KeyValueStore(
                BlueprintKeyValueStoreSchema {
                    key: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<NonFungibleLocalId>(),
                    ),
                    value: TypeRef::Generic(0u8),
                    can_own: false,
                },
            ));

            let mut functions = BTreeMap::new();
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateOutput>()),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateWithAddressInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateWithAddressOutput>()),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateWithInitialSupplyInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateWithInitialSupplyOutput>()),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_UUID_WITH_INITIAL_SUPPLY_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateUuidWithInitialSupplyInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateUuidWithInitialSupplyOutput>()),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_UUID_WITH_INITIAL_SUPPLY_IDENT.to_string(),
                },
            );

            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<NonFungibleResourceManagerMintInput>(
                            ),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<NonFungibleResourceManagerMintOutput>(
                            ),
                    ),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerGetNonFungibleInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerGetNonFungibleOutput>()),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT.to_string(),
                },
            );

            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerUpdateDataInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerUpdateDataOutput>()),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerExistsInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerExistsOutput>()),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT.to_string(),
                },
            );

            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintUuidInput>(
                        )),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintUuidOutput>(
                        )),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintSingleUuidInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintSingleUuidOutput>()),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT.to_string(),
                },
            );

            functions.insert(
                RESOURCE_MANAGER_PACKAGE_BURN_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerPackageBurnInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerPackageBurnOutput>(),
                    ),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_PACKAGE_BURN_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_BURN_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ResourceManagerBurnInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ResourceManagerBurnOutput>(),
                    ),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultOutput>()),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME
                        .to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketOutput>()),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME
                        .to_string(),
                },
            );

            functions.insert(
                RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerGetResourceTypeInput>(
                            ),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerGetResourceTypeOutput>(
                            ),
                    ),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyOutput>(
                            ),
                    ),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketInput>(
                            ),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketOutput>(
                            ),
                    ),
                    export: NON_FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME.to_string(),
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

            BlueprintDefinitionInit {
                outer_blueprint: None,
                dependencies: btreeset!(),
                feature_set: btreeset!(TRACK_TOTAL_SUPPLY_FEATURE.to_string()),
                schema: BlueprintSchemaInit {
                    generics: vec![Generic::Any],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections,
                    },
                    events: event_schema,
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },

                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: btreemap!(
                        NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string() => rule!(allow_all),
                        NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string() => rule!(allow_all),
                        NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT.to_string() => rule!(allow_all),
                        NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_UUID_WITH_INITIAL_SUPPLY_IDENT.to_string() => rule!(allow_all),
                    ),
                    method_auth: MethodAuthTemplate::Static {
                        auth: method_auth_template! {
                            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT => [MINT_ROLE];
                            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT => [MINT_ROLE];
                            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT => [MINT_ROLE];
                            RESOURCE_MANAGER_BURN_IDENT => [BURN_ROLE];
                            RESOURCE_MANAGER_PACKAGE_BURN_IDENT => [RESOURCE_PACKAGE_ROLE];
                            NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT => [UPDATE_NON_FUNGIBLE_DATA_ROLE];
                            RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT => MethodPermission::Public;
                            RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT => MethodPermission::Public;
                            RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT => MethodPermission::Public;
                            RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT => MethodPermission::Public;
                            RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT => MethodPermission::Public;
                            NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT => MethodPermission::Public;
                            NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT => MethodPermission::Public;
                        },
                        outer_auth: btreemap!(),
                    },
                },
            }
        };

        //====================================================================================

        let fungible_vault_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<FungibleVaultBalanceSubstate>(),
            ));
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<LockedFungibleResource>(),
            ));

            let mut functions = BTreeMap::new();
            functions.insert(
                VAULT_TAKE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultTakeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultTakeOutput>(),
                    ),
                    export: FUNGIBLE_VAULT_TAKE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_PUT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultPutInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultPutOutput>(),
                    ),
                    export: FUNGIBLE_VAULT_PUT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_GET_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultGetAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultGetAmountOutput>(),
                    ),
                    export: FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_VAULT_LOCK_FEE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<FungibleVaultLockFeeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<FungibleVaultLockFeeOutput>(),
                    ),
                    export: FUNGIBLE_VAULT_LOCK_FEE_IDENT.to_string(),
                },
            );
            functions.insert(
                VAULT_RECALL_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultRecallInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultRecallOutput>(),
                    ),
                    export: FUNGIBLE_VAULT_RECALL_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_FREEZE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultFreezeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultFreezeOutput>(),
                    ),
                    export: FUNGIBLE_VAULT_FREEZE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_UNFREEZE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultUnfreezeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultUnfreezeOutput>(),
                    ),
                    export: FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_CREATE_PROOF_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultCreateProofInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultCreateProofOutput>(),
                    ),
                    export: FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<VaultCreateProofOfAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<VaultCreateProofOfAmountOutput>(),
                    ),
                    export: FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<FungibleVaultLockFungibleAmountInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<FungibleVaultLockFungibleAmountOutput>()),
                    export: FUNGIBLE_VAULT_LOCK_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<FungibleVaultUnlockFungibleAmountInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<FungibleVaultUnlockFungibleAmountOutput>(
                        )),
                    export: FUNGIBLE_VAULT_UNLOCK_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_BURN_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultBurnInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultBurnOutput>(),
                    ),
                    export: FUNGIBLE_VAULT_BURN_EXPORT_NAME.to_string(),
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

            BlueprintDefinitionInit {
                outer_blueprint: Some(FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                dependencies: btreeset!(),
                feature_set: btreeset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: event_schema,
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },

                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: btreemap!(),

                    method_auth: MethodAuthTemplate::Static {
                        // This is the mapping from Vault methods to Vault roles
                        // NOTE: This is an extra filter on top of the ResourceManager filter
                        // This is for use with the freezing feature
                        // Any roles mentioned here have to be added as Public in create_empty_vault else you'll get spurious errors
                        auth: method_auth_template! {
                            VAULT_GET_AMOUNT_IDENT => MethodPermission::Public;
                            VAULT_CREATE_PROOF_IDENT => MethodPermission::Public;
                            VAULT_CREATE_PROOF_OF_AMOUNT_IDENT => MethodPermission::Public;
                            VAULT_RECALL_IDENT => MethodPermission::Public;
                            FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT => MethodPermission::Public;
                            FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT => MethodPermission::Public;
                            VAULT_FREEZE_IDENT => MethodPermission::Public;
                            VAULT_UNFREEZE_IDENT => MethodPermission::Public;
                            VAULT_PUT_IDENT => MethodPermission::Public;
                            VAULT_BURN_IDENT => MethodPermission::Public;
                            FUNGIBLE_VAULT_LOCK_FEE_IDENT => [VAULT_WITHDRAW_ROLE];
                            VAULT_TAKE_IDENT => [VAULT_WITHDRAW_ROLE];
                        },

                        // This is the mapping to ResourceManager roles
                        // NOTE: This is an extra filter on top of the Vault filter
                        outer_auth: method_auth_template! {
                            VAULT_GET_AMOUNT_IDENT => MethodPermission::Public;
                            VAULT_CREATE_PROOF_IDENT => MethodPermission::Public;
                            VAULT_CREATE_PROOF_OF_AMOUNT_IDENT => MethodPermission::Public;
                            VAULT_FREEZE_IDENT => [FREEZE_ROLE];
                            VAULT_UNFREEZE_IDENT => [UNFREEZE_ROLE];
                            VAULT_TAKE_IDENT => [WITHDRAW_ROLE];
                            FUNGIBLE_VAULT_LOCK_FEE_IDENT => [WITHDRAW_ROLE];
                            VAULT_RECALL_IDENT => [RECALL_ROLE];
                            VAULT_PUT_IDENT => [DEPOSIT_ROLE];
                            VAULT_BURN_IDENT => [BURN_ROLE];
                            FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT => [RESOURCE_PACKAGE_ROLE];
                            FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT => [RESOURCE_PACKAGE_ROLE];
                        },
                    },
                },
            }
        };

        //====================================================================================

        let non_fungible_vault_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<NonFungibleVaultBalanceSubstate>(),
            ));
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<LockedNonFungibleResource>(),
            ));

            let mut collections = Vec::new();
            collections.push(BlueprintCollectionSchema::Index(BlueprintIndexSchema {}));

            let mut functions = BTreeMap::new();
            functions.insert(
                VAULT_TAKE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultTakeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultTakeOutput>(),
                    ),
                    export: NON_FUNGIBLE_VAULT_TAKE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultTakeNonFungiblesInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultTakeNonFungiblesOutput>()),
                    export: NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT.to_string(),
                },
            );
            functions.insert(
                VAULT_RECALL_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultRecallInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultRecallOutput>(),
                    ),
                    export: NON_FUNGIBLE_VAULT_RECALL_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_FREEZE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultFreezeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultFreezeOutput>(),
                    ),
                    export: NON_FUNGIBLE_VAULT_FREEZE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_UNFREEZE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultUnfreezeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultUnfreezeOutput>(),
                    ),
                    export: NON_FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo {
                        receiver: Receiver::SelfRefMut,
                        ref_types: RefTypes::DIRECT_ACCESS,
                    }),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultRecallNonFungiblesInput>(
                        )),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultRecallNonFungiblesOutput>(
                        )),
                    export: NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT.to_string(),
                },
            );
            functions.insert(
                VAULT_PUT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultPutInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultPutOutput>(),
                    ),
                    export: NON_FUNGIBLE_VAULT_PUT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_GET_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultGetAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultGetAmountOutput>(),
                    ),
                    export: NON_FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultGetNonFungibleLocalIdsInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultGetNonFungibleLocalIdsOutput>()),
                    export: NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
                },
            );
            functions.insert(
                VAULT_CREATE_PROOF_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultCreateProofInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultCreateProofOutput>(),
                    ),
                    export: NON_FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<VaultCreateProofOfAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<VaultCreateProofOfAmountOutput>(),
                    ),
                    export: NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultCreateProofOfNonFungiblesInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultCreateProofOfNonFungiblesOutput>()),
                    export: NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultLockNonFungiblesInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultLockNonFungiblesOutput>()),
                    export: NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultUnlockNonFungiblesInput>(
                        )),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultUnlockNonFungiblesOutput>(
                        )),
                    export: NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_BURN_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultBurnInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<VaultBurnOutput>(),
                    ),
                    export: NON_FUNGIBLE_VAULT_BURN_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultBurnNonFungiblesInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultBurnNonFungiblesOutput>()),
                    export: NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT.to_string(),
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

            BlueprintDefinitionInit {
                outer_blueprint: Some(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                dependencies: btreeset!(),
                feature_set: btreeset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections,
                    },
                    events: event_schema,
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },

                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: btreemap!(),
                    method_auth: MethodAuthTemplate::Static {
                        // This is the mapping from Vault methods to Vault roles
                        // NOTE: This is an extra filter on top of the ResourceManager filter
                        // This is for use with the freezing feature
                        // Any roles mentioned here have to be added as Public in create_empty_vault else you'll get spurious errors
                        auth: method_auth_template! {
                            VAULT_GET_AMOUNT_IDENT => MethodPermission::Public;
                            VAULT_CREATE_PROOF_IDENT => MethodPermission::Public;
                            VAULT_CREATE_PROOF_OF_AMOUNT_IDENT => MethodPermission::Public;
                            NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT => MethodPermission::Public;
                            VAULT_RECALL_IDENT => MethodPermission::Public;
                            NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT => MethodPermission::Public;
                            NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => MethodPermission::Public;
                            NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT => MethodPermission::Public;
                            NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT => MethodPermission::Public;
                            VAULT_FREEZE_IDENT => MethodPermission::Public;
                            VAULT_UNFREEZE_IDENT => MethodPermission::Public;
                            VAULT_PUT_IDENT => MethodPermission::Public;
                            VAULT_BURN_IDENT => MethodPermission::Public;
                            NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT => MethodPermission::Public;
                            VAULT_TAKE_IDENT => [VAULT_WITHDRAW_ROLE];
                            NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT => [VAULT_WITHDRAW_ROLE];
                        },
                        // This is the mapping to ResourceManager roles
                        // NOTE: This is an extra filter on top of the Vault filter
                        outer_auth: method_auth_template! {
                            VAULT_GET_AMOUNT_IDENT => MethodPermission::Public;
                            NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => MethodPermission::Public;
                            VAULT_CREATE_PROOF_IDENT => MethodPermission::Public;
                            VAULT_CREATE_PROOF_OF_AMOUNT_IDENT => MethodPermission::Public;
                            NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT => MethodPermission::Public;

                            VAULT_TAKE_IDENT => [WITHDRAW_ROLE];
                            NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT => [WITHDRAW_ROLE];
                            VAULT_RECALL_IDENT => [RECALL_ROLE];
                            VAULT_FREEZE_IDENT => [FREEZE_ROLE];
                            VAULT_UNFREEZE_IDENT => [UNFREEZE_ROLE];
                            NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT => [RECALL_ROLE];
                            VAULT_PUT_IDENT => [DEPOSIT_ROLE];
                            VAULT_BURN_IDENT => [BURN_ROLE];
                            NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT => [BURN_ROLE];
                            NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT => [RESOURCE_PACKAGE_ROLE];
                            NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT => [RESOURCE_PACKAGE_ROLE];
                        },
                    },
                },
            }
        };

        //====================================================================================

        let fungible_bucket_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<LiquidFungibleResource>(),
            ));
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<LockedFungibleResource>(),
            ));

            let mut functions = BTreeMap::new();
            functions.insert(
                BUCKET_PUT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketPutInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketPutOutput>(),
                    ),
                    export: FUNGIBLE_BUCKET_PUT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_TAKE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketTakeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketTakeOutput>(),
                    ),
                    export: FUNGIBLE_BUCKET_TAKE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_GET_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketGetAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketGetAmountOutput>(),
                    ),
                    export: FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_GET_RESOURCE_ADDRESS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<BucketGetResourceAddressInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<BucketGetResourceAddressOutput>(),
                    ),
                    export: FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketCreateProofInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketCreateProofOutput>(),
                    ),
                    export: FUNGIBLE_BUCKET_CREATE_PROOF_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<BucketCreateProofOfAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<BucketCreateProofOfAmountOutput>(),
                    ),
                    export: FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_OF_ALL_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketCreateProofOfAllInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketCreateProofOfAllOutput>(),
                    ),
                    export: FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<FungibleBucketLockAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<FungibleBucketLockAmountOutput>(),
                    ),
                    export: FUNGIBLE_BUCKET_LOCK_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<FungibleBucketUnlockAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<FungibleBucketUnlockAmountOutput>(),
                    ),
                    export: FUNGIBLE_BUCKET_UNLOCK_AMOUNT_EXPORT_NAME.to_string(),
                },
            );

            let schema = generate_full_schema(aggregator);

            BlueprintDefinitionInit {
                outer_blueprint: Some(FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                dependencies: btreeset!(),
                feature_set: btreeset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: BlueprintEventSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },

                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: btreemap!(),
                    method_auth: MethodAuthTemplate::Static {
                        auth: btreemap!(),
                        outer_auth: method_auth_template! {
                            BUCKET_GET_AMOUNT_IDENT => MethodPermission::Public;
                            BUCKET_GET_RESOURCE_ADDRESS_IDENT => MethodPermission::Public;
                            BUCKET_CREATE_PROOF_IDENT => MethodPermission::Public;
                            BUCKET_CREATE_PROOF_OF_ALL_IDENT => MethodPermission::Public;
                            BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT => MethodPermission::Public;
                            BUCKET_PUT_IDENT => MethodPermission::Public;
                            BUCKET_TAKE_IDENT => MethodPermission::Public;

                            FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT => [RESOURCE_PACKAGE_ROLE];
                            FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT => [RESOURCE_PACKAGE_ROLE];
                        },
                    },
                },
            }
        };

        //====================================================================================

        let non_fungible_bucket_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<LiquidNonFungibleResource>(),
            ));
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<LockedNonFungibleResource>(),
            ));

            let mut functions = BTreeMap::new();
            functions.insert(
                BUCKET_PUT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketPutInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketPutOutput>(),
                    ),
                    export: NON_FUNGIBLE_BUCKET_PUT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_TAKE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketTakeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketTakeOutput>(),
                    ),
                    export: NON_FUNGIBLE_BUCKET_TAKE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                BUCKET_GET_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketGetAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketGetAmountOutput>(),
                    ),
                    export: NON_FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_GET_RESOURCE_ADDRESS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<BucketGetResourceAddressInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<BucketGetResourceAddressOutput>(),
                    ),
                    export: NON_FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketCreateProofInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketCreateProofOutput>(),
                    ),
                    export: NON_FUNGIBLE_BUCKET_CREATE_PROOF_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<BucketCreateProofOfAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<BucketCreateProofOfAmountOutput>(),
                    ),
                    export: NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator.add_child_type_and_descendents::<NonFungibleBucketCreateProofOfNonFungiblesInput>()),
                    output: TypeRef::Static(aggregator.add_child_type_and_descendents::<NonFungibleBucketCreateProofOfNonFungiblesOutput>()),
                    export: NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_OF_ALL_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketCreateProofOfAllInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketCreateProofOfAllOutput>(),
                    ),
                    export: NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketTakeNonFungiblesInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketTakeNonFungiblesOutput>(),
                    ),
                    export: NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<BucketGetNonFungibleLocalIdsInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<BucketGetNonFungibleLocalIdsOutput>(),
                    ),
                    export: NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleBucketLockNonFungiblesInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleBucketLockNonFungiblesOutput>(
                        )),
                    export: NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleBucketUnlockNonFungiblesInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<NonFungibleBucketUnlockNonFungiblesOutput>()),
                    export: NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_EXPORT_NAME.to_string(),
                },
            );

            let schema = generate_full_schema(aggregator);

            BlueprintDefinitionInit {
                outer_blueprint: Some(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                dependencies: btreeset!(),
                feature_set: btreeset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: BlueprintEventSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },

                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: btreemap!(),
                    method_auth: MethodAuthTemplate::Static {
                        auth: btreemap!(),
                        outer_auth: method_auth_template! {
                            BUCKET_GET_AMOUNT_IDENT => MethodPermission::Public;
                            BUCKET_GET_RESOURCE_ADDRESS_IDENT => MethodPermission::Public;
                            BUCKET_CREATE_PROOF_IDENT => MethodPermission::Public;
                            BUCKET_CREATE_PROOF_OF_ALL_IDENT => MethodPermission::Public;
                            BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT => MethodPermission::Public;
                            BUCKET_PUT_IDENT => MethodPermission::Public;
                            BUCKET_TAKE_IDENT => MethodPermission::Public;
                            NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => MethodPermission::Public;
                            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT => MethodPermission::Public;

                            NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT => [RESOURCE_PACKAGE_ROLE];
                            NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_IDENT => [RESOURCE_PACKAGE_ROLE];
                        },
                    },
                },
            }
        };

        let fungible_proof_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<ProofMoveableSubstate>(),
            ));
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<FungibleProofSubstate>(),
            ));

            let mut functions = BTreeMap::new();
            functions.insert(
                PROOF_DROP_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofDropInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofDropOutput>(),
                    ),
                    export: FUNGIBLE_PROOF_DROP_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                PROOF_CLONE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofCloneInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofCloneOutput>(),
                    ),
                    export: FUNGIBLE_PROOF_CLONE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                PROOF_GET_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofGetAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofGetAmountOutput>(),
                    ),
                    export: FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                PROOF_GET_RESOURCE_ADDRESS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofGetResourceAddressInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ProofGetResourceAddressOutput>(),
                    ),
                    export: FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME.to_string(),
                },
            );

            let schema = generate_full_schema(aggregator);

            BlueprintDefinitionInit {
                outer_blueprint: Some(FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                dependencies: btreeset!(),
                feature_set: btreeset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: BlueprintEventSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },

                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: btreemap!(
                        PROOF_DROP_IDENT.to_string() => rule!(allow_all),
                    ),
                    method_auth: MethodAuthTemplate::Static {
                        auth: btreemap!(),
                        outer_auth: method_auth_template!(
                            PROOF_GET_RESOURCE_ADDRESS_IDENT => MethodPermission::Public;
                            PROOF_CLONE_IDENT => MethodPermission::Public;
                            PROOF_DROP_IDENT => MethodPermission::Public;
                            PROOF_GET_AMOUNT_IDENT => MethodPermission::Public;
                        ),
                    },
                },
            }
        };

        let non_fungible_proof_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<ProofMoveableSubstate>(),
            ));
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<NonFungibleProofSubstate>(),
            ));

            let mut functions = BTreeMap::new();
            functions.insert(
                PROOF_DROP_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofDropInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofDropOutput>(),
                    ),
                    export: NON_FUNGIBLE_PROOF_DROP_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                PROOF_CLONE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofCloneInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofCloneOutput>(),
                    ),
                    export: NON_FUNGIBLE_PROOF_CLONE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                PROOF_GET_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofGetAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofGetAmountOutput>(),
                    ),
                    export: NON_FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                PROOF_GET_RESOURCE_ADDRESS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<ProofGetResourceAddressInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<ProofGetResourceAddressOutput>(),
                    ),
                    export: NON_FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<NonFungibleProofGetLocalIdsInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<NonFungibleProofGetLocalIdsOutput>(),
                    ),
                    export: NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT.to_string(),
                },
            );

            let schema = generate_full_schema(aggregator);

            BlueprintDefinitionInit {
                outer_blueprint: Some(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                dependencies: btreeset!(),
                feature_set: btreeset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: BlueprintEventSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },

                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: btreemap!(
                        PROOF_DROP_IDENT.to_string() => rule!(allow_all),
                    ),
                    method_auth: MethodAuthTemplate::Static {
                        auth: btreemap!(),
                        outer_auth: method_auth_template!(
                            PROOF_GET_RESOURCE_ADDRESS_IDENT => MethodPermission::Public;
                            PROOF_CLONE_IDENT => MethodPermission::Public;
                            PROOF_DROP_IDENT => MethodPermission::Public;
                            PROOF_GET_AMOUNT_IDENT => MethodPermission::Public;
                            NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT => MethodPermission::Public;
                        ),
                    },
                },
            }
        };

        let worktop_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<WorktopSubstate>(),
            ));

            let mut functions = BTreeMap::new();
            functions.insert(
                WORKTOP_DROP_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<WorktopDropInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<WorktopDropOutput>(),
                    ),
                    export: WORKTOP_DROP_IDENT.to_string(),
                },
            );
            functions.insert(
                WORKTOP_PUT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<WorktopPutInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<WorktopPutOutput>(),
                    ),
                    export: WORKTOP_PUT_IDENT.to_string(),
                },
            );
            functions.insert(
                WORKTOP_TAKE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<WorktopTakeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<WorktopTakeOutput>(),
                    ),
                    export: WORKTOP_TAKE_IDENT.to_string(),
                },
            );
            functions.insert(
                WORKTOP_TAKE_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<WorktopTakeNonFungiblesInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<WorktopTakeNonFungiblesOutput>(),
                    ),
                    export: WORKTOP_TAKE_NON_FUNGIBLES_IDENT.to_string(),
                },
            );
            functions.insert(
                WORKTOP_TAKE_ALL_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<WorktopTakeAllInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<WorktopTakeAllOutput>(),
                    ),
                    export: WORKTOP_TAKE_ALL_IDENT.to_string(),
                },
            );
            functions.insert(
                WORKTOP_ASSERT_CONTAINS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<WorktopAssertContainsInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<WorktopAssertContainsOutput>(),
                    ),
                    export: WORKTOP_ASSERT_CONTAINS_IDENT.to_string(),
                },
            );
            functions.insert(
                WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<WorktopAssertContainsAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<WorktopAssertContainsAmountOutput>(),
                    ),
                    export: WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT.to_string(),
                },
            );
            functions.insert(
                WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<WorktopAssertContainsNonFungiblesInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<WorktopAssertContainsNonFungiblesOutput>(
                        )),
                    export: WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT.to_string(),
                },
            );
            functions.insert(
                WORKTOP_DRAIN_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<WorktopDrainInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<WorktopDrainOutput>(),
                    ),
                    export: WORKTOP_DRAIN_IDENT.to_string(),
                },
            );
            let schema = generate_full_schema(aggregator);

            BlueprintDefinitionInit {
                outer_blueprint: None,
                dependencies: btreeset!(),
                feature_set: btreeset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: BlueprintEventSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },

                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: btreemap!(
                        WORKTOP_DROP_IDENT.to_string() => rule!(allow_all),
                    ),
                    method_auth: MethodAuthTemplate::Static {
                        auth: btreemap!(),
                        outer_auth: btreemap!(),
                    },
                },
            }
        };

        let auth_zone_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<AuthZone>(),
            ));

            let mut functions = BTreeMap::new();
            functions.insert(
                AUTH_ZONE_POP_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZonePopInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZonePopOutput>(),
                    ),
                    export: AUTH_ZONE_POP_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                AUTH_ZONE_PUSH_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZonePushInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZonePushOutput>(),
                    ),
                    export: AUTH_ZONE_PUSH_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                AUTH_ZONE_CREATE_PROOF_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZoneCreateProofInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZoneCreateProofOutput>(),
                    ),
                    export: AUTH_ZONE_CREATE_PROOF_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<AuthZoneCreateProofOfAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<AuthZoneCreateProofOfAmountOutput>(),
                    ),
                    export: AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<AuthZoneCreateProofOfNonFungiblesInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<AuthZoneCreateProofOfNonFungiblesOutput>(
                        )),
                    export: AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                AUTH_ZONE_CREATE_PROOF_OF_ALL_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<AuthZoneCreateProofOfAllInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<AuthZoneCreateProofOfAllOutput>(),
                    ),
                    export: AUTH_ZONE_CREATE_PROOF_OF_ALL_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                AUTH_ZONE_CLEAR_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZoneClearInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZoneClearOutput>(),
                    ),
                    export: AUTH_ZONE_CLEAR_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<AuthZoneClearVirtualProofsInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<AuthZoneClearVirtualProofsOutput>(),
                    ),
                    export: AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                AUTH_ZONE_DRAIN_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZoneDrainInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZoneDrainOutput>(),
                    ),
                    export: AUTH_ZONE_DRAIN_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                AUTH_ZONE_DROP_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZoneDropInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZoneDropOutput>(),
                    ),
                    export: AUTH_ZONE_DROP_EXPORT_NAME.to_string(),
                },
            );

            let schema = generate_full_schema(aggregator);
            let auth_zone_blueprint = BlueprintStateSchemaInit {
                fields,
                collections: vec![],
            };

            BlueprintDefinitionInit {
                outer_blueprint: None,
                dependencies: btreeset!(),
                feature_set: btreeset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: auth_zone_blueprint,
                    events: BlueprintEventSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },

                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: btreemap!(),
                    method_auth: MethodAuthTemplate::Static {
                        auth: btreemap!(),
                        outer_auth: btreemap!(),
                    },
                },
            }
        };

        let blueprints = btreemap!(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string() => fungible_resource_manager_blueprint,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string() => non_fungible_resource_manager_blueprint,
            FUNGIBLE_VAULT_BLUEPRINT.to_string() => fungible_vault_blueprint,
            NON_FUNGIBLE_VAULT_BLUEPRINT.to_string() => non_fungible_vault_blueprint,
            FUNGIBLE_BUCKET_BLUEPRINT.to_string() => fungible_bucket_blueprint,
            NON_FUNGIBLE_BUCKET_BLUEPRINT.to_string() => non_fungible_bucket_blueprint,
            FUNGIBLE_PROOF_BLUEPRINT.to_string() => fungible_proof_blueprint,
            NON_FUNGIBLE_PROOF_BLUEPRINT.to_string() => non_fungible_proof_blueprint,
            WORKTOP_BLUEPRINT.to_string() => worktop_blueprint,
            AUTH_ZONE_BLUEPRINT.to_string() => auth_zone_blueprint,
        );

        PackageDefinition { blueprints }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        match export_name {
            FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: FungibleResourceManagerCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::create(
                    input.track_total_supply,
                    input.divisibility,
                    input.metadata,
                    input.access_rules,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: FungibleResourceManagerCreateWithInitialSupplyInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleResourceManagerBlueprint::create_with_initial_supply(
                    input.track_total_supply,
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

                let input: FungibleResourceManagerCreateWithInitialSupplyAndAddressInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleResourceManagerBlueprint::create_with_initial_supply_and_address(
                    input.track_total_supply,
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
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::mint(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: ResourceManagerBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::burn(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_PACKAGE_BURN_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: ResourceManagerPackageBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::package_burn(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: ResourceManagerDropEmptyBucketInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::drop_empty_bucket(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerCreateEmptyVaultInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleResourceManagerBlueprint::create_empty_vault(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerCreateEmptyBucketInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;

                let rtn = FungibleResourceManagerBlueprint::create_empty_bucket(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerGetResourceTypeInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleResourceManagerBlueprint::get_resource_type(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerGetTotalSupplyInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::get_total_supply(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleResourceManagerCreateInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::create(
                    input.id_type,
                    input.track_total_supply,
                    input.non_fungible_schema,
                    input.metadata,
                    input.access_rules,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleResourceManagerCreateWithAddressInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::create_with_address(
                    input.id_type,
                    input.track_total_supply,
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

                let input: NonFungibleResourceManagerCreateWithInitialSupplyInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::create_with_initial_supply(
                    input.id_type,
                    input.track_total_supply,
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

                let input: NonFungibleResourceManagerCreateUuidWithInitialSupplyInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;

                let rtn = NonFungibleResourceManagerBlueprint::create_uuid_with_initial_supply(
                    input.track_total_supply,
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
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    NonFungibleResourceManagerBlueprint::mint_non_fungible(input.entries, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleResourceManagerMintUuidInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
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
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
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
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleResourceManagerBlueprint::burn(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_PACKAGE_BURN_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: ResourceManagerPackageBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleResourceManagerBlueprint::package_burn(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: ResourceManagerDropEmptyBucketInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    NonFungibleResourceManagerBlueprint::drop_empty_bucket(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerCreateEmptyBucketInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;

                let rtn = NonFungibleResourceManagerBlueprint::create_empty_bucket(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerCreateEmptyVaultInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::create_empty_vault(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleResourceManagerUpdateDataInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
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
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::non_fungible_exists(input.id, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerGetResourceTypeInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::get_resource_type(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerGetTotalSupplyInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleResourceManagerBlueprint::get_total_supply(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleResourceManagerGetNonFungibleInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::get_non_fungible(input.id, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            FUNGIBLE_VAULT_LOCK_FEE_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let receiver = Runtime::get_node_id(api)?;
                let input: FungibleVaultLockFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                FungibleVaultBlueprint::lock_fee(&receiver, input.amount, input.contingent, api)
            }
            FUNGIBLE_VAULT_TAKE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultTakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::take(&input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_RECALL_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultRecallInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::recall(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_FREEZE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: VaultFreezeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::freeze(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: VaultUnfreezeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::unfreeze(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_PUT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultPutInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::put(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: VaultGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::get_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = Runtime::get_node_id(api)?;
                let _input: VaultCreateProofInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::create_proof(&receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = Runtime::get_node_id(api)?;
                let input: VaultCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    FungibleVaultBlueprint::create_proof_of_amount(&receiver, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_LOCK_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = Runtime::get_node_id(api)?;
                let input: FungibleVaultLockFungibleAmountInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleVaultBlueprint::lock_amount(&receiver, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_UNLOCK_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: FungibleVaultUnlockFungibleAmountInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleVaultBlueprint::unlock_amount(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_BURN_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::burn(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            NON_FUNGIBLE_VAULT_TAKE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultTakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::take(&input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleVaultTakeNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
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
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::recall(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_FREEZE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: VaultFreezeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::freeze(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: VaultUnfreezeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::unfreeze(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleVaultRecallNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
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
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::put(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: VaultGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::get_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: NonFungibleVaultGetNonFungibleLocalIdsInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::get_non_fungible_local_ids(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = Runtime::get_node_id(api)?;
                let _input: VaultCreateProofInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::create_proof(&receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = Runtime::get_node_id(api)?;
                let input: VaultCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::create_proof_of_amount(
                    &receiver,
                    input.amount,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = Runtime::get_node_id(api)?;
                let input: NonFungibleVaultCreateProofOfNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::create_proof_of_non_fungibles(
                    &receiver, input.ids, api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = Runtime::get_node_id(api)?;
                let input: NonFungibleVaultLockNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn =
                    NonFungibleVaultBlueprint::lock_non_fungibles(&receiver, input.local_ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleVaultUnlockNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::unlock_non_fungibles(input.local_ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_BURN_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::burn(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleVaultBurnNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::burn_non_fungibles(
                    input.non_fungible_local_ids,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            FUNGIBLE_PROOF_CLONE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let _input: ProofCloneInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleProofBlueprint::clone(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let _input: ProofGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleProofBlueprint::get_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ProofGetResourceAddressInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = FungibleProofBlueprint::get_resource_address(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_PROOF_DROP_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ProofDropInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleProofBlueprint::drop(input.proof, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_CLONE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let _input: ProofCloneInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleProofBlueprint::clone(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let _input: ProofGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleProofBlueprint::get_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let _input: NonFungibleProofGetLocalIdsInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleProofBlueprint::get_local_ids(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            NON_FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: ProofGetResourceAddressInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = NonFungibleProofBlueprint::get_resource_address(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_DROP_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: ProofDropInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
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
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
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
                let receiver = Runtime::get_node_id(api)?;
                let _input: BucketCreateProofInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleBucketBlueprint::create_proof(&receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let receiver = Runtime::get_node_id(api)?;
                let input: BucketCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    FungibleBucketBlueprint::create_proof_of_amount(&receiver, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;
                let receiver = Runtime::get_node_id(api)?;
                let _input: BucketCreateProofOfAllInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleBucketBlueprint::create_proof_of_all(&receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_BUCKET_LOCK_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = Runtime::get_node_id(api)?;
                FungibleBucketBlueprint::lock_amount(&receiver, input, api)
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
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
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

                let receiver = Runtime::get_node_id(api)?;
                let _input: BucketCreateProofInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleBucketBlueprint::create_proof(&receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = Runtime::get_node_id(api)?;
                let input: BucketCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleBucketBlueprint::create_proof_of_amount(
                    &receiver,
                    input.amount,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = Runtime::get_node_id(api)?;
                let input: NonFungibleBucketCreateProofOfNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleBucketBlueprint::create_proof_of_non_fungibles(
                    &receiver, input.ids, api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = Runtime::get_node_id(api)?;
                let _input: BucketCreateProofOfAllInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleBucketBlueprint::create_proof_of_all(&receiver, api)?;
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

                let receiver = Runtime::get_node_id(api)?;
                NonFungibleBucketBlueprint::lock_non_fungibles(&receiver, input, api)
            }
            NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                NonFungibleBucketBlueprint::unlock_non_fungibles(input, api)
            }

            WORKTOP_DROP_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

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
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let proof = AuthZoneBlueprint::pop(api)?;

                Ok(IndexedScryptoValue::from_typed(&proof))
            }
            AUTH_ZONE_PUSH_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AuthZonePushInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                AuthZoneBlueprint::push(input.proof, api)?;

                Ok(IndexedScryptoValue::from_typed(&()))
            }
            AUTH_ZONE_CREATE_PROOF_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: AuthZoneCreateProofInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let proof = AuthZoneBlueprint::create_proof(input.resource_address, api)?;

                Ok(IndexedScryptoValue::from_typed(&proof))
            }
            AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let input: AuthZoneCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
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
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
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
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let proof = AuthZoneBlueprint::create_proof_of_all(input.resource_address, api)?;

                Ok(IndexedScryptoValue::from_typed(&proof))
            }
            AUTH_ZONE_CLEAR_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let _input: AuthZoneClearInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                AuthZoneBlueprint::clear(api)?;

                Ok(IndexedScryptoValue::from_typed(&()))
            }
            AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let _input: AuthZoneClearInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                AuthZoneBlueprint::clear_signature_proofs(api)?;

                Ok(IndexedScryptoValue::from_typed(&()))
            }
            AUTH_ZONE_DRAIN_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let _input: AuthZoneDrainInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let proofs = AuthZoneBlueprint::drain(api)?;

                Ok(IndexedScryptoValue::from_typed(&proofs))
            }
            AUTH_ZONE_DROP_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                AuthZoneBlueprint::drop(input, api)
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
