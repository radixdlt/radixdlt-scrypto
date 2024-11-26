use super::fungible::*;
use super::non_fungible::*;
use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::system_callback::SystemLockData;
use radix_blueprint_schema_init::*;
use radix_engine_interface::api::SystemApi;
use radix_engine_interface::blueprints::hooks::*;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, FunctionAuth, MethodAuthTemplate,
    PackageDefinition, RoleSpecification, StaticRoleDefinition,
};
use radix_engine_interface::blueprints::resource::*;

pub(crate) const FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME: &str =
    "create_FungibleResourceManager";
pub(crate) const FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME: &str =
    "create_with_initial_supply_and_address_FungibleResourceManager";
pub(crate) const FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME: &str = "burn_FungibleResourceManager";
pub(crate) const FUNGIBLE_RESOURCE_MANAGER_PACKAGE_BURN_EXPORT_NAME: &str =
    "package_burn_FungibleResourceManager";
pub(crate) const FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME: &str = "mint_FungibleResourceManager";
pub(crate) const FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME: &str =
    "create_empty_vault_FungibleResourceManager";
pub(crate) const FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME: &str =
    "create_empty_bucket_FungibleResourceManager";
pub(crate) const FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME: &str =
    "get_resource_type_FungibleResourceManager";
pub(crate) const FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME: &str =
    "get_total_supply_FungibleResourceManager";
pub(crate) const FUNGIBLE_RESOURCE_MANAGER_AMOUNT_FOR_WITHDRAWAL_EXPORT_NAME: &str =
    "amount_for_withdrawal_FungibleResourceManager";
pub(crate) const FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME: &str =
    "drop_empty_bucket_FungibleResourceManager";

pub(crate) const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME: &str =
    "create_NonFungibleResourceManager";
pub(crate) const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME: &str =
    "create_with_initial_supply_NonFungibleResourceManager";
pub(crate) const NON_FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME: &str =
    "burn_NonFungibleResourceManager";
pub(crate) const NON_FUNGIBLE_RESOURCE_MANAGER_PACKAGE_BURN_EXPORT_NAME: &str =
    "package_burn_NonFungibleResourceManager";
pub(crate) const NON_FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME: &str =
    "mint_NonFungibleResourceManager";
pub(crate) const NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_EXPORT_NAME: &str =
    "mint_ruid_NonFungibleResourceManager";
pub(crate) const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME: &str =
    "create_empty_vault_NonFungibleResourceManager";
pub(crate) const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME: &str =
    "create_empty_bucket_NonFungibleResourceManager";
pub(crate) const NON_FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME: &str =
    "get_resource_type_NonFungibleResourceManager";
pub(crate) const NON_FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME: &str =
    "get_total_supply_NonFungibleResourceManager";
pub(crate) const NON_FUNGIBLE_RESOURCE_MANAGER_AMOUNT_FOR_WITHDRAWAL_EXPORT_NAME: &str =
    "amount_for_withdrawal_NonFungibleResourceManager";
pub(crate) const NON_FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME: &str =
    "drop_empty_bucket_NonFungibleResourceManager";

pub(crate) const FUNGIBLE_VAULT_TAKE_EXPORT_NAME: &str = "take_FungibleVault";
pub(crate) const FUNGIBLE_VAULT_TAKE_ADVANCED_EXPORT_NAME: &str = "take_advanced_FungibleVault";
pub(crate) const FUNGIBLE_VAULT_PUT_EXPORT_NAME: &str = "put_FungibleVault";
pub(crate) const FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_FungibleVault";
pub(crate) const FUNGIBLE_VAULT_RECALL_EXPORT_NAME: &str = "recall_FungibleVault";
pub(crate) const FUNGIBLE_VAULT_FREEZE_EXPORT_NAME: &str = "freeze_FungibleVault";
pub(crate) const FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME: &str = "unfreeze_FungibleVault";
pub(crate) const FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME: &str =
    "create_proof_of_amount_FungibleVault";
pub(crate) const FUNGIBLE_VAULT_LOCK_AMOUNT_EXPORT_NAME: &str = "lock_amount_FungibleVault";
pub(crate) const FUNGIBLE_VAULT_UNLOCK_AMOUNT_EXPORT_NAME: &str = "unlock_amount_FungibleVault";
pub(crate) const FUNGIBLE_VAULT_BURN_EXPORT_NAME: &str = "burn_FungibleVault";

