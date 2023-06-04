use super::events::*;
use super::state_machine::*;
use crate::errors::{ApplicationError, RuntimeError, SystemUpstreamError};
use crate::kernel::kernel_api::KernelNodeApi;
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use crate::types::*;
use crate::{event_schema, method_auth_template};
use native_sdk::component::BorrowedObject;
use native_sdk::modules::access_rules::{AccessRules, AccessRulesObject, AttachedAccessRules};
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::NativeBucket;
use native_sdk::resource::NativeVault;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::metadata::{
    Url, METADATA_GET_IDENT, METADATA_REMOVE_IDENT, METADATA_SET_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::package::{
    BlueprintSetup, BlueprintTemplate, PackageSetup,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::{BlueprintSchema, FeaturedSchema, FieldSchema, ReceiverInfo};
use radix_engine_interface::schema::{FunctionSchema, SchemaMethodKey, SchemaMethodPermission};
use radix_engine_interface::time::Instant;
use radix_engine_interface::types::ClientCostingReason;
use radix_engine_interface::*;
use radix_engine_interface::{api::*, rule};
use resources_tracker_macro::trace_resources;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct AccessControllerSubstate {
    /// A vault where the asset controlled by the access controller lives.
    pub controlled_asset: Own,

    /// The amount of time (in minutes) that it takes for timed recovery to be done. Maximum is
    /// 4,294,967,295 minutes which is 8171.5511700913 years. When this is [`None`], then timed
    /// recovery can not be performed through this access controller.
    pub timed_recovery_delay_in_minutes: Option<u32>,

    /// The resource address of the recovery badge that will be used by the wallet and optionally
    /// by other clients as well.
    pub recovery_badge: ResourceAddress,

    /// The states of the Access Controller.
    pub state: (
        // Controls whether the primary role is locked or unlocked
        PrimaryRoleLockingState,
        // Primary role recovery and withdraw states
        PrimaryRoleRecoveryAttemptState,
        PrimaryRoleBadgeWithdrawAttemptState,
        // Recovery role recovery and withdraw states
        RecoveryRoleRecoveryAttemptState,
        RecoveryRoleBadgeWithdrawAttemptState,
    ),
}

impl AccessControllerSubstate {
    pub fn new(
        controlled_asset: Own,
        timed_recovery_delay_in_minutes: Option<u32>,
        recovery_badge: ResourceAddress,
    ) -> Self {
        Self {
            controlled_asset,
            timed_recovery_delay_in_minutes,
            recovery_badge,
            state: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Default)]
pub enum PrimaryRoleLockingState {
    #[default]
    Unlocked,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Default)]
pub enum PrimaryRoleRecoveryAttemptState {
    #[default]
    NoRecoveryAttempt,
    RecoveryAttempt(RecoveryProposal),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Default)]
pub enum PrimaryRoleBadgeWithdrawAttemptState {
    #[default]
    NoBadgeWithdrawAttempt,
    BadgeWithdrawAttempt,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Default)]
