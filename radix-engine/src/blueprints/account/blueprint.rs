use super::*;
use crate::blueprints::util::{PresecurifiedRoleAssignment, SecurifiedRoleAssignment};
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::FieldValue;
use radix_engine_interface::api::{AttachedModuleId, GenericArgs, SystemApi, ACTOR_STATE_SELF};
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::hooks::OnVirtualizeInput;
use radix_engine_interface::blueprints::hooks::OnVirtualizeOutput;
use radix_engine_interface::blueprints::resource::{Bucket, Proof};
use radix_engine_interface::metadata_init;
use radix_engine_interface::object_modules::metadata::*;
use radix_native_sdk::modules::metadata::Metadata;
use radix_native_sdk::modules::role_assignment::RoleAssignment;
use radix_native_sdk::resource::NativeFungibleVault;
use radix_native_sdk::resource::NativeNonFungibleVault;
use radix_native_sdk::resource::NativeVault;
use radix_native_sdk::resource::{NativeBucket, NativeNonFungibleBucket};
use radix_native_sdk::runtime::Runtime;

// =================================================================================================
// Notes:
// 1. All deposits should go through the `deposit` method since it emits the deposit events.
// 2. The `try_deposit` methods are responsible for emitting the rejected deposit events.
// =================================================================================================

