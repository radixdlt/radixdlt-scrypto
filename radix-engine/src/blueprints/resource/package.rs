use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::event_schema;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::system_modules::costing::{FIXED_HIGH_FEE, FIXED_LOW_FEE, FIXED_MEDIUM_FEE};
use crate::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::BlueprintSchema;
use radix_engine_interface::schema::FunctionSchema;
use radix_engine_interface::schema::PackageSchema;
use radix_engine_interface::schema::Receiver;
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
const FUNGIBLE_RESOURCE_MANAGER_CREATE_BUCKET_EXPORT_NAME: &str =
    "create_bucket_FungibleResourceManager";
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
const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_BUCKET_EXPORT_NAME: &str =
    "create_bucket_NonFungibleResourceManager";
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
const FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME: &str = "create_proof_FungibleVault";
const FUNGIBLE_VAULT_CREATE_PROOF_BY_AMOUNT_EXPORT_NAME: &str =
    "create_proof_by_amount_FungibleVault";
const FUNGIBLE_VAULT_LOCK_AMOUNT_EXPORT_NAME: &str = "lock_amount_FungibleVault";
const FUNGIBLE_VAULT_UNLOCK_AMOUNT_EXPORT_NAME: &str = "unlock_amount_FungibleVault";

const NON_FUNGIBLE_VAULT_TAKE_EXPORT_NAME: &str = "take_NonFungibleVault";
const NON_FUNGIBLE_VAULT_PUT_EXPORT_NAME: &str = "put_NonFungibleVault";
const NON_FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_NonFungibleVault";
const NON_FUNGIBLE_VAULT_RECALL_EXPORT_NAME: &str = "recall_NonFungibleVault";
const NON_FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME: &str = "create_proof_NonFungibleVault";
const NON_FUNGIBLE_VAULT_CREATE_PROOF_BY_AMOUNT_EXPORT_NAME: &str =
    "create_proof_by_amount_NonFungibleVault";
const NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_EXPORT_NAME: &str = "unlock_fungibles_NonFungibleVault";
const NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_EXPORT_NAME: &str =
    "unlock_non_fungibles_NonFungibleVault";

const FUNGIBLE_BUCKET_TAKE_EXPORT_NAME: &str = "take_FungibleBucket";
const FUNGIBLE_BUCKET_PUT_EXPORT_NAME: &str = "put_FungibleBucket";
const FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_FungibleBucket";
const FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME: &str =
    "get_resource_address_FungibleBucket";
const FUNGIBLE_BUCKET_CREATE_PROOF_EXPORT_NAME: &str = "create_proof_FungibleBucket";
const FUNGIBLE_BUCKET_LOCK_AMOUNT_EXPORT_NAME: &str = "lock_amount_FungibleBucket";
const FUNGIBLE_BUCKET_UNLOCK_AMOUNT_EXPORT_NAME: &str = "unlock_amount_FungibleBucket";

const NON_FUNGIBLE_BUCKET_TAKE_EXPORT_NAME: &str = "take_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_PUT_EXPORT_NAME: &str = "put_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME: &str =
    "get_resource_address_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_CREATE_PROOF_EXPORT_NAME: &str = "create_proof_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_EXPORT_NAME: &str =
    "unlock_fungibles_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_EXPORT_NAME: &str =
    "unlock_non_fungibles_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_EXPORT_NAME: &str =
    "take_non_fungibles_NonFungibleBucket";
const NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_EXPORT_NAME: &str =
    "get_non_fungible_local_ids_NonFungibleBucket";

pub struct ResourceManagerNativePackage;

impl ResourceManagerNativePackage {
    pub fn schema() -> PackageSchema {
        //====================================================================================

        let fungible_resource_manager_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut substates = Vec::new();
            substates.push(
                aggregator.add_child_type_and_descendents::<FungibleResourceManagerSubstate>(),
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
                    export_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME.to_string(),
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
                    export_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME.to_string(),
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
                    export_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerMintInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerMintOutput>(),
                    export_name: FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_BURN_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<ResourceManagerBurnInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerBurnOutput>(),
                    export_name: FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultOutput>(),
                    export_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME
                        .to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketOutput>(),
                    export_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME
                        .to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_RESOURCE_MANAGER_CREATE_BUCKET_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateBucketInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleResourceManagerCreateBucketOutput>(),
                    export_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_BUCKET_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetResourceTypeInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetResourceTypeOutput>(),
                    export_name: FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME
                        .to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyOutput>(),
                    export_name: FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketOutput>(),
                    export_name: FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME
                        .to_string(),
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
                parent: None,
                schema,
                substates,
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema,
            }
        };