pub enum RecoveryRoleRecoveryAttemptState {
    #[default]
    NoRecoveryAttempt,
    RecoveryAttempt(RecoveryRoleRecoveryState),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum RecoveryRoleRecoveryState {
    UntimedRecovery(RecoveryProposal),
    TimedRecovery {
        proposal: RecoveryProposal,
        timed_recovery_allowed_after: Instant,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Default)]
pub enum RecoveryRoleBadgeWithdrawAttemptState {
    #[default]
    NoBadgeWithdrawAttempt,
    BadgeWithdrawAttempt,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AccessControllerError {
    /// Occurs when some action requires that the primary role is unlocked to happen.
    OperationRequiresUnlockedPrimaryRole,

    /// Occurs when adding time to an [`Instant`] results in an overflow
    TimeOverflow,

    /// Occurs when a proposer attempts to initiate another recovery when they already have a
    /// recovery underway.
    RecoveryAlreadyExistsForProposer { proposer: Proposer },

    /// Occurs when no recovery can be found for a given proposer.
    NoRecoveryExistsForProposer { proposer: Proposer },

    /// Occurs when a proposer attempts to initiate another badge withdraw when they already have a
    /// recovery underway.
    BadgeWithdrawAttemptAlreadyExistsForProposer { proposer: Proposer },

    /// Occurs when no recovery can be found for a given proposer.
    NoBadgeWithdrawAttemptExistsForProposer { proposer: Proposer },

    /// Occurs when there is no timed recoveries on the controller - typically because it isn't in
    /// the state that allows for it.
    NoTimedRecoveriesFound,

    /// Occurs when trying to perform a timed confirm recovery on a recovery proposal that could
    /// be time-confirmed but whose delay has not yet elapsed.
    TimedRecoveryDelayHasNotElapsed,

    /// Occurs when the expected recovery proposal doesn't match that which was found
    RecoveryProposalMismatch {
        expected: Box<RecoveryProposal>,
        found: Box<RecoveryProposal>,
    },
}

impl From<AccessControllerError> for RuntimeError {
    fn from(value: AccessControllerError) -> Self {
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(value))
    }
}

pub struct AccessControllerNativePackage;

impl AccessControllerNativePackage {
    pub fn definition() -> PackageSetup {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(FieldSchema::normal(
            aggregator.add_child_type_and_descendents::<AccessControllerSubstate>(),
        ));

        let mut functions = BTreeMap::new();
        functions.insert(
            ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerCreateGlobalInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerCreateGlobalOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_POST_INSTANTIATION_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerPostInstantiationInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerPostInstantiationOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_POST_INSTANTIATION_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerCreateProofInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerCreateProofOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_CREATE_PROOF_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateRecoveryAsPrimaryInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateRecoveryAsPrimaryOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateRecoveryAsRecoveryInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateRecoveryAsRecoveryOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmPrimaryRoleRecoveryProposalOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmRecoveryRoleRecoveryProposalOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerTimedConfirmRecoveryInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerTimedConfirmRecoveryOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelPrimaryRoleRecoveryProposalInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelPrimaryRoleRecoveryProposalOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelRecoveryRoleRecoveryProposalInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelRecoveryRoleRecoveryProposalOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerLockPrimaryRoleInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerLockPrimaryRoleOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerUnlockPrimaryRoleInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerUnlockPrimaryRoleOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerStopTimedRecoveryInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerStopTimedRecoveryOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerMintRecoveryBadgesInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerMintRecoveryBadgesOutput>(),
                export: FeaturedSchema::normal(ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT),
            },
        );

        let event_schema = event_schema! {
            aggregator,
            [
                InitiateRecoveryEvent,
                RuleSetUpdateEvent,
                CancelRecoveryProposalEvent,
                LockPrimaryRoleEvent,
                UnlockPrimaryRoleEvent,
                StopTimedRecoveryEvent,
                InitiateBadgeWithdrawAttemptEvent,
                BadgeWithdrawEvent,
                CancelBadgeWithdrawAttemptEvent
            ]
        };

