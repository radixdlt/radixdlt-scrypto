use crate::blueprints::consensus_manager::*;
use crate::blueprints::util::SecurifiedRoleAssignment;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::internal_prelude::*;
use crate::{event_schema, roles_template};
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::{
    AttachedModuleId, FieldValue, SystemApi, ACTOR_REF_GLOBAL, ACTOR_STATE_OUTER_OBJECT,
    ACTOR_STATE_SELF,
};
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, FunctionAuth, MethodAuthTemplate,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::UncheckedUrl;
use radix_engine_interface::{burn_roles, metadata_init, mint_roles, rule};
use radix_native_sdk::modules::metadata::Metadata;
use radix_native_sdk::resource::NativeVault;
use radix_native_sdk::resource::ResourceManager;
use radix_native_sdk::resource::{NativeBucket, NativeNonFungibleBucket};
use radix_native_sdk::runtime::Runtime;
use sbor::rust::mem;

use super::{
    ClaimXrdEvent, RegisterValidatorEvent, StakeEvent, UnregisterValidatorEvent, UnstakeEvent,
    UpdateAcceptingStakeDelegationStateEvent,
};

pub const VALIDATOR_PROTOCOL_VERSION_NAME_LEN: usize = 32;

/// A performance-driven limit on the number of simultaneously pending "delayed withdrawal"
/// operations on any validator's owner's stake units vault.
pub const OWNER_STAKE_UNITS_PENDING_WITHDRAWALS_LIMIT: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ValidatorSubstate {
    /// A key used internally for storage of registered validators sorted by their stake descending.
    /// It is only useful when the validator is registered and has non-zero stake - hence, the field
    /// is [`None`] otherwise.
    /// Note: in theory, this value could be always computed from the [`is_registered`] status and
    /// the amount stored in [`stake_xrd_vault_id`]; we simply keep it cached to simplify certain
    /// updates.
    pub sorted_key: Option<SortedKey>,

    /// This validator's public key.
    pub key: Secp256k1PublicKey,

    /// Whether this validator is currently interested in participating in the consensus.
    pub is_registered: bool,

    /// Whether this validator is currently accepting delegated stake or not
    pub accepts_delegated_stake: bool,

    /// A fraction of the effective emission amount which gets transferred to the validator's owner
    /// (by staking it and depositing the stake units to the [`locked_owner_stake_unit_vault_id`]).
    /// Note: it is a decimal factor, not a percentage (i.e. `0.015` means "1.5%" here).
    /// Note: it may be overridden by [`validator_fee_change_request`], if it contains a change
    /// which already became effective.
    pub validator_fee_factor: Decimal,

    /// The most recent request to change the [`validator_fee_factor`] (which requires a delay).
    /// Note: the value from this request will be used instead of [`validator_fee_factor`] if the
    /// request has already reached its effective epoch.
    /// Note: when another change is requested, the value from this (previous) one is moved to the
    /// [`validator_fee_factor`] - provided that it became already effective. Otherwise, this
    /// request is overwritten by the new one.
    pub validator_fee_change_request: Option<ValidatorFeeChangeRequest>,

    /// A type of fungible resource representing stake units specific to this validator.
    /// Conceptually, "staking to validator A" means "contributing to the validator's staking pool,
    /// and receiving the validator's stake units which act as the pool units for the staking pool".
    pub stake_unit_resource: ResourceAddress,

    /// A vault holding the XRDs currently staked to this validator.
    pub stake_xrd_vault_id: Own,

    /// A type of non-fungible token used as a receipt for unstaked stake units.
    /// Unstaking burns the SUs and inactivates the staked XRDs (i.e. moves it from the regular
    /// [`stake_xrd_vault_id`] to the [`pending_xrd_withdraw_vault_id`]), and then requires to claim
    /// the XRDs using this NFT after a delay (see [`UnstakeData.claim_epoch`]).
    pub claim_nft: ResourceAddress,

    /// A vault holding the XRDs that were unstaked (see the [`unstake_nft`]) but not yet claimed.
    pub pending_xrd_withdraw_vault_id: Own,

    /// A vault holding the SUs that this validator's owner voluntarily decided to temporarily lock
    /// here, as a public display of their confidence in this validator's future reliability.
    /// Withdrawing SUs from this vault is subject to a delay (which is configured separately from
    /// the regular unstaking delay, see [`ConsensusManagerConfigSubstate.num_owner_stake_units_unlock_epochs`]).
    /// This vault is private to the owner (i.e. the owner's badge is required for any interaction
    /// with this vault).
    pub locked_owner_stake_unit_vault_id: Own,

    /// A vault holding the SUs which the owner has decided to withdraw from their "public display"
    /// vault (see [`locked_owner_stake_unit_vault_id`]) but which have not yet been unlocked after
    /// the mandatory delay (see [`pending_owner_stake_unit_withdrawals`]).
    pub pending_owner_stake_unit_unlock_vault_id: Own,

    /// All currently pending "delayed withdrawal" operations of the owner's stake units vault (see
    /// [`locked_owner_stake_unit_vault_id`]).
    /// This maps an epoch number to an amount of stake units that become unlocked at that epoch.
    /// Note: because of performance considerations, a maximum size of this map is limited to
    /// [`OWNER_STAKE_UNITS_PENDING_WITHDRAWALS_LIMIT`]: starting another withdrawal will first
    /// attempt to move any already-available amount to [`already_unlocked_owner_stake_unit_amount`]
    /// and only then will fail if the limit is exceeded.
    pub pending_owner_stake_unit_withdrawals: BTreeMap<Epoch, Decimal>,

    /// An amount of owner's stake units that has already waited for a sufficient number of epochs
    /// in the [`pending_owner_stake_unit_withdrawals`] and was automatically moved from there.
    /// The very next [`finish_unlock_owner_stake_units()`] operation will release this amount.
    pub already_unlocked_owner_stake_unit_amount: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct ValidatorProtocolUpdateReadinessSignalSubstate {
    pub protocol_version_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct UnstakeData {
    pub name: String,

    /// An epoch number at (or after) which the pending unstaked XRD may be claimed.
    /// Note: on unstake, it is fixed to be [`ConsensusManagerConfigSubstate.num_unstake_epochs`] away.
    pub claim_epoch: Epoch,

    /// An XRD amount to be claimed.
    pub claim_amount: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ValidatorFeeChangeRequest {
    /// An epoch number at (or after) which the fee change is effective.
    /// To be specific: when a next epoch `N` begins, we perform accounting of emissions due for
    /// previous epoch `N-1` - this means that we will use this [`new_validator_fee_factor`] only if
    /// `epoch_effective <= N-1`, and [`ValidatorSubstate.validator_fee_factor`] otherwise.
    /// Note: when requesting a fee decrease, this will be "next epoch"; and when requesting an
    /// increase, this will be set to [`ConsensusManagerConfigSubstate.num_fee_increase_delay_epochs`]
    /// epochs away.
    pub epoch_effective: Epoch,

    /// A requested new value of [`ConsensusManagerSubstate.validator_fee_factor`].
    pub new_fee_factor: Decimal,
}

impl NonFungibleData for UnstakeData {
    const MUTABLE_FIELDS: &'static [&'static str] = &[];
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ValidatorError {
    InvalidClaimResource,
    InvalidGetRedemptionAmount,
    UnexpectedDecimalComputationError,
    EpochUnlockHasNotOccurredYet,
    PendingOwnerStakeWithdrawalLimitReached,
    InvalidValidatorFeeFactor,
    ValidatorIsNotAcceptingDelegatedStake,
    InvalidProtocolVersionNameLength { expected: usize, actual: usize },
    EpochMathOverflow,
}

declare_native_blueprint_state! {
    blueprint_ident: Validator,
    blueprint_snake_case: validator,
    features: {
    },
    fields: {
        state: {
            ident: State,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        },
        protocol_update_readiness_signal: {
            ident: ProtocolUpdateReadinessSignal,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        },
    },
    collections: {
    }
}

pub type ValidatorStateV1 = ValidatorSubstate;
pub type ValidatorProtocolUpdateReadinessSignalV1 = ValidatorProtocolUpdateReadinessSignalSubstate;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
enum UpdateSecondaryIndex {
    Create {
        index_key: SortedKey,
        key: Secp256k1PublicKey,
        stake: Decimal,
    },
    UpdateStake {
        index_key: SortedKey,
        new_index_key: SortedKey,
        new_stake_amount: Decimal,
    },
    UpdatePublicKey {
        index_key: SortedKey,
        key: Secp256k1PublicKey,
    },
    Remove {
        index_key: SortedKey,
    },
}

pub struct ValidatorBlueprint;

impl ValidatorBlueprint {
    pub fn definition() -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let feature_set = ValidatorFeatureSet::all_features();
        let state = ValidatorStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = index_map_new();
        functions.insert(
            VALIDATOR_REGISTER_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorRegisterInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorRegisterOutput>(),
                ),
                export: VALIDATOR_REGISTER_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_UNREGISTER_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorUnregisterInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorUnregisterOutput>(),
                ),
                export: VALIDATOR_UNREGISTER_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_STAKE_AS_OWNER_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorStakeAsOwnerInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorStakeAsOwnerOutput>(),
                ),
                export: VALIDATOR_STAKE_AS_OWNER_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_STAKE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorStakeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorStakeOutput>(),
                ),
                export: VALIDATOR_STAKE_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_UNSTAKE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorUnstakeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorUnstakeOutput>(),
                ),
                export: VALIDATOR_UNSTAKE_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_CLAIM_XRD_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorClaimXrdInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorClaimXrdOutput>(),
                ),
                export: VALIDATOR_CLAIM_XRD_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_UPDATE_KEY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorUpdateKeyInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorUpdateKeyOutput>(),
                ),
                export: VALIDATOR_UPDATE_KEY_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_UPDATE_FEE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorUpdateFeeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorUpdateFeeOutput>(),
                ),
                export: VALIDATOR_UPDATE_FEE_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<ValidatorUpdateAcceptDelegatedStakeInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<ValidatorUpdateAcceptDelegatedStakeOutput>()),
                export: VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ValidatorAcceptsDelegatedStakeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ValidatorAcceptsDelegatedStakeOutput>(),
                ),
                export: VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_TOTAL_STAKE_XRD_AMOUNT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ValidatorTotalStakeXrdAmountInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ValidatorTotalStakeXrdAmountOutput>(),
                ),
                export: VALIDATOR_TOTAL_STAKE_XRD_AMOUNT_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_TOTAL_STAKE_UNIT_SUPPLY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ValidatorTotalStakeUnitSupplyInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ValidatorTotalStakeUnitSupplyOutput>(),
                ),
                export: VALIDATOR_TOTAL_STAKE_UNIT_SUPPLY_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_GET_REDEMPTION_VALUE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorGetRedemptionValueInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ValidatorGetRedemptionValueOutput>(),
                ),
                export: VALIDATOR_GET_REDEMPTION_VALUE_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<ValidatorSignalProtocolUpdateReadinessInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<ValidatorSignalProtocolUpdateReadinessOutput>()),
                export: VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_GET_PROTOCOL_UPDATE_READINESS_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<ValidatorGetProtocolUpdateReadinessInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<ValidatorGetProtocolUpdateReadinessOutput>()),
                export: VALIDATOR_GET_PROTOCOL_UPDATE_READINESS_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ValidatorLockOwnerStakeUnitsInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<ValidatorLockOwnerStakeUnitsOutput>(),
                ),
                export: VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<ValidatorStartUnlockOwnerStakeUnitsInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<ValidatorStartUnlockOwnerStakeUnitsOutput>()),
                export: VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<ValidatorFinishUnlockOwnerStakeUnitsInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<ValidatorFinishUnlockOwnerStakeUnitsOutput>()),
                export: VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_APPLY_EMISSION_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorApplyEmissionInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorApplyEmissionOutput>(),
                ),
                export: VALIDATOR_APPLY_EMISSION_IDENT.to_string(),
            },
        );
        functions.insert(
            VALIDATOR_APPLY_REWARD_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorApplyRewardInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ValidatorApplyRewardOutput>(),
                ),
                export: VALIDATOR_APPLY_REWARD_IDENT.to_string(),
            },
        );

        let event_schema = event_schema! {
            aggregator,
            [
                RegisterValidatorEvent,
                UnregisterValidatorEvent,
                StakeEvent,
                UnstakeEvent,
                ClaimXrdEvent,
                ProtocolUpdateReadinessSignalEvent,
                UpdateAcceptingStakeDelegationStateEvent,
                ValidatorEmissionAppliedEvent,
                ValidatorRewardAppliedEvent
            ]
        };

        let schema = generate_full_schema(aggregator);

        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::Inner {
                outer_blueprint: CONSENSUS_MANAGER_BLUEPRINT.to_string(),
            },
            is_transient: false,
            feature_set,
            dependencies: indexset!(),
            schema: BlueprintSchemaInit {
                generics: vec![],
                schema,
                state,
                events: event_schema,
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit { functions },
                hooks: BlueprintHooksInit::default(),
            },
            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig {
                function_auth: FunctionAuth::AllowAll,
                method_auth: MethodAuthTemplate::StaticRoleDefinition(roles_template! {
                    methods {
                        VALIDATOR_UNSTAKE_IDENT => MethodAccessibility::Public;
                        VALIDATOR_CLAIM_XRD_IDENT => MethodAccessibility::Public;
                        VALIDATOR_STAKE_IDENT => MethodAccessibility::Public;
                        VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT => MethodAccessibility::Public;
                        VALIDATOR_TOTAL_STAKE_XRD_AMOUNT_IDENT => MethodAccessibility::Public;
                        VALIDATOR_TOTAL_STAKE_UNIT_SUPPLY_IDENT => MethodAccessibility::Public;
                        VALIDATOR_GET_REDEMPTION_VALUE_IDENT => MethodAccessibility::Public;
                        VALIDATOR_STAKE_AS_OWNER_IDENT => [OWNER_ROLE];
                        VALIDATOR_REGISTER_IDENT => [OWNER_ROLE];
                        VALIDATOR_UNREGISTER_IDENT => [OWNER_ROLE];
                        VALIDATOR_UPDATE_KEY_IDENT => [OWNER_ROLE];
                        VALIDATOR_UPDATE_FEE_IDENT => [OWNER_ROLE];
                        VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT => [OWNER_ROLE];
                        VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT => [OWNER_ROLE];
                        VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT => [OWNER_ROLE];
                        VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT => [OWNER_ROLE];
                        VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS_IDENT => [OWNER_ROLE];
                        VALIDATOR_GET_PROTOCOL_UPDATE_READINESS_IDENT => MethodAccessibility::OuterObjectOnly;
                        VALIDATOR_APPLY_EMISSION_IDENT => MethodAccessibility::OuterObjectOnly;
                        VALIDATOR_APPLY_REWARD_IDENT => MethodAccessibility::OuterObjectOnly;
                    }
                }),
            },
        }
    }

    pub fn register<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        Self::register_update(true, api)
    }

    pub fn unregister<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        Self::register_update(false, api)
    }

    pub fn stake_as_owner<Y: SystemApi<RuntimeError>>(
        xrd_bucket: Bucket,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Ok(Self::stake_internal(xrd_bucket, true, api)?.into())
    }

    pub fn stake<Y: SystemApi<RuntimeError>>(
        xrd_bucket: Bucket,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        Ok(Self::stake_internal(xrd_bucket, false, api)?.into())
    }

    fn stake_internal<Y: SystemApi<RuntimeError>>(
        xrd_bucket: Bucket,
        is_owner: bool,
        api: &mut Y,
    ) -> Result<FungibleBucket, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.field_index(),
            LockFlags::MUTABLE,
        )?;

        let mut validator = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        if !is_owner {
            if !validator.accepts_delegated_stake {
                api.field_close(handle)?;

                // TODO: Should this be an Option returned instead similar to Account?
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::ValidatorError(
                        ValidatorError::ValidatorIsNotAcceptingDelegatedStake,
                    ),
                ));
            }
        }

        let xrd_bucket_amount = xrd_bucket.amount(api)?;

        // Stake
        let (stake_unit_bucket, new_stake_amount) = {
            let mut stake_unit_resman = ResourceManager(validator.stake_unit_resource);
            let mut xrd_vault = Vault(validator.stake_xrd_vault_id);
            let stake_unit_mint_amount = Self::calculate_stake_unit_amount(
                xrd_bucket_amount,
                xrd_vault.amount(api)?,
                stake_unit_resman.total_supply(api)?.unwrap(),
            )?;

            let stake_unit_bucket = stake_unit_resman.mint_fungible(stake_unit_mint_amount, api)?;
            xrd_vault.put(xrd_bucket, api)?;
            let new_stake_amount = xrd_vault.amount(api)?;
            (stake_unit_bucket, new_stake_amount)
        };

        // Update ConsensusManager
        let new_index_key =
            Self::index_update(&validator, validator.is_registered, new_stake_amount, api)?;

        validator.sorted_key = new_index_key;
        api.field_write_typed(
            handle,
            &ValidatorStateFieldPayload::from_content_source(validator),
        )?;

        Runtime::emit_event(
            api,
            StakeEvent {
                xrd_staked: xrd_bucket_amount,
            },
        )?;

        Ok(stake_unit_bucket)
    }

    pub fn unstake<Y: SystemApi<RuntimeError>>(
        stake_unit_bucket: Bucket,
        api: &mut Y,
    ) -> Result<NonFungibleBucket, RuntimeError> {
        let stake_unit_bucket_amount = stake_unit_bucket.amount(api)?;

        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.field_index(),
            LockFlags::MUTABLE,
        )?;
        let mut validator_substate = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        // Unstake
        let (unstake_bucket, new_stake_amount) = {
            let xrd_amount = Self::calculate_redemption_value(
                stake_unit_bucket_amount,
                &validator_substate,
                api,
            )?;

            let mut stake_vault = Vault(validator_substate.stake_xrd_vault_id);
            let mut unstake_vault = Vault(validator_substate.pending_xrd_withdraw_vault_id);
            let nft_resman = ResourceManager(validator_substate.claim_nft);
            let mut stake_unit_resman = ResourceManager(validator_substate.stake_unit_resource);

            stake_unit_resman.burn(stake_unit_bucket, api)?;

            let manager_handle = api.actor_open_field(
                ACTOR_STATE_OUTER_OBJECT,
                ConsensusManagerField::State.into(),
                LockFlags::read_only(),
            )?;
            let manager_substate = api
                .field_read_typed::<ConsensusManagerStateFieldPayload>(manager_handle)?
                .fully_update_and_into_latest_version();
            let current_epoch = manager_substate.epoch;
            api.field_close(manager_handle)?;

            let config_handle = api.actor_open_field(
                ACTOR_STATE_OUTER_OBJECT,
                ConsensusManagerField::Configuration.into(),
                LockFlags::read_only(),
            )?;
            let config_substate = api
                .field_read_typed::<ConsensusManagerConfigurationFieldPayload>(config_handle)?
                .fully_update_and_into_latest_version();
            api.field_close(config_handle)?;

            let claim_epoch = current_epoch
                .after(config_substate.config.num_unstake_epochs)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::ValidatorError(ValidatorError::EpochMathOverflow),
                ))?;
            let data = UnstakeData {
                name: "Stake Claim".into(),
                claim_epoch,
                claim_amount: xrd_amount,
            };

            let bucket = stake_vault.take(xrd_amount, api)?;
            unstake_vault.put(bucket, api)?;
            let (unstake_bucket, _) = nft_resman.mint_non_fungible_single_ruid(data, api)?;

            let new_stake_amount = stake_vault.amount(api)?;

            (unstake_bucket, new_stake_amount)
        };

        // Update ConsensusManager
        let new_index_key = Self::index_update(
            &validator_substate,
            validator_substate.is_registered,
            new_stake_amount,
            api,
        )?;

        validator_substate.sorted_key = new_index_key;
        api.field_write_typed(
            handle,
            &ValidatorStateFieldPayload::from_content_source(validator_substate),
        )?;

        Runtime::emit_event(
            api,
            UnstakeEvent {
                stake_units: stake_unit_bucket_amount,
            },
        )?;

        Ok(unstake_bucket)
    }

    pub fn signal_protocol_update_readiness<Y: SystemApi<RuntimeError>>(
        protocol_version_name: String,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        if protocol_version_name.len() != VALIDATOR_PROTOCOL_VERSION_NAME_LEN {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ValidatorError(
                    ValidatorError::InvalidProtocolVersionNameLength {
                        expected: VALIDATOR_PROTOCOL_VERSION_NAME_LEN,
                        actual: protocol_version_name.len(),
                    },
                ),
            ));
        }

        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::ProtocolUpdateReadinessSignal.into(),
            LockFlags::MUTABLE,
        )?;
        let mut signal = api
            .field_read_typed::<ValidatorProtocolUpdateReadinessSignalFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        signal.protocol_version_name = Some(protocol_version_name.clone());
        api.field_write_typed(
            handle,
            &ValidatorProtocolUpdateReadinessSignalFieldPayload::from_content_source(signal),
        )?;
        api.field_close(handle)?;

        Runtime::emit_event(
            api,
            ProtocolUpdateReadinessSignalEvent {
                protocol_version_name,
            },
        )?;

        Ok(())
    }

    pub fn get_protocol_update_readiness<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Option<String>, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::ProtocolUpdateReadinessSignal.into(),
            LockFlags::read_only(),
        )?;
        let signal = api
            .field_read_typed::<ValidatorProtocolUpdateReadinessSignalFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        api.field_close(handle)?;

        Ok(signal.protocol_version_name)
    }

    fn register_update<Y: SystemApi<RuntimeError>>(
        new_registered: bool,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.field_index(),
            LockFlags::MUTABLE,
        )?;

        let mut validator = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        // No update
        if validator.is_registered == new_registered {
            return Ok(());
        }

        let stake_amount = {
            let stake_vault = Vault(validator.stake_xrd_vault_id);
            stake_vault.amount(api)?
        };

        let index_key = Self::index_update(&validator, new_registered, stake_amount, api)?;

        validator.is_registered = new_registered;
        validator.sorted_key = index_key;
        api.field_write_typed(
            handle,
            &ValidatorStateFieldPayload::from_content_source(validator),
        )?;

        if new_registered {
            Runtime::emit_event(api, RegisterValidatorEvent)?;
        } else {
            Runtime::emit_event(api, UnregisterValidatorEvent)?;
        }

        return Ok(());
    }

    fn index_update<Y: SystemApi<RuntimeError>>(
        validator: &ValidatorSubstate,
        new_registered: bool,
        new_stake_amount: Decimal,
        api: &mut Y,
    ) -> Result<Option<SortedKey>, RuntimeError> {
        let validator_address: ComponentAddress =
            ComponentAddress::new_or_panic(api.actor_get_node_id(ACTOR_REF_GLOBAL)?.into());
        let new_sorted_key =
            Self::to_sorted_key(new_registered, new_stake_amount, validator_address)?;

        let update = if let Some(cur_index_key) = &validator.sorted_key {
            if let Some(new_index_key) = &new_sorted_key {
                Some(UpdateSecondaryIndex::UpdateStake {
                    index_key: cur_index_key.clone(),
                    new_index_key: new_index_key.clone(),
                    new_stake_amount,
                })
            } else {
                Some(UpdateSecondaryIndex::Remove {
                    index_key: cur_index_key.clone(),
                })
            }
        } else {
            if let Some(new_index_key) = &new_sorted_key {
                Some(UpdateSecondaryIndex::Create {
                    index_key: new_index_key.clone(),
                    stake: new_stake_amount,
                    key: validator.key,
                })
            } else {
                None
            }
        };

        if let Some(update) = update {
            Self::update_validator(update, api)?;
        }

        Ok(new_sorted_key)
    }

    pub fn claim_xrd<Y: SystemApi<RuntimeError>>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.field_index(),
            LockFlags::read_only(),
        )?;
        let validator = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        let mut nft_resman = ResourceManager(validator.claim_nft);
        let resource_address = validator.claim_nft;
        let mut unstake_vault = Vault(validator.pending_xrd_withdraw_vault_id);

        if !resource_address.eq(&bucket.resource_address(api)?) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ValidatorError(ValidatorError::InvalidClaimResource),
            ));
        }

        let current_epoch = {
            let mgr_handle = api.actor_open_field(
                ACTOR_STATE_OUTER_OBJECT,
                ConsensusManagerField::State.field_index(),
                LockFlags::read_only(),
            )?;
            let mgr_substate = api
                .field_read_typed::<ConsensusManagerStateFieldPayload>(mgr_handle)?
                .fully_update_and_into_latest_version();
            let epoch = mgr_substate.epoch;
            api.field_close(mgr_handle)?;
            epoch
        };

        let mut unstake_amount = Decimal::zero();

        for id in bucket.non_fungible_local_ids(api)? {
            let data: UnstakeData = nft_resman.get_non_fungible_data(id, api)?;
            if current_epoch < data.claim_epoch {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::ValidatorError(ValidatorError::EpochUnlockHasNotOccurredYet),
                ));
            }
            unstake_amount = unstake_amount.checked_add(data.claim_amount).ok_or(
                RuntimeError::ApplicationError(ApplicationError::ValidatorError(
                    ValidatorError::UnexpectedDecimalComputationError,
                )),
            )?;
        }
        nft_resman.burn(bucket, api)?;

        let claimed_bucket = unstake_vault.take(unstake_amount, api)?;

        let amount = claimed_bucket.amount(api)?;
        Runtime::emit_event(
            api,
            ClaimXrdEvent {
                claimed_xrd: amount,
            },
        )?;

        Ok(claimed_bucket)
    }

    pub fn update_key<Y: SystemApi<RuntimeError>>(
        key: Secp256k1PublicKey,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.into(),
            LockFlags::MUTABLE,
        )?;
        let mut validator = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        // Update Consensus Manager
        {
            if let Some(index_key) = &validator.sorted_key {
                let update = UpdateSecondaryIndex::UpdatePublicKey {
                    index_key: index_key.clone(),
                    key,
                };

                Self::update_validator(update, api)?;
            }
        }

        validator.key = key;
        api.field_write_typed(
            handle,
            &ValidatorStateFieldPayload::from_content_source(validator),
        )?;

        Ok(())
    }

    pub fn update_fee<Y: SystemApi<RuntimeError>>(
        new_fee_factor: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        // check if new fee is valid
        check_validator_fee_factor(new_fee_factor)?;

        // read the current epoch
        let consensus_manager_handle = api.actor_open_field(
            ACTOR_STATE_OUTER_OBJECT,
            ConsensusManagerField::State.into(),
            LockFlags::read_only(),
        )?;
        let consensus_manager = api
            .field_read_typed::<ConsensusManagerStateFieldPayload>(consensus_manager_handle)?
            .fully_update_and_into_latest_version();
        let current_epoch = consensus_manager.epoch;
        api.field_close(consensus_manager_handle)?;

        // read the configured fee increase epochs delay
        let config_handle = api.actor_open_field(
            ACTOR_STATE_OUTER_OBJECT,
            ConsensusManagerField::Configuration.into(),
            LockFlags::read_only(),
        )?;
        let config_substate = api
            .field_read_typed::<ConsensusManagerConfigurationFieldPayload>(config_handle)?
            .fully_update_and_into_latest_version();
        api.field_close(config_handle)?;

        // begin the read+modify+write of the validator substate...
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        // - promote any currently pending change if it became effective already
        if let Some(previous_request) = substate.validator_fee_change_request {
            if previous_request.epoch_effective <= current_epoch {
                substate.validator_fee_factor = previous_request.new_fee_factor;
            }
        }

        // - calculate the effective epoch of the requested change
        let epoch_effective = if new_fee_factor > substate.validator_fee_factor {
            current_epoch.after(config_substate.config.num_fee_increase_delay_epochs)
        } else {
            current_epoch.next() // make it effective on the *beginning* of next epoch
        }
        .ok_or(RuntimeError::ApplicationError(
            ApplicationError::ValidatorError(ValidatorError::EpochMathOverflow),
        ))?;

        // ...end the read+modify+write of the validator substate
        substate.validator_fee_change_request = Some(ValidatorFeeChangeRequest {
            epoch_effective,
            new_fee_factor,
        });
        api.field_write_typed(
            handle,
            &ValidatorStateFieldPayload::from_content_source(substate),
        )?;
        api.field_close(handle)?;

        Ok(())
    }

    pub fn accepts_delegated_stake<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.into(),
            LockFlags::read_only(),
        )?;

        let substate = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        api.field_close(handle)?;

        Ok(substate.accepts_delegated_stake)
    }

    pub fn total_stake_xrd_amount<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Decimal, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.into(),
            LockFlags::read_only(),
        )?;

        let substate = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        let stake_vault = Vault(substate.stake_xrd_vault_id);
        let stake_amount = stake_vault.amount(api)?;
        api.field_close(handle)?;

        Ok(stake_amount)
    }

    pub fn total_stake_unit_supply<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Decimal, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.into(),
            LockFlags::read_only(),
        )?;

        let substate = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        let stake_resource = ResourceManager(substate.stake_unit_resource);
        let total_stake_unit_supply = stake_resource.total_supply(api)?.unwrap();
        api.field_close(handle)?;

        Ok(total_stake_unit_supply)
    }

    pub fn get_redemption_value<Y: SystemApi<RuntimeError>>(
        amount_of_stake_units: Decimal,
        api: &mut Y,
    ) -> Result<Decimal, RuntimeError> {
        if amount_of_stake_units.is_negative() || amount_of_stake_units.is_zero() {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ValidatorError(ValidatorError::InvalidGetRedemptionAmount),
            ));
        }

        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.into(),
            LockFlags::read_only(),
        )?;
        let validator = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        {
            let stake_unit_resman = ResourceManager(validator.stake_unit_resource);
            let total_stake_unit_supply = stake_unit_resman.total_supply(api)?.unwrap();
            if amount_of_stake_units > total_stake_unit_supply {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::ValidatorError(ValidatorError::InvalidGetRedemptionAmount),
                ));
            }
        }

        let redemption_value =
            Self::calculate_redemption_value(amount_of_stake_units, &validator, api)?;
        api.field_close(handle)?;

        Ok(redemption_value)
    }

    pub fn update_accept_delegated_stake<Y: SystemApi<RuntimeError>>(
        accept_delegated_stake: bool,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        substate.accepts_delegated_stake = accept_delegated_stake;
        api.field_write_typed(
            handle,
            &ValidatorStateFieldPayload::from_content_source(substate),
        )?;
        api.field_close(handle)?;

        Runtime::emit_event(
            api,
            UpdateAcceptingStakeDelegationStateEvent {
                accepts_delegation: accept_delegated_stake,
            },
        )?;

        Ok(())
    }

    /// Locks the given stake units in an internal "delayed withdrawal" vault (which is the owner's
    /// way of showing their commitment to running this validator in an orderly fashion - see
    /// [`ValidatorSubstate.locked_owner_stake_unit_vault_id`]).
    pub fn lock_owner_stake_units<Y: SystemApi<RuntimeError>>(
        stake_unit_bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.into(),
            LockFlags::read_only(),
        )?;
        let substate = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        Vault(substate.locked_owner_stake_unit_vault_id).put(stake_unit_bucket, api)?;

        api.field_close(handle)?;
        Ok(())
    }

    /// Starts the process of unlocking the owner's stake units stored in the internal vault.
    /// The requested amount of stake units (if available) will be ready for withdrawal after the
    /// network-configured [`ConsensusManagerConfigSubstate.num_owner_stake_units_unlock_epochs`] via a
    /// call to [`finish_unlock_owner_stake_units()`].
    pub fn start_unlock_owner_stake_units<Y: SystemApi<RuntimeError>>(
        requested_stake_unit_amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        // read the current epoch (needed for a drive-by "finish unlocking" of available withdrawals)
        let consensus_manager_handle = api.actor_open_field(
            ACTOR_STATE_OUTER_OBJECT,
            ConsensusManagerField::State.into(),
            LockFlags::read_only(),
        )?;
        let consensus_manager = api
            .field_read_typed::<ConsensusManagerStateFieldPayload>(consensus_manager_handle)?
            .fully_update_and_into_latest_version();
        let current_epoch = consensus_manager.epoch;
        api.field_close(consensus_manager_handle)?;

        // read the configured unlock epochs delay
        let config_handle = api.actor_open_field(
            ACTOR_STATE_OUTER_OBJECT,
            ConsensusManagerField::Configuration.into(),
            LockFlags::read_only(),
        )?;
        let config_substate = api
            .field_read_typed::<ConsensusManagerConfigurationFieldPayload>(config_handle)?
            .fully_update_and_into_latest_version();
        api.field_close(config_handle)?;

        // begin the read+modify+write of the validator substate...
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        // - move the already-available withdrawals to a dedicated field
        Self::normalize_available_owner_stake_unit_withdrawals(&mut substate, current_epoch)?;

        // - insert the requested withdrawal as pending
        substate
            .pending_owner_stake_unit_withdrawals
            .entry(
                current_epoch
                    .after(config_substate.config.num_owner_stake_units_unlock_epochs)
                    .ok_or(RuntimeError::ApplicationError(
                        ApplicationError::ValidatorError(ValidatorError::EpochMathOverflow),
                    ))?,
            )
            .and_modify(|pending_amount| {
                *pending_amount = pending_amount
                    .checked_add(requested_stake_unit_amount)
                    .unwrap_or(Decimal::MAX)
            })
            .or_insert(requested_stake_unit_amount);

        // ...end the read+modify+write of the validator substate
        let mut locked_owner_stake_unit_vault = Vault(substate.locked_owner_stake_unit_vault_id);
        let mut pending_owner_stake_unit_unlock_vault =
            Vault(substate.pending_owner_stake_unit_unlock_vault_id);
        api.field_write_typed(
            handle,
            &ValidatorStateFieldPayload::from_content_source(substate),
        )?;

        // move the requested stake units from the "locked vault" to the "pending withdrawal vault"
        let pending_unlock_stake_unit_bucket =
            locked_owner_stake_unit_vault.take(requested_stake_unit_amount, api)?;
        pending_owner_stake_unit_unlock_vault.put(pending_unlock_stake_unit_bucket, api)?;

        api.field_close(handle)?;
        Ok(())
    }

    /// Finishes the process of unlocking the owner's stake units by withdrawing *all* the pending
    /// amounts which have reached their target epoch and thus are already available (potentially
    /// none).
    pub fn finish_unlock_owner_stake_units<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        // read the current epoch
        let consensus_manager_handle = api.actor_open_field(
            ACTOR_STATE_OUTER_OBJECT,
            ConsensusManagerField::State.into(),
            LockFlags::read_only(),
        )?;
        let consensus_manager = api
            .field_read_typed::<ConsensusManagerStateFieldPayload>(consensus_manager_handle)?
            .fully_update_and_into_latest_version();
        let current_epoch = consensus_manager.epoch;
        api.field_close(consensus_manager_handle)?;

        // drain the already-available withdrawals
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        Self::normalize_available_owner_stake_unit_withdrawals(&mut substate, current_epoch)?;
        let total_already_available_amount = mem::replace(
            &mut substate.already_unlocked_owner_stake_unit_amount,
            Decimal::zero(),
        );

        let mut pending_owner_stake_unit_unlock_vault =
            Vault(substate.pending_owner_stake_unit_unlock_vault_id);
        api.field_write_typed(
            handle,
            &ValidatorStateFieldPayload::from_content_source(substate),
        )?;

        // return the already-available withdrawals
        let already_available_stake_unit_bucket =
            pending_owner_stake_unit_unlock_vault.take(total_already_available_amount, api)?;

        api.field_close(handle)?;
        Ok(already_available_stake_unit_bucket)
    }

    /// Removes all no-longer-pending owner stake unit withdrawals (i.e. those which have already
    /// reached the given [`current_epoch`]) from [`pending_owner_stake_unit_withdrawals`] into
    /// [`already_unlocked_owner_stake_unit_amount`].
    /// Note: this house-keeping operation prevents the internal collection from growing to a size
    /// which would affect performance (or exceed the substate size limit).
    fn normalize_available_owner_stake_unit_withdrawals(
        substate: &mut ValidatorSubstate,
        current_epoch: Epoch,
    ) -> Result<(), RuntimeError> {
        let available_withdrawal_epochs = substate
            .pending_owner_stake_unit_withdrawals
            .range(..=current_epoch)
            .map(|(epoch, _available_amount)| epoch.clone())
            .collect::<Vec<_>>();
        for available_withdrawal_epoch in available_withdrawal_epochs {
            // no batch delete in a BTree
            let available_amount = substate
                .pending_owner_stake_unit_withdrawals
                .remove(&available_withdrawal_epoch)
                .expect("key was just returned by the iterator");
            substate.already_unlocked_owner_stake_unit_amount = substate
                .already_unlocked_owner_stake_unit_amount
                .checked_add(available_amount)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::ValidatorError(
                        ValidatorError::UnexpectedDecimalComputationError,
                    ),
                ))?;
        }
        Ok(())
    }

    /// Puts the given bucket into this validator's stake XRD vault, effectively increasing the
    /// value of all its stake units.
    /// Note: the validator's proposal statistics passed to this method are used only for creating
    /// an event (i.e. they are only informational and they do not drive any logic at this point).
    pub fn apply_emission<Y: SystemApi<RuntimeError>>(
        xrd_bucket: Bucket,
        concluded_epoch: Epoch,
        proposals_made: u64,
        proposals_missed: u64,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        // begin the read+modify+write of the validator substate...
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        // - resolve the effective validator fee factor
        let effective_validator_fee_factor = match &substate.validator_fee_change_request {
            Some(request) if request.epoch_effective <= concluded_epoch => request.new_fee_factor,
            _ => substate.validator_fee_factor,
        };

        // - calculate the validator fee and subtract it from the emission bucket
        let total_emission_xrd = xrd_bucket.amount(api)?;
        let validator_fee_xrd = effective_validator_fee_factor
            .checked_mul(total_emission_xrd)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::ValidatorError(ValidatorError::UnexpectedDecimalComputationError),
            ))?;
        let fee_xrd_bucket = xrd_bucket.take(validator_fee_xrd, api)?;

        // - put the net emission XRDs into the stake pool
        let mut stake_xrd_vault = Vault(substate.stake_xrd_vault_id);
        let starting_stake_pool_xrd = stake_xrd_vault.amount(api)?;
        stake_xrd_vault.put(xrd_bucket, api)?;

        // - stake the validator fee XRDs (effectively same as regular staking)
        let mut stake_unit_resman = ResourceManager(substate.stake_unit_resource);
        let stake_pool_added_xrd = total_emission_xrd.checked_sub(validator_fee_xrd).ok_or(
            RuntimeError::ApplicationError(ApplicationError::ValidatorError(
                ValidatorError::UnexpectedDecimalComputationError,
            )),
        )?;
        let post_emission_stake_pool_xrd = starting_stake_pool_xrd
            .checked_add(stake_pool_added_xrd)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::ValidatorError(ValidatorError::UnexpectedDecimalComputationError),
            ))?;
        let total_stake_unit_supply = stake_unit_resman.total_supply(api)?.unwrap();
        let stake_unit_mint_amount = Self::calculate_stake_unit_amount(
            validator_fee_xrd,
            post_emission_stake_pool_xrd,
            total_stake_unit_supply,
        )?;
        let fee_stake_unit_bucket = stake_unit_resman.mint_fungible(stake_unit_mint_amount, api)?;
        stake_xrd_vault.put(fee_xrd_bucket, api)?;

        // - immediately lock these new stake units in the internal owner's "public display" vault
        Vault(substate.locked_owner_stake_unit_vault_id).put(fee_stake_unit_bucket.into(), api)?;

        // - update the index, since the stake increased (because of net emission + staking of the validator fee)
        let new_stake_xrd = starting_stake_pool_xrd
            .checked_add(total_emission_xrd)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::ValidatorError(ValidatorError::UnexpectedDecimalComputationError),
            ))?;
        let new_index_key =
            Self::index_update(&substate, substate.is_registered, new_stake_xrd, api)?;

        // ...end the read+modify+write of the validator substate (event can be emitted afterwards)
        substate.sorted_key = new_index_key;
        api.field_write_typed(
            handle,
            &ValidatorStateFieldPayload::from_content_source(substate),
        )?;
        api.field_close(handle)?;

        Runtime::emit_event(
            api,
            ValidatorEmissionAppliedEvent {
                epoch: concluded_epoch,
                starting_stake_pool_xrd,
                stake_pool_added_xrd,
                total_stake_unit_supply,
                validator_fee_xrd,
                proposals_made,
                proposals_missed,
            },
        )?;

        Ok(())
    }

    pub fn apply_reward<Y: SystemApi<RuntimeError>>(
        xrd_bucket: Bucket,
        concluded_epoch: Epoch,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        // begin the read+modify+write of the validator substate...
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            ValidatorField::State.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate = api
            .field_read_typed::<ValidatorStateFieldPayload>(handle)?
            .fully_update_and_into_latest_version();

        // Get the total reward amount
        let total_reward_xrd = xrd_bucket.amount(api)?;

        // Stake it
        let mut stake_xrd_vault = Vault(substate.stake_xrd_vault_id);
        let starting_stake_pool_xrd = stake_xrd_vault.amount(api)?;
        let mut stake_unit_resman = ResourceManager(substate.stake_unit_resource);
        let total_stake_unit_supply = stake_unit_resman.total_supply(api)?.unwrap();
        let stake_unit_mint_amount = Self::calculate_stake_unit_amount(
            total_reward_xrd,
            starting_stake_pool_xrd,
            total_stake_unit_supply,
        )?;
        let new_stake_unit_bucket = stake_unit_resman.mint_fungible(stake_unit_mint_amount, api)?;
        stake_xrd_vault.put(xrd_bucket, api)?;

        // Lock these new stake units in the internal owner's "public display" vault
        Vault(substate.locked_owner_stake_unit_vault_id).put(new_stake_unit_bucket.into(), api)?;

        // Update the index, since the stake increased (because of staking of the reward)
        let new_stake_xrd = starting_stake_pool_xrd
            .checked_add(total_reward_xrd)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::ValidatorError(ValidatorError::UnexpectedDecimalComputationError),
            ))?;
        let new_index_key =
            Self::index_update(&substate, substate.is_registered, new_stake_xrd, api)?;

        // Flush validator substate changes
        substate.sorted_key = new_index_key;
        api.field_write_typed(
            handle,
            &ValidatorStateFieldPayload::from_content_source(substate),
        )?;
        api.field_close(handle)?;

        Runtime::emit_event(
            api,
            ValidatorRewardAppliedEvent {
                epoch: concluded_epoch,
                amount: total_reward_xrd,
            },
        )?;

        Ok(())
    }

    fn to_sorted_key(
        registered: bool,
        stake: Decimal,
        address: ComponentAddress,
    ) -> Result<Option<SortedKey>, RuntimeError> {
        if !registered || stake.is_zero() {
            Ok(None)
        } else {
            Ok(Some((
                create_sort_prefix_from_stake(stake)?,
                scrypto_encode(&address).unwrap(),
            )))
        }
    }

    fn update_validator<Y: SystemApi<RuntimeError>>(
        update: UpdateSecondaryIndex,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        match update {
            UpdateSecondaryIndex::Create {
                index_key,
                key,
                stake,
            } => {
                api.actor_sorted_index_insert_typed(
                    ACTOR_STATE_OUTER_OBJECT,
                    ConsensusManagerCollection::RegisteredValidatorByStakeSortedIndex
                        .collection_index(),
                    index_key,
                    ConsensusManagerRegisteredValidatorByStakeEntryPayload::from_content_source(
                        Validator { key, stake },
                    ),
                )?;
            }
            UpdateSecondaryIndex::UpdatePublicKey { index_key, key } => {
                let mut validator = api
                    .actor_sorted_index_remove_typed::<ConsensusManagerRegisteredValidatorByStakeEntryPayload>(
                        ACTOR_STATE_OUTER_OBJECT,
                        ConsensusManagerCollection::RegisteredValidatorByStakeSortedIndex.collection_index(),
                        &index_key,
                    )?
                    .unwrap().fully_update_and_into_latest_version();
                validator.key = key;
                api.actor_sorted_index_insert_typed(
                    ACTOR_STATE_OUTER_OBJECT,
                    ConsensusManagerCollection::RegisteredValidatorByStakeSortedIndex
                        .collection_index(),
                    index_key,
                    ConsensusManagerRegisteredValidatorByStakeEntryPayload::from_content_source(
                        validator,
                    ),
                )?;
            }
            UpdateSecondaryIndex::UpdateStake {
                index_key,
                new_index_key,
                new_stake_amount,
            } => {
                let mut validator = api
                    .actor_sorted_index_remove_typed::<ConsensusManagerRegisteredValidatorByStakeEntryPayload>(
                        ACTOR_STATE_OUTER_OBJECT,
                        ConsensusManagerCollection::RegisteredValidatorByStakeSortedIndex.collection_index(),
                        &index_key,
                    )?
                    .unwrap().fully_update_and_into_latest_version();
                validator.stake = new_stake_amount;
                api.actor_sorted_index_insert_typed(
                    ACTOR_STATE_OUTER_OBJECT,
                    ConsensusManagerCollection::RegisteredValidatorByStakeSortedIndex
                        .collection_index(),
                    new_index_key,
                    ConsensusManagerRegisteredValidatorByStakeEntryPayload::from_content_source(
                        validator,
                    ),
                )?;
            }
            UpdateSecondaryIndex::Remove { index_key } => {
                api.actor_sorted_index_remove(
                    ACTOR_STATE_OUTER_OBJECT,
                    ConsensusManagerCollection::RegisteredValidatorByStakeSortedIndex
                        .collection_index(),
                    &index_key,
                )?;
            }
        }

        Ok(())
    }

    fn calculate_redemption_value<Y: SystemApi<RuntimeError>>(
        amount_of_stake_units: Decimal,
        validator_substate: &ValidatorSubstate,
        api: &mut Y,
    ) -> Result<Decimal, RuntimeError> {
        let stake_vault = Vault(validator_substate.stake_xrd_vault_id);
        let stake_unit_resman = ResourceManager(validator_substate.stake_unit_resource);

        let active_stake_amount = stake_vault.amount(api)?;
        let total_stake_unit_supply = stake_unit_resman.total_supply(api)?.unwrap();
        let xrd_amount = if total_stake_unit_supply.is_zero() {
            Decimal::zero()
        } else {
            active_stake_amount
                .checked_div(total_stake_unit_supply)
                .and_then(|amount| amount_of_stake_units.checked_mul(amount))
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::ValidatorError(
                        ValidatorError::UnexpectedDecimalComputationError,
                    ),
                ))?
        };

        Ok(xrd_amount)
    }

    /// Returns an amount of stake units to be minted when [`xrd_amount`] of XRDs is being staked.
    fn calculate_stake_unit_amount(
        xrd_amount: Decimal,
        total_stake_xrd_amount: Decimal,
        total_stake_unit_supply: Decimal,
    ) -> Result<Decimal, RuntimeError> {
        if total_stake_xrd_amount.is_zero() {
            Ok(xrd_amount)
        } else {
            total_stake_unit_supply
                .checked_div(total_stake_xrd_amount)
                .and_then(|amount| xrd_amount.checked_mul(amount))
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::ValidatorError(
                        ValidatorError::UnexpectedDecimalComputationError,
                    ),
                ))
        }
    }
}