        //====================================================================================

        let non_fungible_resource_manager_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut substates = Vec::new();
            substates.push(
                aggregator.add_child_type_and_descendents::<NonFungibleResourceManagerSubstate>(),
            );

            let mut functions = BTreeMap::new();
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                FunctionSchema {
                    receiver: None,
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateOutput>(),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME.to_string(),
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
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
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
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME.to_string(),
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
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_UUID_WITH_INITIAL_SUPPLY_IDENT.to_string(),
                },
            );

            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintOutput>(),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerGetNonFungibleInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerGetNonFungibleOutput>(),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT.to_string(),
                },
            );

            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerUpdateDataInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerUpdateDataOutput>(),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerExistsInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerExistsOutput>(),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT.to_string(),
                },
            );

            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintUuidInput>(
                        ),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintUuidOutput>(
                        ),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintSingleUuidInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerMintSingleUuidOutput>(),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT.to_string(),
                },
            );

            functions.insert(
                RESOURCE_MANAGER_BURN_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<ResourceManagerBurnInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerBurnOutput>(),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyVaultOutput>(),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME
                        .to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerCreateEmptyBucketOutput>(),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME
                        .to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_RESOURCE_MANAGER_CREATE_BUCKET_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateBucketInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleResourceManagerCreateBucketOutput>(),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_BUCKET_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetResourceTypeInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetResourceTypeOutput>(),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME
                        .to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerGetTotalSupplyOutput>(),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME
                        .to_string(),
                },
            );
            functions.insert(
                RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator
                        .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<ResourceManagerDropEmptyBucketOutput>(),
                    export_name: NON_FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME
                        .to_string(),
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
                parent: None,
                schema,
                substates,
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema,
            }
        };

        //====================================================================================

        let fungible_vault_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
            let mut substates = Vec::new();
            substates.push(
                aggregator.add_child_type_and_descendents::<FungibleVaultDivisibilitySubstate>(),
            );
            substates
                .push(aggregator.add_child_type_and_descendents::<FungibleVaultBalanceSubstate>());
            substates.push(aggregator.add_child_type_and_descendents::<LockedFungibleResource>());

            let mut functions = BTreeMap::new();
            functions.insert(
                VAULT_TAKE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<VaultTakeInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultTakeOutput>(),
                    export_name: FUNGIBLE_VAULT_TAKE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_PUT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<VaultPutInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultPutOutput>(),
                    export_name: FUNGIBLE_VAULT_PUT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_GET_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator.add_child_type_and_descendents::<VaultGetAmountInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultGetAmountOutput>(),
                    export_name: FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_VAULT_LOCK_FEE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<FungibleVaultLockFeeInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleVaultLockFeeOutput>(),
                    export_name: FUNGIBLE_VAULT_LOCK_FEE_IDENT.to_string(),
                },
            );
            functions.insert(
                VAULT_RECALL_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<VaultRecallInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultRecallOutput>(),
                    export_name: FUNGIBLE_VAULT_RECALL_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_CREATE_PROOF_OF_ALL_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<VaultCreateProofOfAllInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<VaultCreateProofOfAllOutput>(),
                    export_name: FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<VaultCreateProofOfAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<VaultCreateProofOfAmountOutput>(),
                    export_name: FUNGIBLE_VAULT_CREATE_PROOF_BY_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleVaultLockFungibleAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleVaultLockFungibleAmountOutput>(),
                    export_name: FUNGIBLE_VAULT_LOCK_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleVaultUnlockFungibleAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleVaultUnlockFungibleAmountOutput>(
                        ),
                    export_name: FUNGIBLE_VAULT_UNLOCK_AMOUNT_EXPORT_NAME.to_string(),
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
                parent: Some(FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                schema,
                substates,
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema,
            }
        };

        //====================================================================================

        let non_fungible_vault_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
            let mut substates = Vec::new();
            substates.push(
                aggregator.add_child_type_and_descendents::<NonFungibleVaultIdTypeSubstate>(),
            );
            substates.push(
                aggregator.add_child_type_and_descendents::<NonFungibleVaultBalanceSubstate>(),
            );
            substates
                .push(aggregator.add_child_type_and_descendents::<LockedNonFungibleResource>());

            let mut functions = BTreeMap::new();
            functions.insert(
                VAULT_TAKE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<VaultTakeInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultTakeOutput>(),
                    export_name: NON_FUNGIBLE_VAULT_TAKE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultTakeNonFungiblesInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultTakeNonFungiblesOutput>(),
                    export_name: NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT.to_string(),
                },
            );
            functions.insert(
                VAULT_RECALL_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<VaultRecallInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultRecallOutput>(),
                    export_name: NON_FUNGIBLE_VAULT_RECALL_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultRecallNonFungiblesInput>(
                        ),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultRecallNonFungiblesOutput>(
                        ),
                    export_name: NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT.to_string(),
                },
            );
            functions.insert(
                VAULT_PUT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<VaultPutInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultPutOutput>(),
                    export_name: NON_FUNGIBLE_VAULT_PUT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_GET_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator.add_child_type_and_descendents::<VaultGetAmountInput>(),
                    output: aggregator.add_child_type_and_descendents::<VaultGetAmountOutput>(),
                    export_name: NON_FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultGetNonFungibleLocalIdsInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultGetNonFungibleLocalIdsOutput>(),
                    export_name: NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
                },
            );
            functions.insert(
                VAULT_CREATE_PROOF_OF_ALL_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<VaultCreateProofOfAllInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<VaultCreateProofOfAmountOutput>(),
                    export_name: NON_FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                VAULT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<VaultCreateProofOfAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<VaultCreateProofOfAmountOutput>(),
                    export_name: NON_FUNGIBLE_VAULT_CREATE_PROOF_BY_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultCreateProofOfNonFungiblesInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultCreateProofOfNonFungiblesOutput>(),
                    export_name: NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultLockNonFungiblesInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultLockNonFungiblesOutput>(),
                    export_name: NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultUnlockNonFungiblesInput>(
                        ),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleVaultUnlockNonFungiblesOutput>(
                        ),
                    export_name: NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_EXPORT_NAME.to_string(),
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
                parent: Some(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                schema,
                substates,
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema,
            }
        };

        //====================================================================================

        let fungible_bucket_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut substates = Vec::new();
            substates.push(aggregator.add_child_type_and_descendents::<BucketInfoSubstate>());
            substates.push(aggregator.add_child_type_and_descendents::<LiquidFungibleResource>());
            substates.push(aggregator.add_child_type_and_descendents::<LockedFungibleResource>());

            let mut functions = BTreeMap::new();
            functions.insert(
                BUCKET_PUT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<BucketPutInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketPutOutput>(),
                    export_name: FUNGIBLE_BUCKET_PUT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_TAKE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<BucketTakeInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketTakeOutput>(),
                    export_name: FUNGIBLE_BUCKET_TAKE_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_GET_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator.add_child_type_and_descendents::<BucketGetAmountInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketGetAmountOutput>(),
                    export_name: FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_GET_RESOURCE_ADDRESS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator
                        .add_child_type_and_descendents::<BucketGetResourceAddressInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<BucketGetResourceAddressOutput>(),
                    export_name: FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<BucketCreateProofInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketCreateProofOutput>(),
                    export_name: FUNGIBLE_BUCKET_CREATE_PROOF_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleBucketLockAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleBucketLockAmountOutput>(),
                    export_name: FUNGIBLE_BUCKET_LOCK_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<FungibleBucketUnlockAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<FungibleBucketUnlockAmountOutput>(),
                    export_name: FUNGIBLE_BUCKET_UNLOCK_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            let schema = generate_full_schema(aggregator);
            BlueprintSchema {
                parent: Some(FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                schema,
                substates,
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema: [].into(),
            }
        };

        //====================================================================================

        let non_fungible_bucket_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut substates = Vec::new();
            substates.push(aggregator.add_child_type_and_descendents::<BucketInfoSubstate>());
            substates
                .push(aggregator.add_child_type_and_descendents::<LiquidNonFungibleResource>());
            substates
                .push(aggregator.add_child_type_and_descendents::<LockedNonFungibleResource>());

            let mut functions = BTreeMap::new();
            functions.insert(
                BUCKET_PUT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<BucketPutInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketPutOutput>(),
                    export_name: NON_FUNGIBLE_BUCKET_PUT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_TAKE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<BucketTakeInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketTakeOutput>(),
                    export_name: NON_FUNGIBLE_BUCKET_TAKE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                BUCKET_GET_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator.add_child_type_and_descendents::<BucketGetAmountInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketGetAmountOutput>(),
                    export_name: NON_FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_GET_RESOURCE_ADDRESS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator
                        .add_child_type_and_descendents::<BucketGetResourceAddressInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<BucketGetResourceAddressOutput>(),
                    export_name: NON_FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                BUCKET_CREATE_PROOF_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator.add_child_type_and_descendents::<BucketCreateProofInput>(),
                    output: aggregator.add_child_type_and_descendents::<BucketCreateProofOutput>(),
                    export_name: NON_FUNGIBLE_BUCKET_CREATE_PROOF_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<BucketTakeNonFungiblesInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<BucketTakeNonFungiblesOutput>(),
                    export_name: NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRef),
                    input: aggregator
                        .add_child_type_and_descendents::<BucketGetNonFungibleLocalIdsInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<BucketGetNonFungibleLocalIdsOutput>(),
                    export_name: NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_EXPORT_NAME
                        .to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleBucketLockNonFungiblesInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleBucketLockNonFungiblesOutput>(
                        ),
                    export_name: NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(Receiver::SelfRefMut),
                    input: aggregator
                        .add_child_type_and_descendents::<NonFungibleBucketUnlockNonFungiblesInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<NonFungibleBucketUnlockNonFungiblesOutput>(),
                    export_name: NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_EXPORT_NAME.to_string(),
                },
            );
            let schema = generate_full_schema(aggregator);
            BlueprintSchema {
                parent: Some(NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string()),
                schema,
                substates,
                functions,
                virtual_lazy_load_functions: btreemap!(),
                event_schema: [].into(),
            }
        };

        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut substates = Vec::new();
        substates.push(aggregator.add_child_type_and_descendents::<ProofInfoSubstate>());
        substates.push(aggregator.add_child_type_and_descendents::<FungibleProof>());
        substates.push(aggregator.add_child_type_and_descendents::<NonFungibleProof>());

        let mut functions = BTreeMap::new();
        functions.insert(
            PROOF_DROP_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<ProofDropInput>(),
                output: aggregator.add_child_type_and_descendents::<ProofDropOutput>(),
                export_name: PROOF_DROP_IDENT.to_string(),
            },
        );
        functions.insert(
            PROOF_CLONE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<ProofCloneInput>(),
                output: aggregator.add_child_type_and_descendents::<ProofCloneOutput>(),
                export_name: PROOF_CLONE_IDENT.to_string(),
            },
        );
        functions.insert(
            PROOF_GET_AMOUNT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRef),
                input: aggregator.add_child_type_and_descendents::<ProofGetAmountInput>(),
                output: aggregator.add_child_type_and_descendents::<ProofGetAmountOutput>(),
                export_name: PROOF_GET_AMOUNT_IDENT.to_string(),
            },
        );
        functions.insert(
            PROOF_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRef),
                input: aggregator
                    .add_child_type_and_descendents::<ProofGetNonFungibleLocalIdsInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<ProofGetNonFungibleLocalIdsOutput>(),
                export_name: PROOF_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
            },
        );
        functions.insert(
            PROOF_GET_RESOURCE_ADDRESS_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRef),
                input: aggregator.add_child_type_and_descendents::<ProofGetResourceAddressInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<ProofGetResourceAddressOutput>(),
                export_name: PROOF_GET_RESOURCE_ADDRESS_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        let proof_schema = BlueprintSchema {
            parent: None,
            schema,
            substates,
            functions,
            virtual_lazy_load_functions: btreemap!(),
            event_schema: [].into(),
        };

        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut substates = Vec::new();
        substates.push(aggregator.add_child_type_and_descendents::<WorktopSubstate>());

        let mut functions = BTreeMap::new();
        functions.insert(
            WORKTOP_DROP_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<WorktopDropInput>(),
                output: aggregator.add_child_type_and_descendents::<WorktopDropOutput>(),
                export_name: WORKTOP_DROP_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_PUT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<WorktopPutInput>(),
                output: aggregator.add_child_type_and_descendents::<WorktopPutOutput>(),
                export_name: WORKTOP_PUT_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_TAKE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<WorktopTakeInput>(),
                output: aggregator.add_child_type_and_descendents::<WorktopTakeOutput>(),
                export_name: WORKTOP_TAKE_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_TAKE_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<WorktopTakeNonFungiblesInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<WorktopTakeNonFungiblesOutput>(),
                export_name: WORKTOP_TAKE_NON_FUNGIBLES_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_TAKE_ALL_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<WorktopTakeAllInput>(),
                output: aggregator.add_child_type_and_descendents::<WorktopTakeAllOutput>(),
                export_name: WORKTOP_TAKE_ALL_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_ASSERT_CONTAINS_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<WorktopAssertContainsInput>(),
                output: aggregator.add_child_type_and_descendents::<WorktopAssertContainsOutput>(),
                export_name: WORKTOP_ASSERT_CONTAINS_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<WorktopAssertContainsAmountInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<WorktopAssertContainsAmountOutput>(),
                export_name: WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<WorktopAssertContainsNonFungiblesInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<WorktopAssertContainsNonFungiblesOutput>(),
                export_name: WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT.to_string(),
            },
        );
        functions.insert(
            WORKTOP_DRAIN_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<WorktopDrainInput>(),
                output: aggregator.add_child_type_and_descendents::<WorktopDrainOutput>(),
                export_name: WORKTOP_DRAIN_IDENT.to_string(),
            },
        );
        let schema = generate_full_schema(aggregator);
        let worktop_schema = BlueprintSchema {
            parent: None,
            schema,
            substates,
            functions,
            virtual_lazy_load_functions: btreemap!(),
            event_schema: [].into(),
        };

        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut substates = Vec::new();
        substates.push(aggregator.add_child_type_and_descendents::<AuthZone>());

        let mut functions = BTreeMap::new();
        functions.insert(
            AUTH_ZONE_POP_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<AuthZonePopInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZonePopOutput>(),
                export_name: AUTH_ZONE_POP_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_PUSH_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<AuthZonePushInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZonePushOutput>(),
                export_name: AUTH_ZONE_PUSH_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_CREATE_PROOF_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<AuthZoneCreateProofInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZoneCreateProofOutput>(),
                export_name: AUTH_ZONE_CREATE_PROOF_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_CREATE_PROOF_BY_AMOUNT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AuthZoneCreateProofByAmountInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AuthZoneCreateProofByAmountOutput>(),
                export_name: AUTH_ZONE_CREATE_PROOF_BY_AMOUNT_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_CREATE_PROOF_BY_IDS_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<AuthZoneCreateProofByIdsInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AuthZoneCreateProofByIdsOutput>(),
                export_name: AUTH_ZONE_CREATE_PROOF_BY_IDS_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_CLEAR_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<AuthZoneClearInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZoneClearOutput>(),
                export_name: AUTH_ZONE_CLEAR_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AuthZoneClearVirtualProofsInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AuthZoneClearVirtualProofsOutput>(),
                export_name: AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_EXPORT_NAME.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_DRAIN_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<AuthZoneDrainInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZoneDrainOutput>(),
                export_name: AUTH_ZONE_DRAIN_EXPORT_NAME.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        let auth_zone_schema = BlueprintSchema {
            parent: None,
            schema,
            substates,
            functions,
            event_schema: btreemap!(),
            virtual_lazy_load_functions: btreemap!(),
        };

        PackageSchema {
            blueprints: btreemap!(
                FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string() => fungible_resource_manager_schema,
                NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string() => non_fungible_resource_manager_schema,
                FUNGIBLE_VAULT_BLUEPRINT.to_string() => fungible_vault_schema,
                NON_FUNGIBLE_VAULT_BLUEPRINT.to_string() => non_fungible_vault_schema,
                FUNGIBLE_BUCKET_BLUEPRINT.to_string() => fungible_bucket_schema,
                NON_FUNGIBLE_BUCKET_BLUEPRINT.to_string() => non_fungible_bucket_schema,
                PROOF_BLUEPRINT.to_string() => proof_schema,
                WORKTOP_BLUEPRINT.to_string() => worktop_schema,
                AUTH_ZONE_BLUEPRINT.to_string() => auth_zone_schema
            ),
        }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<&NodeId>,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
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
            FUNGIBLE_RESOURCE_MANAGER_CREATE_BUCKET_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: FungibleResourceManagerCreateBucketInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;

                let rtn = FungibleResourceManagerBlueprint::create_bucket(input.amount, api)?;
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
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_BUCKET_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: NonFungibleResourceManagerCreateBucketInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;

                let rtn = NonFungibleResourceManagerBlueprint::create_bucket(input.entries, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let _input: ResourceManagerCreateEmptyVaultInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::create_vault(api)?;

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
            FUNGIBLE_VAULT_RECALL_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultRecallInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::recall(input.amount, api)?;
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
            FUNGIBLE_VAULT_PUT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_MEDIUM_FEE, ClientCostingReason::RunNative)?;

                let input: VaultPutInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::put(input.bucket, api)?;
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
            FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: VaultGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::get_amount(api)?;
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
            FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: VaultCreateProofOfAllInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::create_proof(receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_CREATE_PROOF_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: VaultCreateProofOfAllInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::create_proof(receiver, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_CREATE_PROOF_BY_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: VaultCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn =
                    FungibleVaultBlueprint::create_proof_by_amount(receiver, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_CREATE_PROOF_BY_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: VaultCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn =
                    NonFungibleVaultBlueprint::create_proof_by_amount(receiver, input.amount, api)?;
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
                let rtn = NonFungibleVaultBlueprint::create_proof_by_ids(receiver, input.ids, api)?;
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
            PROOF_DROP_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                ProofBlueprint::drop(input, api)
            }
            PROOF_CLONE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                ProofBlueprint::clone(input, api)
            }
            PROOF_GET_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                ProofBlueprint::get_amount(input, api)
            }
            PROOF_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                ProofBlueprint::get_non_fungible_local_ids(input, api)
            }
            PROOF_GET_RESOURCE_ADDRESS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                ProofBlueprint::get_resource_address(input, api)
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

                FungibleBucketBlueprint::get_amount(input, api)
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
                FungibleBucketBlueprint::create_proof(receiver, input, api)
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

                NonFungibleBucketBlueprint::get_amount(input, api)
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
                NonFungibleBucketBlueprint::create_proof(receiver, input, api)
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

                AuthZoneBlueprint::pop(input, api)
            }
            AUTH_ZONE_PUSH_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                AuthZoneBlueprint::push(input, api)
            }
            AUTH_ZONE_CREATE_PROOF_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                AuthZoneBlueprint::create_proof(input, api)
            }
            AUTH_ZONE_CREATE_PROOF_BY_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                AuthZoneBlueprint::create_proof_by_amount(input, api)
            }
            AUTH_ZONE_CREATE_PROOF_BY_IDS_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                AuthZoneBlueprint::create_proof_by_ids(input, api)
            }
            AUTH_ZONE_CLEAR_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                AuthZoneBlueprint::clear(input, api)
            }
            AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                AuthZoneBlueprint::clear_signature_proofs(input, api)
            }
            AUTH_ZONE_DRAIN_EXPORT_NAME => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                AuthZoneBlueprint::drain(input, api)
            }
            _ => Err(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