pub(crate) const NON_FUNGIBLE_VAULT_TAKE_EXPORT_NAME: &str = "take_NonFungibleVault";
pub(crate) const NON_FUNGIBLE_VAULT_TAKE_ADVANCED_EXPORT_NAME: &str =
    "take_advanced_NonFungibleVault";
pub(crate) const NON_FUNGIBLE_VAULT_PUT_EXPORT_NAME: &str = "put_NonFungibleVault";
pub(crate) const NON_FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_NonFungibleVault";
pub(crate) const NON_FUNGIBLE_VAULT_RECALL_EXPORT_NAME: &str = "recall_NonFungibleVault";
pub(crate) const NON_FUNGIBLE_VAULT_FREEZE_EXPORT_NAME: &str = "freeze_NonFungibleVault";
pub(crate) const NON_FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME: &str = "unfreeze_NonFungibleVault";
pub(crate) const NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_EXPORT_NAME: &str =
    "unlock_fungibles_NonFungibleVault";
pub(crate) const NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_EXPORT_NAME: &str =
    "unlock_non_fungibles_NonFungibleVault";
pub(crate) const NON_FUNGIBLE_VAULT_BURN_EXPORT_NAME: &str = "burn_NonFungibleVault";

pub(crate) const FUNGIBLE_BUCKET_TAKE_EXPORT_NAME: &str = "take_FungibleBucket";
pub(crate) const FUNGIBLE_BUCKET_TAKE_ADVANCED_EXPORT_NAME: &str = "take_advanced_FungibleBucket";
pub(crate) const FUNGIBLE_BUCKET_PUT_EXPORT_NAME: &str = "put_FungibleBucket";
pub(crate) const FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_FungibleBucket";
pub(crate) const FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME: &str =
    "get_resource_address_FungibleBucket";
pub(crate) const FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME: &str =
    "create_proof_of_amount_FungibleBucket";
pub(crate) const FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME: &str =
    "create_proof_of_all_FungibleBucket";
pub(crate) const FUNGIBLE_BUCKET_LOCK_AMOUNT_EXPORT_NAME: &str = "lock_amount_FungibleBucket";
pub(crate) const FUNGIBLE_BUCKET_UNLOCK_AMOUNT_EXPORT_NAME: &str = "unlock_amount_FungibleBucket";

pub(crate) const NON_FUNGIBLE_BUCKET_TAKE_EXPORT_NAME: &str = "take_NonFungibleBucket";
pub(crate) const NON_FUNGIBLE_BUCKET_TAKE_ADVANCED_EXPORT_NAME: &str =
    "take_advanced_NonFungibleBucket";
pub(crate) const NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_EXPORT_NAME: &str =
    "take_non_fungibles_NonFungibleBucket";
pub(crate) const NON_FUNGIBLE_BUCKET_PUT_EXPORT_NAME: &str = "put_NonFungibleBucket";
pub(crate) const NON_FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_NonFungibleBucket";
pub(crate) const NON_FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME: &str =
    "get_resource_address_NonFungibleBucket";
pub(crate) const NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_EXPORT_NAME: &str =
    "create_proof_of_non_fungibles_NonFungibleBucket";
pub(crate) const NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME: &str =
    "create_proof_of_all_NonFungibleBucket";
pub(crate) const NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_EXPORT_NAME: &str =
    "unlock_fungibles_NonFungibleBucket";
pub(crate) const NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_EXPORT_NAME: &str =
    "unlock_non_fungibles_NonFungibleBucket";
pub(crate) const NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_EXPORT_NAME: &str =
    "get_non_fungible_local_ids_NonFungibleBucket";
pub(crate) const NON_FUNGIBLE_BUCKET_CONTAINS_NON_FUNGIBLE_EXPORT_NAME: &str =
    "contains_non_fungible_NonFungibleBucket";

pub(crate) const FUNGIBLE_PROOF_CLONE_EXPORT_NAME: &str = "clone_FungibleProof";
pub(crate) const FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_FungibleProof";
pub(crate) const FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME: &str =
    "get_resource_address_FungibleProof";
pub(crate) const FUNGIBLE_PROOF_DROP_EXPORT_NAME: &str = "drop_FungibleProof";
pub(crate) const FUNGIBLE_PROOF_ON_DROP_EXPORT_NAME: &str = "on_drop_FungibleProof";
pub(crate) const FUNGIBLE_PROOF_ON_MOVE_EXPORT_NAME: &str = "on_move_FungibleProof";