        let method_auth_template = method_auth_template!(
            SchemaMethodKey::metadata(METADATA_SET_IDENT) => [SELF_ROLE];
            SchemaMethodKey::metadata(METADATA_REMOVE_IDENT) => [SELF_ROLE];
            SchemaMethodKey::metadata(METADATA_GET_IDENT) => SchemaMethodPermission::Public;

            SchemaMethodKey::main(ACCESS_CONTROLLER_CREATE_PROOF_IDENT) => ["primary"];
            SchemaMethodKey::main(ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT) => ["primary"];
            SchemaMethodKey::main(ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT) => ["primary"];
            SchemaMethodKey::main(ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT) => ["primary"];
            SchemaMethodKey::main(ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT) =>  ["primary"];
            SchemaMethodKey::main(ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT) => ["recovery"];
            SchemaMethodKey::main(ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT) => ["recovery"];
            SchemaMethodKey::main(ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT) => SchemaMethodPermission::Public;
            SchemaMethodKey::main(ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT) => ["recovery"];
            SchemaMethodKey::main(ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT) => ["recovery"];
            SchemaMethodKey::main(ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT) => ["recovery"];
            SchemaMethodKey::main(ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT) => ["recovery"];

            SchemaMethodKey::main(ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT) => ["recovery", "confirmation"];
            SchemaMethodKey::main(ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT) => ["recovery", "confirmation"];

            SchemaMethodKey::main(ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT) => ["primary", "confirmation"];
            SchemaMethodKey::main(ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT) => ["primary", "confirmation"];

            SchemaMethodKey::main(ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT) => ["primary", "recovery"];

            SchemaMethodKey::main(ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT) => ["primary", "confirmation", "recovery"];

            SchemaMethodKey::main(ACCESS_CONTROLLER_POST_INSTANTIATION_IDENT) => ["this_package"];
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            ACCESS_CONTROLLER_BLUEPRINT.to_string() => BlueprintSetup {
                schema: BlueprintSchema {
                    outer_blueprint: None,
                    schema,
                    fields,
                    collections: vec![],
                    functions,
                    virtual_lazy_load_functions: btreemap!(),
                    event_schema,
                    dependencies: btreeset!(
                        PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
                    ),
                    features: btreeset!(),
                },
                function_auth: btreemap!(
                    ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string() => rule!(allow_all),
                ),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template,
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
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        match export_name {
            ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                Self::create_global(input, api)
            }
            ACCESS_CONTROLLER_POST_INSTANTIATION_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::post_instantiation(input, api)
            }
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::create_proof(input, api)
            }
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::initiate_recovery_as_primary(input, api)
            }
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::initiate_recovery_as_recovery(input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::quick_confirm_primary_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::quick_confirm_recovery_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::timed_confirm_recovery(receiver, input, api)
            }
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::cancel_primary_role_recovery_proposal(input, api)
            }
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::cancel_recovery_role_recovery_proposal(input, api)
            }
            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::lock_primary_role(input, api)
            }
            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::unlock_primary_role(input, api)
            }
            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::stop_timed_recovery(input, api)
            }
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::initiate_badge_withdraw_attempt_as_primary(input, api)
            }
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::initiate_badge_withdraw_attempt_as_recovery(input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::quick_confirm_primary_role_badge_withdraw_attempt(receiver, input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::quick_confirm_recovery_role_badge_withdraw_attempt(receiver, input, api)
            }
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::cancel_primary_role_badge_withdraw_attempt(input, api)
            }
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::cancel_recovery_role_badge_withdraw_attempt(input, api)
            }
            ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::mint_recovery_badges(input, api)
            }
            _ => Err(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    fn create_global<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let input: AccessControllerCreateGlobalInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        // Allocating the address of the access controller - this will be needed for the metadata
        // and access rules of the recovery badge
        let node_id = api.kernel_allocate_node_id(EntityType::GlobalAccessController)?;
        let address = GlobalAddress::new_or_panic(node_id.0);

        // Creating a new vault and putting in it the controlled asset
        let vault = {
            let mut vault = input
                .controlled_asset
                .resource_address(api)
                .and_then(|resource_address| Vault::create(resource_address, api))?;
            vault.put(input.controlled_asset, api)?;

            vault
        };

        // Creating a new recovery badge resource
        let recovery_badge_resource = {
            let global_component_caller_badge =
                NonFungibleGlobalId::global_caller_badge(GlobalCaller::GlobalObject(address));

            // Hack: Interfaces for initializing metadata only allows for <String, String> metadata
            // but the interfaces for setting metadata allow for any `MetadataEntry`. So we will
            // set the metadata to be updatable by the component caller badge and then switch it
            // back to only be immutable.
            // FIXME: When metadata initialization allows MetadataEntry stop making update metadata
            // rule be transient
            let access_rules = [
                (
                    ResourceMethodAuthKey::Mint,
                    (
                        rule!(require(global_component_caller_badge.clone())),
                        AccessRule::DenyAll,
                    ),
                ),
                (
                    ResourceMethodAuthKey::Burn,
                    (AccessRule::AllowAll, AccessRule::DenyAll),
                ),
                (
                    ResourceMethodAuthKey::Withdraw,
                    (AccessRule::DenyAll, AccessRule::DenyAll),
                ),
                (
                    ResourceMethodAuthKey::Deposit,
                    (AccessRule::AllowAll, AccessRule::DenyAll),
                ),
                (
                    ResourceMethodAuthKey::UpdateMetadata,
                    (
                        rule!(require(global_component_caller_badge.clone())),
                        rule!(require(global_component_caller_badge.clone())),
                    ),
                ),
                (
                    ResourceMethodAuthKey::Recall,
                    (AccessRule::DenyAll, AccessRule::DenyAll),
                ),
                (
                    ResourceMethodAuthKey::UpdateNonFungibleData,
                    (AccessRule::DenyAll, AccessRule::DenyAll),
                ),
            ];

            let resource_address = {
                let (local_type_index, schema) =
                    generate_full_schema_from_single_type::<(), ScryptoCustomSchema>();
                let non_fungible_schema = NonFungibleDataSchema {
                    schema,
                    non_fungible: local_type_index,
                    mutable_fields: Default::default(),
                };

                let result = api.call_function(
                    RESOURCE_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                    scrypto_encode(&NonFungibleResourceManagerCreateInput {
                        features: vec![TRACK_TOTAL_SUPPLY_FEATURE.to_string()],
                        id_type: NonFungibleIdType::Integer,
                        non_fungible_schema,
                        metadata: Default::default(),
                        access_rules: access_rules.into(),
                    })
                    .unwrap(),
                )?;
                scrypto_decode::<ResourceAddress>(result.as_slice()).unwrap()
            };

            resource_address
        };

        let substate = AccessControllerSubstate::new(
            vault.0,
            input.timed_recovery_delay_in_minutes,
            recovery_badge_resource,
        );
        let object_id = api.new_simple_object(
            ACCESS_CONTROLLER_BLUEPRINT,
            vec![scrypto_encode(&substate).unwrap()],
        )?;

        let roles = init_roles_from_rule_set(input.rule_set);
        let access_rules = AccessRules::create(roles, api)?.0;

        let metadata = Metadata::create(api)?;
        let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

        // Creating a global component address for the access controller RENode
        api.globalize_with_address(
            btreemap!(
                ObjectModuleId::Main => object_id,
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ),
            address,
        )?;

        // Invoking the post-initialization method on the component
        api.call_method(
            &node_id,
            ACCESS_CONTROLLER_POST_INSTANTIATION_IDENT,
            scrypto_encode(&AccessControllerPostInstantiationInput).unwrap(),
        )?;

        Ok(IndexedScryptoValue::from_typed(&address))
    }

    /// This method is only callable when the access controller virtual direct package caller
    /// badge is present.
    ///
    /// This method has been added due to an issue with setting the metadata of the recovery badge
    /// resource to include the address of the access controller before the access controller was
    /// globalized. Doing that lead to an error along the lines of "callframe has no reference to
    /// the (access controller) node id" because the access controller's node id has been allocated
    /// but nothing has been globalized to it. Thus, the call frame did not have a reference to the
    /// global address to pass when calling the metadata set method.
    ///
    /// A minimal example that reproduces this issue is only possible after the metadata interfaces
    /// are updated to support any MetadataEntry.
    ///
    /// The method below can be removed and the logic can be moved to the instantiation function
    /// once this bug is fixed.
    fn post_instantiation<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        input
            .as_typed::<AccessControllerPostInstantiationInput>()
            .map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?;

        let access_controller = api.actor_get_global_address()?;
        let resource_address = {
            let substate_key = AccessControllerField::AccessController.into();
            let handle =
                api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::read_only())?;

            let access_controller = {
                let access_controller: AccessControllerSubstate =
                    api.field_lock_read_typed(handle)?;
                access_controller
            };
            access_controller.recovery_badge
        };

        let mut resource_manager = BorrowedObject(resource_address.into_node_id());
        resource_manager.set_metadata("name", "Recovery Badge".to_owned(), api)?;
        resource_manager.set_metadata(
            "icon_url",
            Url("https://assets.radixdlt.com/icons/icon-recovery_badge.png".to_owned()),
            api,
        )?;
        resource_manager.set_metadata("access_controller", access_controller, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn create_proof<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let _input: AccessControllerCreateProofInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let proof = transition(api, AccessControllerCreateProofStateMachineInput)?;

        Ok(IndexedScryptoValue::from_typed(&proof))
    }

    fn initiate_recovery_as_primary<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: AccessControllerInitiateRecoveryAsPrimaryInput =
            input.as_typed().map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?;
        let proposal = RecoveryProposal {
            rule_set: input.rule_set,
            timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
        };

        transition_mut(
            api,
            AccessControllerInitiateRecoveryAsPrimaryStateMachineInput {
                proposal: proposal.clone(),
            },
        )?;

        Runtime::emit_event(
            api,
            InitiateRecoveryEvent {
                proposal,
                proposer: Proposer::Primary,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn initiate_recovery_as_recovery<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: AccessControllerInitiateRecoveryAsRecoveryInput =
            input.as_typed().map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?;
        let proposal = RecoveryProposal {
            rule_set: input.rule_set,
            timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
        };

        transition_mut(
            api,
            AccessControllerInitiateRecoveryAsRecoveryStateMachineInput {
                proposal: proposal.clone(),
            },
        )?;

        Runtime::emit_event(
            api,
            InitiateRecoveryEvent {
                proposal,
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn initiate_badge_withdraw_attempt_as_primary<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        input
            .as_typed::<AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput>()
            .map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?;

        transition_mut(
            api,
            AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            InitiateBadgeWithdrawAttemptEvent {
                proposer: Proposer::Primary,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn initiate_badge_withdraw_attempt_as_recovery<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        input
            .as_typed::<AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput>()
            .map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?;

        transition_mut(
            api,
            AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            InitiateBadgeWithdrawAttemptEvent {
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn quick_confirm_primary_role_recovery_proposal<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput =
            input.as_typed().map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?;
        let proposal = RecoveryProposal {
            rule_set: input.rule_set,
            timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
        };

        let recovery_proposal = transition_mut(
            api,
            AccessControllerQuickConfirmPrimaryRoleRecoveryProposalStateMachineInput {
                proposal_to_confirm: proposal.clone(),
            },
        )?;

        update_access_rules(api, receiver, recovery_proposal.rule_set)?;

        Runtime::emit_event(
            api,
            RuleSetUpdateEvent {
                proposal,
                proposer: Proposer::Primary,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn quick_confirm_recovery_role_recovery_proposal<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput =
            input.as_typed().map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?;
        let proposal = RecoveryProposal {
            rule_set: input.rule_set,
            timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
        };

        let recovery_proposal = transition_mut(
            api,
            AccessControllerQuickConfirmRecoveryRoleRecoveryProposalStateMachineInput {
                proposal_to_confirm: proposal.clone(),
            },
        )?;

        update_access_rules(api, receiver, recovery_proposal.rule_set)?;

        Runtime::emit_event(
            api,
            RuleSetUpdateEvent {
                proposal,
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn quick_confirm_primary_role_badge_withdraw_attempt<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        input
            .as_typed::<AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptInput>()
            .map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?;

        let bucket = transition_mut(
            api,
            AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptStateMachineInput,
        )?;

        update_access_rules(api, receiver, locked_access_rules())?;

        Runtime::emit_event(
            api,
            BadgeWithdrawEvent {
                proposer: Proposer::Primary,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&bucket))
    }

    fn quick_confirm_recovery_role_badge_withdraw_attempt<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        input
            .as_typed::<AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptInput>()
            .map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?;

        let bucket = transition_mut(
            api,
            AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptStateMachineInput,
        )?;

        update_access_rules(api, receiver, locked_access_rules())?;

        Runtime::emit_event(
            api,
            BadgeWithdrawEvent {
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&bucket))
    }

    fn timed_confirm_recovery<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: AccessControllerTimedConfirmRecoveryInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;
        let proposal = RecoveryProposal {
            rule_set: input.rule_set,
            timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
        };

        let recovery_proposal = transition_mut(
            api,
            AccessControllerTimedConfirmRecoveryStateMachineInput {
                proposal_to_confirm: proposal.clone(),
            },
        )?;

        // Update the access rules
        update_access_rules(api, receiver, recovery_proposal.rule_set)?;

        Runtime::emit_event(
            api,
            RuleSetUpdateEvent {
                proposal,
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn cancel_primary_role_recovery_proposal<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let _input: AccessControllerCancelPrimaryRoleRecoveryProposalInput =
            input.as_typed().map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?;

        transition_mut(
            api,
            AccessControllerCancelPrimaryRoleRecoveryProposalStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            CancelRecoveryProposalEvent {
                proposer: Proposer::Primary,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn cancel_recovery_role_recovery_proposal<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let _input: AccessControllerCancelRecoveryRoleRecoveryProposalInput =
            input.as_typed().map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?;

        transition_mut(
            api,
            AccessControllerCancelRecoveryRoleRecoveryProposalStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            CancelRecoveryProposalEvent {
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn cancel_primary_role_badge_withdraw_attempt<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        input
            .as_typed::<AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptInput>()
            .map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?;

        transition_mut(
            api,
            AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            CancelBadgeWithdrawAttemptEvent {
                proposer: Proposer::Primary,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn cancel_recovery_role_badge_withdraw_attempt<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        input
            .as_typed::<AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptInput>()
            .map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?;

        transition_mut(
            api,
            AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            CancelBadgeWithdrawAttemptEvent {
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn lock_primary_role<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let _input: AccessControllerLockPrimaryRoleInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        transition_mut(api, AccessControllerLockPrimaryRoleStateMachineInput)?;
        Runtime::emit_event(api, LockPrimaryRoleEvent {})?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn unlock_primary_role<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let _input: AccessControllerUnlockPrimaryRoleInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        transition_mut(api, AccessControllerUnlockPrimaryRoleStateMachineInput)?;
        Runtime::emit_event(api, UnlockPrimaryRoleEvent {})?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn stop_timed_recovery<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: AccessControllerStopTimedRecoveryInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        transition_mut(
            api,
            AccessControllerStopTimedRecoveryStateMachineInput {
                proposal: RecoveryProposal {
                    rule_set: input.rule_set,
                    timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
                },
            },
        )?;
        Runtime::emit_event(api, StopTimedRecoveryEvent)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn mint_recovery_badges<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let AccessControllerMintRecoveryBadgesInput {
            non_fungible_local_ids,
        } = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let resource_address = {
            let substate_key = AccessControllerField::AccessController.into();
            let handle =
                api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::read_only())?;

            let access_controller = {
                let access_controller: AccessControllerSubstate =
                    api.field_lock_read_typed(handle)?;
                access_controller
            };
            access_controller.recovery_badge
        };

        let non_fungibles: BTreeMap<NonFungibleLocalId, (ScryptoValue,)> = non_fungible_local_ids
            .into_iter()
            .map(|local_id| {
                (
                    local_id,
                    (scrypto_decode(&scrypto_encode(&()).unwrap()).unwrap(),),
                )
            })
            .collect();

        let rtn = api.call_method(
            resource_address.as_node_id(),
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
            scrypto_encode(&NonFungibleResourceManagerMintInput {
                entries: non_fungibles,
            })
            .unwrap(),
        )?;

        Ok(IndexedScryptoValue::from_slice(&rtn).unwrap())
    }
}

//=========
// Helpers
//=========

fn locked_access_rules() -> RuleSet {
    RuleSet {
        primary_role: AccessRule::DenyAll,
        recovery_role: AccessRule::DenyAll,
        confirmation_role: AccessRule::DenyAll,
    }
}

fn init_roles_from_rule_set(rule_set: RuleSet) -> Roles {
    roles2! {
        "this_package" => rule!(require(NonFungibleGlobalId::package_of_direct_caller_badge(ACCESS_CONTROLLER_PACKAGE)));
        "primary" => rule_set.primary_role, mut [SELF_ROLE];
        "recovery" => rule_set.recovery_role, mut [SELF_ROLE];
        "confirmation" => rule_set.confirmation_role, mut [SELF_ROLE];
    }
}

fn transition<Y, I>(
    api: &mut Y,
    input: I,
) -> Result<<AccessControllerSubstate as Transition<I>>::Output, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
    AccessControllerSubstate: Transition<I>,
{
    let substate_key = AccessControllerField::AccessController.into();
    let handle = api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::read_only())?;

    let access_controller = {
        let access_controller: AccessControllerSubstate = api.field_lock_read_typed(handle)?;
        access_controller
    };

    let rtn = access_controller.transition(api, input)?;

    api.field_lock_release(handle)?;

    Ok(rtn)
}

fn transition_mut<Y, I>(
    api: &mut Y,
    input: I,
) -> Result<<AccessControllerSubstate as TransitionMut<I>>::Output, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
    AccessControllerSubstate: TransitionMut<I>,
{
    let substate_key = AccessControllerField::AccessController.into();
    let handle = api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::MUTABLE)?;

    let mut access_controller = {
        let access_controller: AccessControllerSubstate = api.field_lock_read_typed(handle)?;
        access_controller
    };

    let rtn = access_controller.transition_mut(api, input)?;

    {
        api.field_lock_write_typed(handle, &access_controller)?;
    }

    api.field_lock_release(handle)?;

    Ok(rtn)
}

fn update_access_rules<Y>(
    api: &mut Y,
    receiver: &NodeId,
    rule_set: RuleSet,
) -> Result<(), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let attached = AttachedAccessRules(receiver.clone());
    attached.update_role_rules(RoleKey::new("primary"), rule_set.primary_role.clone(), api)?;
    attached.update_role_rules(
        RoleKey::new("recovery"),
        rule_set.recovery_role.clone(),
        api,
    )?;
    attached.update_role_rules(
        RoleKey::new("confirmation"),
        rule_set.confirmation_role.clone(),
        api,
    )?;

    Ok(())
}