fn check_validator_fee_factor(fee_factor: Decimal) -> Result<(), RuntimeError> {
    // only allow a proper fraction
    if fee_factor.is_negative() || fee_factor > Decimal::one() {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::ValidatorError(ValidatorError::InvalidValidatorFeeFactor),
        ));
    }
    Ok(())
}

fn create_sort_prefix_from_stake(stake: Decimal) -> Result<[u8; 2], RuntimeError> {
    // Note: XRD max supply is 24bn
    // 24bn / MAX::16 = 366210.9375 - so 100k as a divisor here is sensible.
    // If all available XRD was staked to one validator, they'd have 3.6 * u16::MAX * 100k stake
    // In reality, validators will have far less than u16::MAX * 100k stake, but let's handle that case just in case
    let stake_100k: Decimal = stake
        .checked_div(100000)
        .ok_or(RuntimeError::ApplicationError(
            ApplicationError::ValidatorError(ValidatorError::UnexpectedDecimalComputationError),
        ))?;

    let stake_100k_whole_units = dec!(10)
        .checked_powi(Decimal::SCALE.into())
        .and_then(|power| stake_100k.checked_div(power))
        .ok_or(RuntimeError::ApplicationError(
            ApplicationError::ValidatorError(ValidatorError::UnexpectedDecimalComputationError),
        ))?
        .attos();

    let stake_u16 = if stake_100k_whole_units > I192::from(u16::MAX) {
        u16::MAX
    } else {
        stake_100k_whole_units.try_into().unwrap()
    };
    // We invert the key because we need high stake to appear first and it's ordered ASC
    Ok((u16::MAX - stake_u16).to_be_bytes())
}