pub(crate) const NON_FUNGIBLE_PROOF_CLONE_EXPORT_NAME: &str = "clone_NonFungibleProof";
pub(crate) const NON_FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME: &str = "get_amount_NonFungibleProof";
pub(crate) const NON_FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME: &str =
    "get_resource_address_NonFungibleProof";
pub(crate) const NON_FUNGIBLE_PROOF_DROP_EXPORT_NAME: &str = "drop_NonFungibleProof";
pub(crate) const NON_FUNGIBLE_PROOF_ON_DROP_EXPORT_NAME: &str = "on_drop_NonFungibleProof";
pub(crate) const NON_FUNGIBLE_PROOF_ON_MOVE_EXPORT_NAME: &str = "on_move_NonFungibleProof";

pub(crate) const AUTH_ZONE_POP_EXPORT_NAME: &str = "AuthZone_pop";
pub(crate) const AUTH_ZONE_PUSH_EXPORT_NAME: &str = "AuthZone_push";
pub(crate) const AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME: &str =
    "AuthZone_create_proof_of_amount";
pub(crate) const AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_EXPORT_NAME: &str =
    "AuthZone_create_proof_of_non_fungibles";
pub(crate) const AUTH_ZONE_CREATE_PROOF_OF_ALL_EXPORT_NAME: &str = "AuthZone_create_proof_of_all";
pub(crate) const AUTH_ZONE_DROP_SIGNATURE_PROOFS_EXPORT_NAME: &str =
    "AuthZone_drop_signature_proofs";
pub(crate) const AUTH_ZONE_DROP_REGULAR_PROOFS_EXPORT_NAME: &str = "AuthZone_drop_regular_proofs";
pub(crate) const AUTH_ZONE_DROP_PROOFS_EXPORT_NAME: &str = "AuthZone_drop_proofs";
pub(crate) const AUTH_ZONE_DRAIN_EXPORT_NAME: &str = "AuthZone_drain";
pub(crate) const AUTH_ZONE_ASSERT_ACCESS_RULE_EXPORT_NAME: &str = "AuthZone_assert_access_rule";

pub struct ResourceNativePackage;

impl ResourceNativePackage {
    pub fn definition() -> PackageDefinition {
        let fungible_resource_manager_blueprint =
            FungibleResourceManagerBlueprint::get_definition();

        let non_fungible_resource_manager_blueprint =
            NonFungibleResourceManagerBlueprint::get_definition();

        let fungible_vault_blueprint = FungibleVaultBlueprint::get_definition();

        let non_fungible_vault_blueprint = NonFungibleVaultBlueprint::get_definition();

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

            let mut functions = index_map_new();
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
                BUCKET_TAKE_ADVANCED_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketTakeAdvancedInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketTakeAdvancedOutput>(),
                    ),
                    export: FUNGIBLE_BUCKET_TAKE_ADVANCED_EXPORT_NAME.to_string(),
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
                FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<FungibleBucketCreateProofOfAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<FungibleBucketCreateProofOfAmountOutput>(),
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
                blueprint_type: BlueprintType::Inner {
                    outer_blueprint: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                },
                is_transient: true,
                dependencies: indexset!(),
                feature_set: indexset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: BlueprintEventSchemaInit::default(),
                    types: BlueprintTypeSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit { functions },
                    hooks: BlueprintHooksInit::default(),
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::StaticRoleDefinition(StaticRoleDefinition {
                        methods: method_auth_template! {
                            BUCKET_GET_AMOUNT_IDENT => MethodAccessibility::Public;
                            BUCKET_GET_RESOURCE_ADDRESS_IDENT => MethodAccessibility::Public;
                            BUCKET_CREATE_PROOF_OF_ALL_IDENT => MethodAccessibility::Public;
                            FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT => MethodAccessibility::Public;
                            BUCKET_PUT_IDENT => MethodAccessibility::Public;
                            BUCKET_TAKE_IDENT => MethodAccessibility::Public;
                            BUCKET_TAKE_ADVANCED_IDENT => MethodAccessibility::Public;

                            FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT => MethodAccessibility::OwnPackageOnly;
                            FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT => MethodAccessibility::OwnPackageOnly;
                        },
                        roles: RoleSpecification::UseOuter,
                    }),
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