pub const ACCOUNT_CREATE_PREALLOCATED_SECP256K1_ID: u8 = 0u8;
pub const ACCOUNT_CREATE_PREALLOCATED_ED25519_ID: u8 = 1u8;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, Clone)]
pub struct AccountSubstate {
    pub default_deposit_rule: DefaultDepositRule,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AccountError {
    VaultDoesNotExist { resource_address: ResourceAddress },
    DepositIsDisallowed { resource_address: ResourceAddress },
    NotAllBucketsCouldBeDeposited,
    NotAnAuthorizedDepositor { depositor: ResourceOrNonFungible },
}

impl From<AccountError> for RuntimeError {
    fn from(value: AccountError) -> Self {
        Self::ApplicationError(ApplicationError::AccountError(value))
    }
}

pub const SECURIFY_ROLE: &'static str = "securify";

struct SecurifiedAccount;

impl SecurifiedRoleAssignment for SecurifiedAccount {
    type OwnerBadgeNonFungibleData = AccountOwnerBadgeData;
    const OWNER_BADGE: ResourceAddress = ACCOUNT_OWNER_BADGE;
    const SECURIFY_ROLE: Option<&'static str> = Some(SECURIFY_ROLE);
}

impl PresecurifiedRoleAssignment for SecurifiedAccount {}

declare_native_blueprint_state! {
    blueprint_ident: Account,
    blueprint_snake_case: account,
    features: {
    },
    fields: {
        deposit_rule:  {
            ident: DepositRule,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        }
    },
    collections: {
        resource_vaults: KeyValue {
            entry_ident: ResourceVault,
            key_type: {
                kind: Static,
                content_type: ResourceAddress,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: true,
        },
        resource_preferences: KeyValue {
            entry_ident: ResourcePreference,
            key_type: {
                kind: Static,
                content_type: ResourceAddress,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: false,
        },
        authorized_depositors: KeyValue {
            entry_ident: AuthorizedDepositor,
            key_type: {
                kind: Static,
                content_type: ResourceOrNonFungible,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: false,
        },
    }
}

pub type AccountDepositRuleV1 = AccountSubstate;
pub type AccountResourceVaultV1 = Vault;
pub type AccountResourcePreferenceV1 = ResourcePreference;
pub type AccountAuthorizedDepositorV1 = ();

pub struct AccountBlueprint;

impl AccountBlueprint {
    pub fn get_definition() -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let feature_set = AccountFeatureSet::all_features();
        let state = AccountStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = index_map_new();

        functions.insert(
            ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateAdvancedInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateAdvancedOutput>(),
                ),
                export: ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateOutput>(),
                ),
                export: ACCOUNT_CREATE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_SECURIFY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountSecurifyInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountSecurifyOutput>(),
                ),
                export: ACCOUNT_SECURIFY_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_FEE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountLockFeeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountLockFeeOutput>(),
                ),
                export: ACCOUNT_LOCK_FEE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountLockContingentFeeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountLockContingentFeeOutput>(),
                ),
                export: ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_DEPOSIT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountDepositInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountDepositOutput>(),
                ),
                export: ACCOUNT_DEPOSIT_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_DEPOSIT_BATCH_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountDepositBatchInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountDepositBatchOutput>(),
                ),
                export: ACCOUNT_DEPOSIT_BATCH_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_WITHDRAW_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountWithdrawInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountWithdrawOutput>(),
                ),
                export: ACCOUNT_WITHDRAW_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountWithdrawNonFungiblesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountWithdrawNonFungiblesOutput>(),
                ),
                export: ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_BURN_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountBurnInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountBurnOutput>(),
                ),
                export: ACCOUNT_BURN_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_BURN_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountBurnNonFungiblesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountBurnNonFungiblesOutput>(),
                ),
                export: ACCOUNT_BURN_NON_FUNGIBLES_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountLockFeeAndWithdrawInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountLockFeeAndWithdrawOutput>(),
                ),
                export: ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccountLockFeeAndWithdrawNonFungiblesInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccountLockFeeAndWithdrawNonFungiblesOutput>(
                    )),
                export: ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateProofOfAmountInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateProofOfAmountOutput>(),
                ),
                export: ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountCreateProofOfNonFungiblesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountCreateProofOfNonFungiblesOutput>(),
                ),
                export: ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountSetDefaultDepositRuleInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountSetDefaultDepositRuleOutput>(),
                ),
                export: ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountSetResourcePreferenceInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountSetResourcePreferenceOutput>(),
                ),
                export: ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_REMOVE_RESOURCE_PREFERENCE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountRemoveResourcePreferenceInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountRemoveResourcePreferenceOutput>(),
                ),
                export: ACCOUNT_REMOVE_RESOURCE_PREFERENCE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountTryDepositOrRefundInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountTryDepositOrRefundOutput>(),
                ),
                export: ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountTryDepositBatchOrRefundInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountTryDepositBatchOrRefundOutput>(),
                ),
                export: ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountTryDepositOrAbortInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountTryDepositOrAbortOutput>(),
                ),
                export: ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountTryDepositBatchOrAbortInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountTryDepositBatchOrAbortOutput>(),
                ),
                export: ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountAddAuthorizedDepositorInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountAddAuthorizedDepositorOutput>(),
                ),
                export: ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_REMOVE_AUTHORIZED_DEPOSITOR_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountRemoveAuthorizedDepositorInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountRemoveAuthorizedDepositorOutput>(),
                ),
                export: ACCOUNT_REMOVE_AUTHORIZED_DEPOSITOR_IDENT.to_string(),
            },
        );

        let events = event_schema! {
            aggregator,
            [
                WithdrawEvent,
                DepositEvent,
                RejectedDepositEvent,
                SetResourcePreferenceEvent,
                RemoveResourcePreferenceEvent,
                SetDefaultDepositRuleEvent,
                AddAuthorizedDepositorEvent,
                RemoveAuthorizedDepositorEvent,
            ]
        };

        let schema = generate_full_schema(aggregator);

        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::default(),
            is_transient: false,
            feature_set,
            dependencies: indexset!(
                SECP256K1_SIGNATURE_RESOURCE.into(),
                ED25519_SIGNATURE_RESOURCE.into(),
                ACCOUNT_OWNER_BADGE.into(),
                PACKAGE_OF_DIRECT_CALLER_RESOURCE.into(),
            ),

            schema: BlueprintSchemaInit {
                generics: vec![],
                schema,
                state,
                events,
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit { functions },
                hooks: BlueprintHooksInit {
                    hooks: indexmap!(BlueprintHook::OnVirtualize => ACCOUNT_ON_VIRTUALIZE_EXPORT_NAME.to_string()),
                },
            },

            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig {
                function_auth: FunctionAuth::AllowAll,
                method_auth: MethodAuthTemplate::StaticRoleDefinition(roles_template!(
                    roles {
                        SECURIFY_ROLE => updaters: [SELF_ROLE];
                    },
                    methods {
                        ACCOUNT_SECURIFY_IDENT => [SECURIFY_ROLE];

                        ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT => [OWNER_ROLE];
                        ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT => [OWNER_ROLE];
                        ACCOUNT_REMOVE_RESOURCE_PREFERENCE_IDENT => [OWNER_ROLE];
                        ACCOUNT_WITHDRAW_IDENT => [OWNER_ROLE];
                        ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT => [OWNER_ROLE];
                        ACCOUNT_LOCK_FEE_IDENT => [OWNER_ROLE];
                        ACCOUNT_LOCK_CONTINGENT_FEE_IDENT => [OWNER_ROLE];
                        ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT => [OWNER_ROLE];
                        ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT => [OWNER_ROLE];
                        ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT => [OWNER_ROLE];
                        ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT => [OWNER_ROLE];
                        ACCOUNT_DEPOSIT_IDENT => [OWNER_ROLE];
                        ACCOUNT_DEPOSIT_BATCH_IDENT => [OWNER_ROLE];
                        ACCOUNT_BURN_IDENT => [OWNER_ROLE];
                        ACCOUNT_BURN_NON_FUNGIBLES_IDENT => [OWNER_ROLE];
                        ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT => [OWNER_ROLE];
                        ACCOUNT_REMOVE_AUTHORIZED_DEPOSITOR_IDENT => [OWNER_ROLE];

                        ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT => MethodAccessibility::Public;
                        ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT => MethodAccessibility::Public;
                        ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT => MethodAccessibility::Public;
                        ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT => MethodAccessibility::Public;
                    }
                )),
            },
        }
    }

    fn create_modules<Y: SystemApi<RuntimeError>>(
        role_assignment: RoleAssignment,
        metadata_init: MetadataInit,
        api: &mut Y,
    ) -> Result<IndexMap<AttachedModuleId, Own>, RuntimeError> {
        let metadata = Metadata::create_with_data(metadata_init, api)?;

        // No component royalties
        let modules = indexmap!(
            AttachedModuleId::RoleAssignment => role_assignment.0,
            AttachedModuleId::Metadata => metadata,
        );

        Ok(modules)
    }

    pub fn on_virtualize<Y: SystemApi<RuntimeError>>(
        input: OnVirtualizeInput,
        api: &mut Y,
    ) -> Result<OnVirtualizeOutput, RuntimeError> {
        match input.variant_id {
            ACCOUNT_CREATE_PREALLOCATED_SECP256K1_ID => {
                let public_key_hash = PublicKeyHash::Secp256k1(Secp256k1PublicKeyHash(input.rid));
                Self::create_virtual(public_key_hash, input.address_reservation, api)
            }
            ACCOUNT_CREATE_PREALLOCATED_ED25519_ID => {
                let public_key_hash = PublicKeyHash::Ed25519(Ed25519PublicKeyHash(input.rid));
                Self::create_virtual(public_key_hash, input.address_reservation, api)
            }
            x => Err(RuntimeError::ApplicationError(
                ApplicationError::PanicMessage(format!("Unexpected variant id: {:?}", x)),
            )),
        }
    }

    fn create_virtual<Y: SystemApi<RuntimeError>>(
        public_key_hash: PublicKeyHash,
        address_reservation: GlobalAddressReservation,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let owner_badge = {
            let bytes = public_key_hash.get_hash_bytes();
            let entity_type = match public_key_hash {
                PublicKeyHash::Ed25519(..) => EntityType::GlobalPreallocatedEd25519Account,
                PublicKeyHash::Secp256k1(..) => EntityType::GlobalPreallocatedSecp256k1Account,
            };

            let mut id_bytes = vec![entity_type as u8];
            id_bytes.extend(bytes);

            NonFungibleLocalId::bytes(id_bytes).unwrap()
        };

        let account = Self::create_local(api)?;
        let owner_id = NonFungibleGlobalId::from_public_key_hash(public_key_hash);
        let role_assignment = SecurifiedAccount::create_presecurified(owner_id, api)?;
        let modules = Self::create_modules(
            role_assignment,
            metadata_init!(
                // NOTE:
                // This is the owner key for ROLA. We choose to set this explicitly to simplify the
                // security-critical logic off-ledger. In particular, we want an owner to be able to
                // explicitly delete the owner keys. If we went with a "no metadata = assume default
                // public key hash", then this could cause unexpected security-critical behavior if
                // a user expected that deleting the metadata removed the owner keys.
                "owner_keys" => vec![public_key_hash], updatable;
                "owner_badge" => owner_badge, locked;
            ),
            api,
        )?;

        api.globalize(
            account.0,
            modules.into_iter().map(|(k, v)| (k, v.0)).collect(),
            Some(address_reservation),
        )?;
        Ok(())
    }

    pub fn securify<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<Bucket, RuntimeError> {
        let receiver = Runtime::get_node_id(api)?;
        let owner_badge_data = AccountOwnerBadgeData {
            name: "Account Owner Badge".into(),
            account: ComponentAddress::new_or_panic(receiver.0),
        };
        let bucket = SecurifiedAccount::securify(
            &receiver,
            owner_badge_data,
            Some(NonFungibleLocalId::bytes(receiver.0).unwrap()),
            api,
        )?;
        Ok(bucket.into())
    }

    pub fn create_advanced<Y: SystemApi<RuntimeError>>(
        owner_role: OwnerRole,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<GlobalAddress, RuntimeError> {
        let account = Self::create_local(api)?;
        let role_assignment = SecurifiedAccount::create_advanced(owner_role, api)?;
        let modules = Self::create_modules(
            role_assignment,
            metadata_init!(
                "owner_badge" => EMPTY, locked;
            ),
            api,
        )?;
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();

        let address = api.globalize(account.0, modules, address_reservation)?;

        Ok(address)
    }

    pub fn create<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<(GlobalAddress, Bucket), RuntimeError> {
        let (address_reservation, address) = api.allocate_global_address(BlueprintId {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
        })?;

        let account = Self::create_local(api)?;
        let (role_assignment, bucket) = SecurifiedAccount::create_securified(
            AccountOwnerBadgeData {
                name: "Account Owner Badge".into(),
                account: address.try_into().expect("Impossible Case"),
            },
            Some(NonFungibleLocalId::bytes(address.as_node_id().0).unwrap()),
            api,
        )?;
        let modules = Self::create_modules(
            role_assignment,
            metadata_init! {
                "owner_badge" => NonFungibleLocalId::bytes(address.as_node_id().0).unwrap(), locked;
            },
            api,
        )?;
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();

        let address = api.globalize(account.0, modules, Some(address_reservation))?;

        Ok((address, bucket))
    }

    fn create_local<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<Own, RuntimeError> {
        let account_id = api.new_object(
            ACCOUNT_BLUEPRINT,
            vec![],
            GenericArgs::default(),
            indexmap! {
                AccountField::DepositRule.field_index() => FieldValue::new(&AccountDepositRuleFieldPayload::from_content_source(AccountDepositRuleV1 {
                    default_deposit_rule: DefaultDepositRule::Accept,
                }))
            },
            indexmap!(),
        )?;

        Ok(Own(account_id))
    }

    fn lock_fee_internal<Y: SystemApi<RuntimeError>>(
        amount: Decimal,
        contingent: bool,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let resource_address = XRD;

        Self::get_vault(
            resource_address,
            |vault, api| {
                if contingent {
                    vault.lock_contingent_fee(api, amount)
                } else {
                    vault.lock_fee(api, amount)
                }
            },
            false,
            api,
        )?;

        Ok(())
    }

    pub fn lock_fee<Y: SystemApi<RuntimeError>>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::lock_fee_internal(amount, false, api)?;
        Ok(())
    }

    pub fn lock_contingent_fee<Y: SystemApi<RuntimeError>>(
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::lock_fee_internal(amount, true, api)?;
        Ok(())
    }

    /// Method requires auth - if call goes through it performs the deposit with no questions asked
    pub fn deposit<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let resource_address = bucket.resource_address(api)?;
        let event = if resource_address.is_fungible() {
            DepositEvent::Fungible(resource_address, bucket.amount(api)?)
        } else {
            DepositEvent::NonFungible(resource_address, bucket.non_fungible_local_ids(api)?)
        };
        Self::get_vault(
            resource_address,
            |vault, api| vault.put(bucket, api),
            true,
            api,
        )?;
        Runtime::emit_event(api, event)?;
        Ok(())
    }

    /// Method requires auth - if call goes through it performs the deposit with no questions asked
    pub fn deposit_batch<Y: SystemApi<RuntimeError>>(
        buckets: Vec<Bucket>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        for bucket in buckets {
            Self::deposit(bucket, api)?;
        }
        Ok(())
    }

    pub fn try_deposit_or_refund<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        authorized_depositor_badge: Option<ResourceOrNonFungible>,
        api: &mut Y,
    ) -> Result<Option<Bucket>, RuntimeError> {
        let resource_address = bucket.resource_address(api)?;
        let is_deposit_allowed = Self::is_deposit_allowed(&resource_address, api)?;
        if is_deposit_allowed {
            Self::deposit(bucket, api)?;
            Ok(None)
        } else if let Some(badge) = authorized_depositor_badge {
            Self::validate_badge_is_authorized_depositor(&badge, api)??;
            Self::validate_badge_is_present(badge, api)?;
            Self::deposit(bucket, api)?;
            Ok(None)
        } else {
            let event = if resource_address.is_fungible() {
                RejectedDepositEvent::Fungible(resource_address, bucket.amount(api)?)
            } else {
                RejectedDepositEvent::NonFungible(
                    resource_address,
                    bucket.non_fungible_local_ids(api)?,
                )
            };
            Runtime::emit_event(api, event)?;
            Ok(Some(bucket))
        }
    }

    pub fn try_deposit_batch_or_refund<Y: SystemApi<RuntimeError>>(
        buckets: Vec<Bucket>,
        authorized_depositor_badge: Option<ResourceOrNonFungible>,
        api: &mut Y,
    ) -> Result<Option<Vec<Bucket>>, RuntimeError> {
        let offending_buckets = buckets
            .iter()
            .map(|bucket| {
                bucket
                    .resource_address(api)
                    .and_then(|resource_address| Self::is_deposit_allowed(&resource_address, api))
                    .map(|can_be_deposited| (bucket, can_be_deposited))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter_map(|(bucket, can_be_deposited)| {
                if !can_be_deposited {
                    Some(Bucket(bucket.0))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if offending_buckets.is_empty() {
            Self::deposit_batch(buckets, api)?;
            Ok(None)
        } else if let Some(badge) = authorized_depositor_badge {
            Self::validate_badge_is_authorized_depositor(&badge, api)??;
            Self::validate_badge_is_present(badge, api)?;
            Self::deposit_batch(buckets, api)?;
            Ok(None)
        } else {
            for bucket in offending_buckets {
                let resource_address = bucket.resource_address(api)?;
                let event = if resource_address.is_fungible() {
                    RejectedDepositEvent::Fungible(resource_address, bucket.amount(api)?)
                } else {
                    RejectedDepositEvent::NonFungible(
                        resource_address,
                        bucket.non_fungible_local_ids(api)?,
                    )
                };
                Runtime::emit_event(api, event)?;
            }
            Ok(Some(buckets))
        }
    }

    pub fn try_deposit_or_abort<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        authorized_depositor_badge: Option<ResourceOrNonFungible>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        if let Some(bucket) = Self::try_deposit_or_refund(bucket, authorized_depositor_badge, api)?
        {
            let resource_address = bucket.resource_address(api)?;
            Err(AccountError::DepositIsDisallowed { resource_address }.into())
        } else {
            Ok(())
        }
    }

    /// Method is public to all - if ANY of the resources can't be deposited then the execution
    /// panics.
    pub fn try_deposit_batch_or_abort<Y: SystemApi<RuntimeError>>(
        buckets: Vec<Bucket>,
        authorized_depositor_badge: Option<ResourceOrNonFungible>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let buckets = Self::try_deposit_batch_or_refund(buckets, authorized_depositor_badge, api)?;
        if let Some(_) = buckets {
            Err(AccountError::NotAllBucketsCouldBeDeposited.into())
        } else {
            Ok(())
        }
    }

    // Returns a result of a result. The outer result's error type is [`RuntimeError`] and it's for
    // cases when something about the process fails, e.g., reading the KVStore fails for some reason
    // or other cases. The inner result is for whether the validation succeeded or not.
    fn validate_badge_is_authorized_depositor<Y: SystemApi<RuntimeError>>(
        badge: &ResourceOrNonFungible,
        api: &mut Y,
    ) -> Result<Result<(), AccountError>, RuntimeError> {
        // Read the account's authorized depositors to ensure that this badge is on the list of
        // permitted depositors
        let encoded_key =
            scrypto_encode(badge).expect("Failed to SBOR encode a `ResourceOrNonFungible`.");
        let kv_store_entry_lock_handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            AccountCollection::AuthorizedDepositorKeyValue.collection_index(),
            &encoded_key,
            LockFlags::read_only(),
        )?;
        let entry = api.key_value_entry_get_typed::<VersionedAccountAuthorizedDepositor>(
            kv_store_entry_lock_handle,
        )?;
        api.key_value_entry_close(kv_store_entry_lock_handle)?;
        if entry.is_none() {
            Ok(Err(AccountError::NotAnAuthorizedDepositor {
                depositor: badge.clone(),
            }))
        } else {
            Ok(Ok(()))
        }
    }

    fn validate_badge_is_present<Y: SystemApi<RuntimeError>>(
        badge: ResourceOrNonFungible,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        // At this point we know that the badge is in the set of allowed depositors, so, we create
        // an access rule and assert against it.
        let access_rule = AccessRule::Protected(CompositeRequirement::BasicRequirement(
            BasicRequirement::Require(badge),
        ));

        Runtime::assert_access_rule(access_rule, api)?;
        Ok(())
    }

    pub fn withdraw<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let bucket = Self::get_vault(
            resource_address,
            |vault, api| vault.take(amount, api),
            false,
            api,
        )?;
        let event = if resource_address.is_fungible() {
            WithdrawEvent::Fungible(resource_address, bucket.amount(api)?)
        } else {
            WithdrawEvent::NonFungible(resource_address, bucket.non_fungible_local_ids(api)?)
        };
        Runtime::emit_event(api, event)?;

        Ok(bucket)
    }

    pub fn withdraw_non_fungibles<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let bucket = Self::get_vault(
            resource_address,
            |vault, api| vault.take_non_fungibles(ids, api),
            false,
            api,
        )?;
        let event =
            WithdrawEvent::NonFungible(resource_address, bucket.non_fungible_local_ids(api)?);
        Runtime::emit_event(api, event)?;

        Ok(bucket)
    }

    pub fn burn<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::get_vault(
            resource_address,
            |vault, api| vault.burn(amount, api),
            false,
            api,
        )
    }

    pub fn burn_non_fungibles<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::get_vault(
            resource_address,
            |vault, api| vault.burn_non_fungibles(ids, api),
            false,
            api,
        )
    }

    pub fn lock_fee_and_withdraw<Y: SystemApi<RuntimeError>>(
        amount_to_lock: Decimal,
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Self::lock_fee_internal(amount_to_lock, false, api)?;

        let bucket = Self::get_vault(
            resource_address,
            |vault, api| vault.take(amount, api),
            false,
            api,
        )?;

        Ok(bucket)
    }

    pub fn lock_fee_and_withdraw_non_fungibles<Y: SystemApi<RuntimeError>>(
        amount_to_lock: Decimal,
        resource_address: ResourceAddress,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Self::lock_fee_internal(amount_to_lock, false, api)?;

        let bucket = Self::get_vault(
            resource_address,
            |vault, api| vault.take_non_fungibles(ids, api),
            false,
            api,
        )?;

        Ok(bucket)
    }

    pub fn create_proof_of_amount<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError> {
        let proof = Self::get_vault(
            resource_address,
            |vault, api| vault.create_proof_of_amount(amount, api),
            false,
            api,
        )?;

        Ok(proof)
    }

    pub fn create_proof_of_non_fungibles<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError> {
        let proof = Self::get_vault(
            resource_address,
            |vault, api| vault.create_proof_of_non_fungibles(ids, api),
            false,
            api,
        )?;

        Ok(proof)
    }

    pub fn set_default_deposit_rule<Y: SystemApi<RuntimeError>>(
        default: DefaultDepositRule,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            AccountField::DepositRule.field_index(),
            LockFlags::MUTABLE,
        )?;
        api.field_write_typed(
            handle,
            &AccountDepositRuleFieldPayload::from_content_source(AccountDepositRuleV1 {
                default_deposit_rule: default,
            }),
        )?;
        api.field_close(handle)?;

        Runtime::emit_event(
            api,
            SetDefaultDepositRuleEvent {
                default_deposit_rule: default,
            },
        )?;

        Ok(())
    }

    pub fn set_resource_preference<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        resource_preference: ResourcePreference,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");
        let kv_store_entry_lock_handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            AccountCollection::ResourcePreferenceKeyValue.collection_index(),
            &encoded_key,
            LockFlags::MUTABLE,
        )?;
        api.key_value_entry_set_typed(
            kv_store_entry_lock_handle,
            &AccountResourcePreferenceVersions::V1(resource_preference).into_versioned(),
        )?;
        api.key_value_entry_close(kv_store_entry_lock_handle)?;

        Runtime::emit_event(
            api,
            SetResourcePreferenceEvent {
                resource_address,
                preference: resource_preference,
            },
        )?;

        Ok(())
    }

    pub fn remove_resource_preference<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");
        api.actor_remove_key_value_entry(
            ACTOR_STATE_SELF,
            AccountCollection::ResourcePreferenceKeyValue.collection_index(),
            &encoded_key,
        )?;

        Runtime::emit_event(api, RemoveResourcePreferenceEvent { resource_address })?;

        Ok(())
    }

    pub fn add_authorized_depositor<Y: SystemApi<RuntimeError>>(
        badge: ResourceOrNonFungible,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let encoded_key =
            scrypto_encode(&badge).expect("Failed to SBOR encode a `ResourceOrNonFungible`.");
        let kv_store_entry_lock_handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            AccountCollection::AuthorizedDepositorKeyValue.collection_index(),
            &encoded_key,
            LockFlags::MUTABLE,
        )?;
        api.key_value_entry_set_typed(
            kv_store_entry_lock_handle,
            &AccountAuthorizedDepositorEntryPayload::from_content_source(()),
        )?;
        api.key_value_entry_close(kv_store_entry_lock_handle)?;

        Runtime::emit_event(
            api,
            AddAuthorizedDepositorEvent {
                authorized_depositor_badge: badge,
            },
        )?;

        Ok(())
    }

    pub fn remove_authorized_depositor<Y: SystemApi<RuntimeError>>(
        badge: ResourceOrNonFungible,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let encoded_key =
            scrypto_encode(&badge).expect("Failed to SBOR encode a `ResourceOrNonFungible`.");
        api.actor_remove_key_value_entry(
            ACTOR_STATE_SELF,
            AccountCollection::AuthorizedDepositorKeyValue.collection_index(),
            &encoded_key,
        )?;

        Runtime::emit_event(
            api,
            RemoveAuthorizedDepositorEvent {
                authorized_depositor_badge: badge,
            },
        )?;

        Ok(())
    }

    fn get_default_deposit_rule<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<DefaultDepositRule, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            AccountField::DepositRule.field_index(),
            LockFlags::read_only(),
        )?;
        let deposit_rule = api
            .field_read_typed::<AccountDepositRuleFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        let default = deposit_rule.default_deposit_rule;
        api.field_close(handle)?;

        Ok(default)
    }

    fn get_vault<Y: SystemApi<RuntimeError>, R>(
        resource_address: ResourceAddress,
        vault_fn: impl FnOnce(&mut Vault, &mut Y) -> Result<R, RuntimeError>,
        create: bool,
        api: &mut Y,
    ) -> Result<R, RuntimeError> {
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let mut kv_store_entry_lock_handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            AccountCollection::ResourceVaultKeyValue.collection_index(),
            &encoded_key,
            LockFlags::read_only(),
        )?;

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then create it if
        // instructed to.
        let vault = {
            let entry = api
                .key_value_entry_get_typed::<AccountResourceVaultEntryPayload>(
                    kv_store_entry_lock_handle,
                )?
                .map(|v| v.fully_update_and_into_latest_version());

            match entry {
                Some(vault) => Ok(vault),
                None => {
                    if create {
                        api.key_value_entry_close(kv_store_entry_lock_handle)?;
                        kv_store_entry_lock_handle = api.actor_open_key_value_entry(
                            ACTOR_STATE_SELF,
                            AccountCollection::ResourceVaultKeyValue.collection_index(),
                            &encoded_key,
                            LockFlags::MUTABLE,
                        )?;
                        let vault = Vault::create(resource_address, api)?;
                        let own = vault.0;
                        api.key_value_entry_set_typed(
                            kv_store_entry_lock_handle,
                            &AccountResourceVaultEntryPayload::from_content_source(vault),
                        )?;
                        Ok(Vault(own))
                    } else {
                        Err(AccountError::VaultDoesNotExist { resource_address })
                    }
                }
            }
        };

        if let Ok(mut vault) = vault {
            match vault_fn(&mut vault, api) {
                Ok(rtn) => {
                    api.key_value_entry_close(kv_store_entry_lock_handle)?;
                    Ok(rtn)
                }
                Err(error) => Err(error),
            }
        } else {
            api.key_value_entry_close(kv_store_entry_lock_handle)?;
            Err(vault.unwrap_err().into())
        }
    }

    fn is_deposit_allowed<Y: SystemApi<RuntimeError>>(
        resource_address: &ResourceAddress,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match Self::get_resource_preference(resource_address, api)? {
            Some(ResourcePreference::Allowed) => Ok(true),
            Some(ResourcePreference::Disallowed) => Ok(false),
            None => {
                let default = Self::get_default_deposit_rule(api)?;
                match default {
                    DefaultDepositRule::Accept => Ok(true),
                    DefaultDepositRule::Reject => Ok(false),
                    DefaultDepositRule::AllowExisting => {
                        Ok(*resource_address == XRD
                            || Self::does_vault_exist(resource_address, api)?)
                    }
                }
            }
        }
    }

    fn does_vault_exist<Y: SystemApi<RuntimeError>>(
        resource_address: &ResourceAddress,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        let encoded_key = scrypto_encode(resource_address).expect("Impossible Case!");

        let kv_store_entry_lock_handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            AccountCollection::ResourceVaultKeyValue.collection_index(),
            &encoded_key,
            LockFlags::read_only(),
        )?;

        let does_vault_exist = {
            let entry = api.key_value_entry_get_typed::<AccountResourceVaultEntryPayload>(
                kv_store_entry_lock_handle,
            )?;
            entry.is_some()
        };

        api.key_value_entry_close(kv_store_entry_lock_handle)?;

        Ok(does_vault_exist)
    }

    fn get_resource_preference<Y: SystemApi<RuntimeError>>(
        resource_address: &ResourceAddress,
        api: &mut Y,
    ) -> Result<Option<ResourcePreference>, RuntimeError> {
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let kv_store_entry_lock_handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            AccountCollection::ResourcePreferenceKeyValue.collection_index(),
            &encoded_key,
            LockFlags::read_only(),
        )?;

        let entry = api
            .key_value_entry_get_typed::<AccountResourcePreferenceEntryPayload>(
                kv_store_entry_lock_handle,
            )?
            .map(|v| v.fully_update_and_into_latest_version());
        api.key_value_entry_close(kv_store_entry_lock_handle)?;
        Ok(entry)
    }
}