struct SecurifiedValidator;

impl SecurifiedRoleAssignment for SecurifiedValidator {
    type OwnerBadgeNonFungibleData = ValidatorOwnerBadgeData;
    const OWNER_BADGE: ResourceAddress = VALIDATOR_OWNER_BADGE;
    const SECURIFY_ROLE: Option<&'static str> = None;
}

pub(crate) struct ValidatorCreator;

impl ValidatorCreator {
    fn create_stake_unit_resource<Y: SystemApi<RuntimeError>>(
        validator_address: GlobalAddress,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError> {
        let stake_unit_resman = ResourceManager::new_fungible(
            OwnerRole::Fixed(rule!(require(global_caller(validator_address)))),
            true,
            18,
            FungibleResourceRoles {
                mint_roles: mint_roles! {
                    minter => rule!(require(global_caller(validator_address)));
                    minter_updater => rule!(deny_all);
                },
                burn_roles: burn_roles! {
                    burner => rule!(require(global_caller(validator_address)));
                    burner_updater => rule!(deny_all);
                },
                ..Default::default()
            },
            metadata_init! {
                "name" => "Liquid Stake Units".to_owned(), locked;
                "description" => "Liquid Stake Unit tokens that represent a proportion of XRD stake delegated to a Radix Network validator.".to_owned(), locked;
                "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-liquid_stake_units.png".to_owned()), locked;
                "validator" => GlobalAddress::from(validator_address), locked;
                "tags" => Vec::<String>::new(), locked;
            },
            None,
            api,
        )?;

        Ok(stake_unit_resman.0)
    }