            let mut functions = index_map_new();
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
                BUCKET_TAKE_ADVANCED_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketTakeAdvancedInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<BucketTakeAdvancedOutput>(),
                    ),
                    export: NON_FUNGIBLE_BUCKET_TAKE_ADVANCED_EXPORT_NAME.to_string(),
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
                NON_FUNGIBLE_BUCKET_CONTAINS_NON_FUNGIBLE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<NonFungibleBucketContainsNonFungibleInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<NonFungibleBucketContainsNonFungibleOutput>(),
                    ),
                    export: NON_FUNGIBLE_BUCKET_CONTAINS_NON_FUNGIBLE_EXPORT_NAME.to_string(),
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
                blueprint_type: BlueprintType::Inner {
                    outer_blueprint: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                },
                is_transient: true,
                dependencies: indexset!(),
                feature_set: indexset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: BlueprintEventSchemaInit::default(),
                    types: BlueprintTypeSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit { functions },
                    hooks: BlueprintHooksInit::default(),
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::StaticRoleDefinition(StaticRoleDefinition {
                        methods: method_auth_template! {
                            BUCKET_GET_AMOUNT_IDENT => MethodAccessibility::Public;
                            BUCKET_GET_RESOURCE_ADDRESS_IDENT => MethodAccessibility::Public;
                            BUCKET_CREATE_PROOF_OF_ALL_IDENT => MethodAccessibility::Public;
                            BUCKET_PUT_IDENT => MethodAccessibility::Public;
                            BUCKET_TAKE_IDENT => MethodAccessibility::Public;
                            BUCKET_TAKE_ADVANCED_IDENT => MethodAccessibility::Public;
                            NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_IDENT => MethodAccessibility::Public;
                            NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => MethodAccessibility::Public;
                            NON_FUNGIBLE_BUCKET_CONTAINS_NON_FUNGIBLE_IDENT => MethodAccessibility::Public;
                            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT => MethodAccessibility::Public;

                            NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT => MethodAccessibility::OwnPackageOnly;
                            NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_IDENT => MethodAccessibility::OwnPackageOnly;
                        },
                        roles: RoleSpecification::UseOuter,
                    }),
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

            let mut functions = index_map_new();
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
                blueprint_type: BlueprintType::Inner {
                    outer_blueprint: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                },
                is_transient: true,
                dependencies: indexset!(),
                feature_set: indexset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: BlueprintEventSchemaInit::default(),
                    types: BlueprintTypeSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit { functions },
                    hooks: BlueprintHooksInit {
                        hooks: indexmap!(
                            BlueprintHook::OnDrop => FUNGIBLE_PROOF_ON_DROP_EXPORT_NAME.to_string(),
                            BlueprintHook::OnMove => FUNGIBLE_PROOF_ON_MOVE_EXPORT_NAME.to_string(),
                        ),
                    },
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::AllowAll,
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

            let mut functions = index_map_new();
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
                blueprint_type: BlueprintType::Inner {
                    outer_blueprint: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                },
                is_transient: true,
                dependencies: indexset!(),
                feature_set: indexset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events: BlueprintEventSchemaInit::default(),
                    types: BlueprintTypeSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit { functions },
                    hooks: BlueprintHooksInit {
                        hooks: indexmap!(
                            BlueprintHook::OnDrop => NON_FUNGIBLE_PROOF_ON_DROP_EXPORT_NAME.to_string(),
                            BlueprintHook::OnMove => NON_FUNGIBLE_PROOF_ON_MOVE_EXPORT_NAME.to_string()
                        ),
                    },
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::AllowAll,
                },
            }
        };

        let worktop_blueprint = WorktopBlueprint::get_definition();

        let auth_zone_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<GenesisSchemaAuthZone>(),
            ));

            let mut functions = index_map_new();
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
                AUTH_ZONE_DROP_PROOFS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZoneDropProofsInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<AuthZoneDropProofsOutput>(),
                    ),
                    export: AUTH_ZONE_DROP_PROOFS_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                AUTH_ZONE_DROP_SIGNATURE_PROOFS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<AuthZoneDropSignatureProofsInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<AuthZoneDropSignatureProofsOutput>(),
                    ),
                    export: AUTH_ZONE_DROP_SIGNATURE_PROOFS_EXPORT_NAME.to_string(),
                },
            );
            functions.insert(
                AUTH_ZONE_DROP_REGULAR_PROOFS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<AuthZoneDropSignatureProofsInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<AuthZoneDropSignatureProofsOutput>(),
                    ),
                    export: AUTH_ZONE_DROP_REGULAR_PROOFS_EXPORT_NAME.to_string(),
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
                AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<AuthZoneAssertAccessRuleInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<AuthZoneAssertAccessRuleOutput>(),
                    ),
                    export: AUTH_ZONE_ASSERT_ACCESS_RULE_EXPORT_NAME.to_string(),
                },
            );

            let schema = generate_full_schema(aggregator);
            let auth_zone_blueprint = BlueprintStateSchemaInit {
                fields,
                collections: vec![],
            };

            BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                is_transient: true,
                dependencies: indexset!(),
                feature_set: indexset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: auth_zone_blueprint,
                    events: BlueprintEventSchemaInit::default(),
                    types: BlueprintTypeSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit { functions },
                    hooks: BlueprintHooksInit::default(),
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::AllowAll,
                },
            }
        };

        let blueprints = indexmap!(
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

    pub fn invoke_export<
        Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    >(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME => {
                let input: FungibleResourceManagerCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::create(
                    input.owner_role,
                    input.track_total_supply,
                    input.divisibility,
                    input.resource_roles,
                    input.metadata,
                    input.address_reservation,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME => {
                let input: FungibleResourceManagerCreateWithInitialSupplyInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleResourceManagerBlueprint::create_with_initial_supply(
                    input.owner_role,
                    input.track_total_supply,
                    input.divisibility,
                    input.initial_supply,
                    input.resource_roles,
                    input.metadata,
                    input.address_reservation,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME => {
                let input: FungibleResourceManagerMintInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::mint(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME => {
                let input: ResourceManagerBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::burn(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_PACKAGE_BURN_EXPORT_NAME => {
                let input: ResourceManagerPackageBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::package_burn(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME => {
                let input: ResourceManagerDropEmptyBucketInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::drop_empty_bucket(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME => {
                let _input: ResourceManagerCreateEmptyVaultInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleResourceManagerBlueprint::create_empty_vault(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME => {
                let _input: ResourceManagerCreateEmptyBucketInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;

                let rtn = FungibleResourceManagerBlueprint::create_empty_bucket(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME => {
                let _input: ResourceManagerGetResourceTypeInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleResourceManagerBlueprint::get_resource_type(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME => {
                let _input: ResourceManagerGetTotalSupplyInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleResourceManagerBlueprint::get_total_supply(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_RESOURCE_MANAGER_AMOUNT_FOR_WITHDRAWAL_EXPORT_NAME => {
                let input: ResourceManagerGetAmountForWithdrawalInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleResourceManagerBlueprint::amount_for_withdrawal(
                    api,
                    input.request_amount,
                    input.withdraw_strategy,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EXPORT_NAME => {
                let input: NonFungibleResourceManagerCreateInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::create(
                    input.owner_role,
                    input.id_type,
                    input.track_total_supply,
                    input.non_fungible_schema,
                    input.resource_roles,
                    input.metadata,
                    input.address_reservation,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_EXPORT_NAME => {
                let input: NonFungibleResourceManagerCreateWithInitialSupplyInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::create_with_initial_supply(
                    input.owner_role,
                    input.id_type,
                    input.track_total_supply,
                    input.non_fungible_schema,
                    input.entries,
                    input.resource_roles,
                    input.metadata,
                    input.address_reservation,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT => {
                let input: NonFungibleResourceManagerCreateRuidWithInitialSupplyInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;

                let rtn = NonFungibleResourceManagerBlueprint::create_ruid_with_initial_supply(
                    input.owner_role,
                    input.track_total_supply,
                    input.non_fungible_schema,
                    input.entries,
                    input.resource_roles,
                    input.metadata,
                    input.address_reservation,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_EXPORT_NAME => {
                let input: NonFungibleResourceManagerMintInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    NonFungibleResourceManagerBlueprint::mint_non_fungible(input.entries, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_EXPORT_NAME => {
                let input: NonFungibleResourceManagerMintRuidInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::mint_ruid_non_fungible(
                    input.entries,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_RUID_IDENT => {
                let input: NonFungibleResourceManagerMintSingleRuidInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::mint_single_ruid_non_fungible(
                    input.entry,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_BURN_EXPORT_NAME => {
                let input: ResourceManagerBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleResourceManagerBlueprint::burn(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_PACKAGE_BURN_EXPORT_NAME => {
                let input: ResourceManagerPackageBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleResourceManagerBlueprint::package_burn(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_DROP_EMPTY_BUCKET_EXPORT_NAME => {
                let input: ResourceManagerDropEmptyBucketInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    NonFungibleResourceManagerBlueprint::drop_empty_bucket(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_EXPORT_NAME => {
                let _input: ResourceManagerCreateEmptyBucketInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;

                let rtn = NonFungibleResourceManagerBlueprint::create_empty_bucket(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_EMPTY_VAULT_EXPORT_NAME => {
                let _input: ResourceManagerCreateEmptyVaultInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::create_empty_vault(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT => {
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
                let input: NonFungibleResourceManagerExistsInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::non_fungible_exists(input.id, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_RESOURCE_TYPE_EXPORT_NAME => {
                let _input: ResourceManagerGetResourceTypeInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::get_resource_type(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_TOTAL_SUPPLY_EXPORT_NAME => {
                let _input: ResourceManagerGetTotalSupplyInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleResourceManagerBlueprint::get_total_supply(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_AMOUNT_FOR_WITHDRAWAL_EXPORT_NAME => {
                let input: ResourceManagerGetAmountForWithdrawalInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::amount_for_withdrawal(
                    api,
                    input.request_amount,
                    input.withdraw_strategy,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT => {
                let input: NonFungibleResourceManagerGetNonFungibleInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleResourceManagerBlueprint::get_non_fungible(input.id, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            FUNGIBLE_VAULT_LOCK_FEE_IDENT => {
                let input: FungibleVaultLockFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::lock_fee(input.amount, input.contingent, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_TAKE_EXPORT_NAME => {
                let input: VaultTakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::take(&input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_TAKE_ADVANCED_EXPORT_NAME => {
                let input: VaultTakeAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::take_advanced(
                    &input.amount,
                    input.withdraw_strategy,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_RECALL_EXPORT_NAME => {
                let input: VaultRecallInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::recall(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_FREEZE_EXPORT_NAME => {
                let input: VaultFreezeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::freeze(input.to_freeze, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME => {
                let input: VaultUnfreezeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::unfreeze(input.to_unfreeze, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_PUT_EXPORT_NAME => {
                let input: VaultPutInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::put(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME => {
                let _input: VaultGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::get_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME => {
                let input: FungibleVaultCreateProofOfAmountInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleVaultBlueprint::create_proof_of_amount(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_LOCK_AMOUNT_EXPORT_NAME => {
                let input: FungibleVaultLockFungibleAmountInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleVaultBlueprint::lock_amount(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_UNLOCK_AMOUNT_EXPORT_NAME => {
                let input: FungibleVaultUnlockFungibleAmountInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleVaultBlueprint::unlock_amount(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_VAULT_BURN_EXPORT_NAME => {
                let input: VaultBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = FungibleVaultBlueprint::burn(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            NON_FUNGIBLE_VAULT_TAKE_ADVANCED_EXPORT_NAME => {
                let input: VaultTakeAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::take_advanced(
                    &input.amount,
                    input.withdraw_strategy,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_TAKE_EXPORT_NAME => {
                let input: VaultTakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::take(&input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT => {
                let input: NonFungibleVaultTakeNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::take_non_fungibles(
                    &input.non_fungible_local_ids,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_RECALL_EXPORT_NAME => {
                let input: VaultRecallInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::recall(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_FREEZE_EXPORT_NAME => {
                let input: VaultFreezeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::freeze(input.to_freeze, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_UNFREEZE_EXPORT_NAME => {
                let input: VaultUnfreezeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::unfreeze(input.to_unfreeze, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT => {
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
                let input: VaultPutInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::put(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_GET_AMOUNT_EXPORT_NAME => {
                let _input: VaultGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::get_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT => {
                let input: NonFungibleVaultGetNonFungibleLocalIdsInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::get_non_fungible_local_ids(input.limit, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_CONTAINS_NON_FUNGIBLE_IDENT => {
                let input: NonFungibleVaultContainsNonFungibleInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::contains_non_fungible(input.id, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT => {
                let input: NonFungibleVaultCreateProofOfNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::create_proof_of_non_fungibles(input.ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_EXPORT_NAME => {
                let input: NonFungibleVaultLockNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::lock_non_fungibles(&input.local_ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_EXPORT_NAME => {
                let input: NonFungibleVaultUnlockNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::unlock_non_fungibles(input.local_ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_BURN_EXPORT_NAME => {
                let input: VaultBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleVaultBlueprint::burn(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT => {
                let input: NonFungibleVaultBurnNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleVaultBlueprint::burn_non_fungibles(
                    &input.non_fungible_local_ids,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            FUNGIBLE_PROOF_CLONE_EXPORT_NAME => {
                let _input: ProofCloneInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleProofBlueprint::clone(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME => {
                let _input: ProofGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleProofBlueprint::get_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME => {
                let _input: ProofGetResourceAddressInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = FungibleProofBlueprint::get_resource_address(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_PROOF_DROP_EXPORT_NAME => {
                let input: ProofDropInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleProofBlueprint::drop(input.proof, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_PROOF_ON_DROP_EXPORT_NAME => {
                let _input: OnDropInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleProofBlueprint::on_drop(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_PROOF_ON_MOVE_EXPORT_NAME => {
                let input: OnMoveInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleProofBlueprint::on_move(
                    input.is_moving_down,
                    input.is_to_barrier,
                    input.destination_blueprint_id,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_CLONE_EXPORT_NAME => {
                let _input: ProofCloneInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleProofBlueprint::clone(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_GET_AMOUNT_EXPORT_NAME => {
                let _input: ProofGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleProofBlueprint::get_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT => {
                let _input: NonFungibleProofGetLocalIdsInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleProofBlueprint::get_local_ids(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            NON_FUNGIBLE_PROOF_GET_RESOURCE_ADDRESS_EXPORT_NAME => {
                let _input: ProofGetResourceAddressInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = NonFungibleProofBlueprint::get_resource_address(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_DROP_EXPORT_NAME => {
                let input: ProofDropInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleProofBlueprint::drop(input.proof, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_ON_DROP_EXPORT_NAME => {
                let _input: OnDropInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleProofBlueprint::on_drop(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_PROOF_ON_MOVE_EXPORT_NAME => {
                let input: OnMoveInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleProofBlueprint::on_move(
                    input.is_moving_down,
                    input.is_to_barrier,
                    input.destination_blueprint_id,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            FUNGIBLE_BUCKET_PUT_EXPORT_NAME => {
                let input: BucketPutInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleBucketBlueprint::put(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_BUCKET_TAKE_EXPORT_NAME => {
                let input: BucketTakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleBucketBlueprint::take(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_BUCKET_TAKE_ADVANCED_EXPORT_NAME => {
                let input: BucketTakeAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleBucketBlueprint::take_advanced(
                    input.amount,
                    input.withdraw_strategy,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME => {
                let _input: BucketGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let amount = FungibleBucketBlueprint::get_amount(api)?;

                Ok(IndexedScryptoValue::from_typed(&amount))
            }
            FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME => {
                let _input: BucketGetResourceAddressInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleBucketBlueprint::get_resource_address(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME => {
                let input: FungibleBucketCreateProofOfAmountInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = FungibleBucketBlueprint::create_proof_of_amount(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME => {
                let _input: BucketCreateProofOfAllInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleBucketBlueprint::create_proof_of_all(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_BUCKET_LOCK_AMOUNT_EXPORT_NAME => {
                let input: FungibleBucketLockAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleBucketBlueprint::lock_amount(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            FUNGIBLE_BUCKET_UNLOCK_AMOUNT_EXPORT_NAME => {
                let input: FungibleBucketLockAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = FungibleBucketBlueprint::unlock_amount(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_PUT_EXPORT_NAME => {
                let input: BucketPutInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleBucketBlueprint::put(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_TAKE_EXPORT_NAME => {
                let input: BucketTakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleBucketBlueprint::take(&input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_TAKE_ADVANCED_EXPORT_NAME => {
                let input: BucketTakeAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleBucketBlueprint::take_advanced(
                    &input.amount,
                    input.withdraw_strategy,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_EXPORT_NAME => {
                let input: BucketTakeNonFungiblesInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleBucketBlueprint::take_non_fungibles(&input.ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_GET_AMOUNT_EXPORT_NAME => {
                let _input: BucketGetAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let amount = NonFungibleBucketBlueprint::get_amount(api)?;

                Ok(IndexedScryptoValue::from_typed(&amount))
            }
            NON_FUNGIBLE_BUCKET_GET_RESOURCE_ADDRESS_EXPORT_NAME => {
                let _input: BucketGetResourceAddressInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let address = NonFungibleBucketBlueprint::get_resource_address(api)?;
                Ok(IndexedScryptoValue::from_typed(&address))
            }
            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_EXPORT_NAME => {
                let input: NonFungibleBucketCreateProofOfNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn =
                    NonFungibleBucketBlueprint::create_proof_of_non_fungibles(input.ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_ALL_EXPORT_NAME => {
                let _input: BucketCreateProofOfAllInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleBucketBlueprint::create_proof_of_all(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_EXPORT_NAME => {
                let _input: BucketGetNonFungibleLocalIdsInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = NonFungibleBucketBlueprint::get_non_fungible_local_ids(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_CONTAINS_NON_FUNGIBLE_EXPORT_NAME => {
                let input: NonFungibleVaultContainsNonFungibleInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleBucketBlueprint::contains_non_fungible(input.id, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_EXPORT_NAME => {
                let input: NonFungibleBucketLockNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleBucketBlueprint::lock_non_fungibles(&input.local_ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_EXPORT_NAME => {
                let input: NonFungibleBucketUnlockNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = NonFungibleBucketBlueprint::unlock_non_fungibles(input.local_ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            WORKTOP_DROP_IDENT => WorktopBlueprint::drop(input, api),
            WORKTOP_PUT_IDENT => WorktopBlueprint::put(input, api),
            WORKTOP_TAKE_IDENT => WorktopBlueprint::take(input, api),
            WORKTOP_TAKE_NON_FUNGIBLES_IDENT => WorktopBlueprint::take_non_fungibles(input, api),
            WORKTOP_TAKE_ALL_IDENT => WorktopBlueprint::take_all(input, api),
            WORKTOP_ASSERT_CONTAINS_IDENT => WorktopBlueprint::assert_contains(input, api),
            WORKTOP_ASSERT_CONTAINS_AMOUNT_IDENT => {
                WorktopBlueprint::assert_contains_amount(input, api)
            }
            WORKTOP_ASSERT_CONTAINS_NON_FUNGIBLES_IDENT => {
                WorktopBlueprint::assert_contains_non_fungibles(input, api)
            }

            WORKTOP_DRAIN_IDENT => WorktopBlueprint::drain(input, api),
            AUTH_ZONE_POP_EXPORT_NAME => {
                let _input: AuthZonePopInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let proof = AuthZoneBlueprint::pop(api)?;

                Ok(IndexedScryptoValue::from_typed(&proof))
            }
            AUTH_ZONE_PUSH_EXPORT_NAME => {
                let input: AuthZonePushInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                AuthZoneBlueprint::push(input.proof, api)?;

                Ok(IndexedScryptoValue::from_typed(&()))
            }
            AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_EXPORT_NAME => {
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
                let input: AuthZoneCreateProofOfAllInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let proof = AuthZoneBlueprint::create_proof_of_all(input.resource_address, api)?;

                Ok(IndexedScryptoValue::from_typed(&proof))
            }
            AUTH_ZONE_DROP_PROOFS_EXPORT_NAME => {
                let _input: AuthZoneDropProofsInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                AuthZoneBlueprint::drop_proofs(api)?;

                Ok(IndexedScryptoValue::from_typed(&()))
            }
            AUTH_ZONE_DROP_SIGNATURE_PROOFS_EXPORT_NAME => {
                let _input: AuthZoneDropProofsInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                AuthZoneBlueprint::drop_signature_proofs(api)?;

                Ok(IndexedScryptoValue::from_typed(&()))
            }
            AUTH_ZONE_DROP_REGULAR_PROOFS_EXPORT_NAME => {
                let _input: AuthZoneDropProofsInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                AuthZoneBlueprint::drop_regular_proofs(api)?;

                Ok(IndexedScryptoValue::from_typed(&()))
            }
            AUTH_ZONE_DRAIN_EXPORT_NAME => {
                let _input: AuthZoneDrainInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let proofs = AuthZoneBlueprint::drain(api)?;

                Ok(IndexedScryptoValue::from_typed(&proofs))
            }
            AUTH_ZONE_ASSERT_ACCESS_RULE_EXPORT_NAME => {
                let input: AuthZoneAssertAccessRuleInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AuthZoneBlueprint::assert_access_rule(input.rule, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