#[derive(ScryptoSbor)]
pub struct AccountOwnerBadgeData {
    pub name: String,
    pub account: ComponentAddress,
}

impl NonFungibleData for AccountOwnerBadgeData {
    const MUTABLE_FIELDS: &'static [&'static str] = &[];
}

pub struct AccountBlueprintBottlenoseExtension;

impl AccountBlueprintBottlenoseExtension {
    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT => {
                let AccountTryDepositOrRefundInput {
                    bucket,
                    authorized_depositor_badge,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::try_deposit_or_refund(bucket, authorized_depositor_badge, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT => {
                let AccountTryDepositBatchOrRefundInput {
                    buckets,
                    authorized_depositor_badge,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn =
                    Self::try_deposit_batch_or_refund(buckets, authorized_depositor_badge, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    pub fn try_deposit_or_refund<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        authorized_depositor_badge: Option<ResourceOrNonFungible>,
        api: &mut Y,
    ) -> Result<Option<Bucket>, RuntimeError> {
        let resource_address = bucket.resource_address(api)?;
        let is_deposit_allowed = AccountBlueprint::is_deposit_allowed(&resource_address, api)?;
        if is_deposit_allowed {
            AccountBlueprint::deposit(bucket, api)?;
            Ok(None)
        } else if let Some(badge) = authorized_depositor_badge {
            match AccountBlueprint::validate_badge_is_authorized_depositor(&badge, api)? {
                // The passed authorized depositor badge is indeed an authorized depositor.
                Ok(_) => {}
                // The badge that they claim to be an authorized depositor is not one. Return the
                // resources back to them.
                Err(AccountError::NotAnAuthorizedDepositor { .. }) => {
                    let event = if resource_address.is_fungible() {
                        RejectedDepositEvent::Fungible(resource_address, bucket.amount(api)?)
                    } else {
                        RejectedDepositEvent::NonFungible(
                            resource_address,
                            bucket.non_fungible_local_ids(api)?,
                        )
                    };
                    Runtime::emit_event(api, event)?;
                    return Ok(Some(bucket));
                }
                // Some other account error is encountered - impossible case since the function
                // will not return it. In either way, we propagate it.
                Err(error) => return Err(error.into()),
            }
            AccountBlueprint::validate_badge_is_present(badge, api)?;
            AccountBlueprint::deposit(bucket, api)?;
            Ok(None)
        } else {
            let event = if resource_address.is_fungible() {
                RejectedDepositEvent::Fungible(resource_address, bucket.amount(api)?)
            } else {
                RejectedDepositEvent::NonFungible(
                    resource_address,
                    bucket.non_fungible_local_ids(api)?,
                )
            };
            Runtime::emit_event(api, event)?;
            Ok(Some(bucket))
        }
    }

    pub fn try_deposit_batch_or_refund<Y: SystemApi<RuntimeError>>(
        buckets: Vec<Bucket>,
        authorized_depositor_badge: Option<ResourceOrNonFungible>,
        api: &mut Y,
    ) -> Result<Option<Vec<Bucket>>, RuntimeError> {
        let offending_buckets = buckets
            .iter()
            .map(|bucket| {
                bucket
                    .resource_address(api)
                    .and_then(|resource_address| {
                        AccountBlueprint::is_deposit_allowed(&resource_address, api)
                    })
                    .map(|can_be_deposited| (bucket, can_be_deposited))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter_map(|(bucket, can_be_deposited)| {
                if !can_be_deposited {
                    Some(Bucket(bucket.0))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if offending_buckets.is_empty() {
            AccountBlueprint::deposit_batch(buckets, api)?;
            Ok(None)
        } else if let Some(badge) = authorized_depositor_badge {
            match AccountBlueprint::validate_badge_is_authorized_depositor(&badge, api)? {
                // The passed authorized depositor badge is indeed an authorized depositor.
                Ok(_) => {}
                // The badge that they claim to be an authorized depositor is not one. Return the
                // resources back to them.
                Err(AccountError::NotAnAuthorizedDepositor { .. }) => {
                    for bucket in offending_buckets {
                        let resource_address = bucket.resource_address(api)?;
                        let event = if resource_address.is_fungible() {
                            RejectedDepositEvent::Fungible(resource_address, bucket.amount(api)?)
                        } else {
                            RejectedDepositEvent::NonFungible(
                                resource_address,
                                bucket.non_fungible_local_ids(api)?,
                            )
                        };
                        Runtime::emit_event(api, event)?;
                    }
                    return Ok(Some(buckets));
                }
                // Some other account error is encountered - impossible case since the function
                // will not return it. In either way, we propagate it.
                Err(error) => return Err(error.into()),
            }
            AccountBlueprint::validate_badge_is_present(badge, api)?;
            AccountBlueprint::deposit_batch(buckets, api)?;
            Ok(None)
        } else {
            for bucket in offending_buckets {
                let resource_address = bucket.resource_address(api)?;
                let event = if resource_address.is_fungible() {
                    RejectedDepositEvent::Fungible(resource_address, bucket.amount(api)?)
                } else {
                    RejectedDepositEvent::NonFungible(
                        resource_address,
                        bucket.non_fungible_local_ids(api)?,
                    )
                };
                Runtime::emit_event(api, event)?;
            }
            Ok(Some(buckets))
        }
    }
}

pub struct AccountBlueprintCuttlefishExtension;

impl AccountBlueprintCuttlefishExtension {
    pub fn added_functions_schema() -> (
        IndexMap<String, FunctionSchemaInit>,
        VersionedSchema<ScryptoCustomSchema>,
    ) {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let mut functions = index_map_new();
        functions.insert(
            ACCOUNT_BALANCE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountBalanceInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountBalanceOutput>(),
                ),
                export: ACCOUNT_BALANCE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountNonFungibleLocalIdsInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountNonFungibleLocalIdsOutput>(),
                ),
                export: ACCOUNT_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_HAS_NON_FUNGIBLE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountHasNonFungibleInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountHasNonFungibleOutput>(),
                ),
                export: ACCOUNT_HAS_NON_FUNGIBLE_IDENT.to_string(),
            },
        );
        let schema = generate_full_schema(aggregator);
        (functions, schema)
    }

    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            ACCOUNT_BALANCE_IDENT => {
                let AccountBalanceInput { resource_address } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::balance(resource_address, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_NON_FUNGIBLE_LOCAL_IDS_IDENT => {
                let AccountNonFungibleLocalIdsInput {
                    resource_address,
                    limit,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::non_fungible_local_ids(resource_address, limit, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_HAS_NON_FUNGIBLE_IDENT => {
                let AccountHasNonFungibleInput {
                    resource_address,
                    local_id,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::has_non_fungible(resource_address, local_id, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    pub fn balance<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Decimal, RuntimeError> {
        match AccountBlueprint::get_vault(
            resource_address,
            |vault, api| vault.amount(api),
            false,
            api,
        ) {
            Ok(balance) => Ok(balance),
            Err(RuntimeError::ApplicationError(ApplicationError::AccountError(
                AccountError::VaultDoesNotExist { .. },
            ))) => Ok(Decimal::ZERO),
            Err(error) => Err(error),
        }
    }

    pub fn non_fungible_local_ids<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        limit: u32,
        api: &mut Y,
    ) -> Result<IndexSet<NonFungibleLocalId>, RuntimeError> {
        match AccountBlueprint::get_vault(
            resource_address,
            |vault, api| vault.non_fungible_local_ids(limit, api),
            false,
            api,
        ) {
            Ok(ids) => Ok(ids),
            Err(RuntimeError::ApplicationError(ApplicationError::AccountError(
                AccountError::VaultDoesNotExist { .. },
            ))) => Ok(Default::default()),
            Err(error) => Err(error),
        }
    }

    pub fn has_non_fungible<Y: SystemApi<RuntimeError>>(
        resource_address: ResourceAddress,
        local_id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match AccountBlueprint::get_vault(
            resource_address,
            |vault, api| vault.contains_non_fungible(local_id, api),
            false,
            api,
        ) {
            Ok(result) => Ok(result),
            Err(RuntimeError::ApplicationError(ApplicationError::AccountError(
                AccountError::VaultDoesNotExist { .. },
            ))) => Ok(false),
            Err(error) => Err(error),
        }
    }
}