    fn create_claim_nft<Y: SystemApi<RuntimeError>>(
        validator_address: GlobalAddress,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError> {
        let unstake_resman = ResourceManager::new_non_fungible::<UnstakeData, Y, RuntimeError, _>(
            OwnerRole::Fixed(rule!(require(global_caller(validator_address)))),
            NonFungibleIdType::RUID,
            true,
            NonFungibleResourceRoles {
                mint_roles: mint_roles! {
                    minter => rule!(require(global_caller(validator_address)));
                    minter_updater => rule!(deny_all);
                },
                burn_roles: burn_roles! {
                    burner => rule!(require(global_caller(validator_address)));
                    burner_updater => rule!(deny_all);
                },
                ..Default::default()
            },
            metadata_init! {
                "name" => "Stake Claims NFTs".to_owned(), locked;
                "description" => "Unique Stake Claim tokens that represent a timed claimable amount of XRD stake from a Radix Network validator.".to_owned(), locked;
                "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-stake_claim_NFTs.png".to_owned()), locked;
                "validator" => GlobalAddress::from(validator_address), locked;
                "tags" => Vec::<String>::new(), locked;
            },
            None,
            api,
        )?;

        Ok(unstake_resman.0)
    }

    pub fn create<Y: SystemApi<RuntimeError>>(
        key: Secp256k1PublicKey,
        is_registered: bool,
        fee_factor: Decimal,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket), RuntimeError> {
        // check if validator fee is valid
        check_validator_fee_factor(fee_factor)?;

        let (address_reservation, validator_address) =
            api.allocate_global_address(BlueprintId {
                package_address: CONSENSUS_MANAGER_PACKAGE,
                blueprint_name: VALIDATOR_BLUEPRINT.to_string(),
            })?;

        let stake_xrd_vault = Vault::create(XRD, api)?;
        let pending_xrd_withdraw_vault = Vault::create(XRD, api)?;
        let claim_nft = Self::create_claim_nft(validator_address, api)?;
        let stake_unit_resource = Self::create_stake_unit_resource(validator_address, api)?;
        let locked_owner_stake_unit_vault = Vault::create(stake_unit_resource, api)?;
        let pending_owner_stake_unit_unlock_vault = Vault::create(stake_unit_resource, api)?;
        let pending_owner_stake_unit_withdrawals = BTreeMap::new();

        let substate = ValidatorSubstate {
            sorted_key: None,
            key,
            is_registered,
            accepts_delegated_stake: false,
            validator_fee_factor: fee_factor,
            validator_fee_change_request: None,
            stake_unit_resource,
            claim_nft: claim_nft,
            stake_xrd_vault_id: stake_xrd_vault.0,
            pending_xrd_withdraw_vault_id: pending_xrd_withdraw_vault.0,
            locked_owner_stake_unit_vault_id: locked_owner_stake_unit_vault.0,
            pending_owner_stake_unit_unlock_vault_id: pending_owner_stake_unit_unlock_vault.0,
            pending_owner_stake_unit_withdrawals,
            already_unlocked_owner_stake_unit_amount: Decimal::zero(),
        };

        let protocol_update_readiness_signal = ValidatorProtocolUpdateReadinessSignalSubstate {
            protocol_version_name: None,
        };

        let validator_id = api.new_simple_object(
            VALIDATOR_BLUEPRINT,
            indexmap! {
                ValidatorField::State.field_index() => FieldValue::new(&ValidatorStateFieldPayload::from_content_source(substate)),
                ValidatorField::ProtocolUpdateReadinessSignal.field_index() => FieldValue::new(&ValidatorProtocolUpdateReadinessSignalFieldPayload::from_content_source(protocol_update_readiness_signal)),
            },
        )?;

        let (role_assignment, owner_token_bucket) = SecurifiedValidator::create_securified(
            ValidatorOwnerBadgeData {
                name: "Validator Owner Badge".to_owned(),
                validator: validator_address.try_into().expect("Impossible Case!"),
            },
            Some(NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()),
            api,
        )?;
        let owner_badge_local_id = owner_token_bucket
            .non_fungible_local_ids(api)?
            .first()
            .expect("Impossible Case")
            .clone();
        let metadata = Metadata::create_with_data(
            metadata_init! {
                "owner_badge" => owner_badge_local_id, locked;
                "pool_unit" => GlobalAddress::from(stake_unit_resource), locked;
                "claim_nft" => GlobalAddress::from(claim_nft), locked;
            },
            api,
        )?;

        api.globalize(
            validator_id,
            indexmap!(
                AttachedModuleId::RoleAssignment => role_assignment.0.0,
                AttachedModuleId::Metadata => metadata.0,
            ),
            Some(address_reservation),
        )?;

        Ok((
            ComponentAddress::new_or_panic(validator_address.into()),
            owner_token_bucket,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_key_is_calculated_correctly() {
        assert_eq!(
            create_sort_prefix_from_stake(Decimal::ZERO).unwrap(),
            u16::MAX.to_be_bytes()
        );
        assert_eq!(
            create_sort_prefix_from_stake(dec!(99_999)).unwrap(),
            u16::MAX.to_be_bytes()
        );
        assert_eq!(
            create_sort_prefix_from_stake(dec!(100_000)).unwrap(),
            (u16::MAX - 1).to_be_bytes()
        );
        assert_eq!(
            create_sort_prefix_from_stake(dec!(199_999)).unwrap(),
            (u16::MAX - 1).to_be_bytes()
        );
        assert_eq!(
            create_sort_prefix_from_stake(dec!(200_000)).unwrap(),
            (u16::MAX - 2).to_be_bytes()
        );
        // https://learn.radixdlt.com/article/start-here-radix-tokens-and-tokenomics
        let max_xrd_supply = dec!(24)
            .checked_mul(dec!(10).checked_powi(12).unwrap())
            .unwrap();
        assert_eq!(
            create_sort_prefix_from_stake(max_xrd_supply).unwrap(),
            0u16.to_be_bytes()
        );
    }
}

#[derive(ScryptoSbor)]
pub struct ValidatorOwnerBadgeData {
    pub name: String,
    pub validator: ComponentAddress,
}

impl NonFungibleData for ValidatorOwnerBadgeData {
    const MUTABLE_FIELDS: &'static [&'static str] = &[];
}
