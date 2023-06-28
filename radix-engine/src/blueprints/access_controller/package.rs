use super::events::*;
use super::state_machine::*;
use crate::errors::{ApplicationError, RuntimeError};
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use crate::{event_schema, roles_template};
use native_sdk::modules::access_rules::{AccessRules, AccessRulesObject, AttachedAccessRules};
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::NativeBucket;
use native_sdk::resource::NativeVault;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::auth::ToRoleEntry;
use radix_engine_interface::api::node_modules::metadata::MetadataRoles;
use radix_engine_interface::api::node_modules::metadata::Url;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, FunctionAuth, MethodAuthTemplate,
    PackageDefinition,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::{
    BlueprintFunctionsSchemaInit, BlueprintSchemaInit, BlueprintStateSchemaInit, FieldSchema,
    FunctionSchemaInit, ReceiverInfo, TypeRef,
};
use radix_engine_interface::time::Instant;
use radix_engine_interface::*;
use radix_engine_interface::{api::*, rule};
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
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(FieldSchema::static_field(
            aggregator.add_child_type_and_descendents::<AccessControllerSubstate>(),
        ));

        let mut functions = BTreeMap::new();
        functions.insert(
            ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerCreateGlobalInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerCreateGlobalOutput>(),
                ),
                export: ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccessControllerCreateProofInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerCreateProofOutput>(),
                ),
                export: ACCESS_CONTROLLER_CREATE_PROOF_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateRecoveryAsPrimaryInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateRecoveryAsPrimaryOutput>()),
                export: ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateRecoveryAsRecoveryInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateRecoveryAsRecoveryOutput>()),
                export: ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmPrimaryRoleRecoveryProposalOutput>()),
                export: ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmRecoveryRoleRecoveryProposalOutput>()),
                export: ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerTimedConfirmRecoveryInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerTimedConfirmRecoveryOutput>()),
                export: ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelPrimaryRoleRecoveryProposalInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelPrimaryRoleRecoveryProposalOutput>()),
                export: ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelRecoveryRoleRecoveryProposalInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelRecoveryRoleRecoveryProposalOutput>()),
                export: ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerLockPrimaryRoleInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerLockPrimaryRoleOutput>(),
                ),
                export: ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerUnlockPrimaryRoleInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerUnlockPrimaryRoleOutput>(
                        ),
                ),
                export: ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerStopTimedRecoveryInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerStopTimedRecoveryOutput>(
                        ),
                ),
                export: ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryOutput>()),
                export: ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryOutput>()),
                export: ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptOutput>()),
                export: ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptOutput>()),
                export: ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptOutput>()),
                export: ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptOutput>()),
                export: ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerMintRecoveryBadgesInput>(
                        ),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerMintRecoveryBadgesOutput>(
                        ),
                ),
                export: ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT.to_string(),
            },
        );

        let events = event_schema! {
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

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            ACCESS_CONTROLLER_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                feature_set: btreeset!(),
                dependencies: btreeset!(
                    PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
                ),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections: vec![],
                    },
                    events,
                    functions: BlueprintFunctionsSchemaInit {
                        virtual_lazy_load_functions: btreemap!(),
                        functions,
                    },
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::StaticRoles(roles_template!(
                        roles {
                            "primary" => updaters: [SELF_ROLE];
                            "recovery" => updaters: [SELF_ROLE];
                            "confirmation" => updaters: [SELF_ROLE];
                            "this_package";
                        },
                        methods {
                            ACCESS_CONTROLLER_CREATE_PROOF_IDENT => ["primary"];
                            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT => ["primary"];
                            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => ["primary"];
                            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT => ["primary"];
                            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT =>  ["primary"];
                            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT => ["recovery"];
                            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT => ["recovery"];
                            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT => MethodAccessibility::Public;
                            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => ["recovery"];
                            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => ["recovery"];
                            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT => ["recovery"];
                            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT => ["recovery"];

                            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => ["recovery", "confirmation"];
                            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => ["recovery", "confirmation"];

                            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => ["primary", "confirmation"];
                            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => ["primary", "confirmation"];

                            ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT => ["primary", "recovery"];

                            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT => ["primary", "confirmation", "recovery"];
                        }
                    )),
                },
            }
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        match export_name {
            ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT => Self::create_global(input, api),
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT => Self::create_proof(input, api),
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT => {
                Self::initiate_recovery_as_primary(input, api)
            }
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT => {
                Self::initiate_recovery_as_recovery(input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                let receiver = Runtime::get_node_id(api)?;
                Self::quick_confirm_primary_role_recovery_proposal(&receiver, input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                let receiver = Runtime::get_node_id(api)?;
                Self::quick_confirm_recovery_role_recovery_proposal(&receiver, input, api)
            }
            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT => {
                let receiver = Runtime::get_node_id(api)?;
                Self::timed_confirm_recovery(&receiver, input, api)
            }
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                Self::cancel_primary_role_recovery_proposal(input, api)
            }
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                Self::cancel_recovery_role_recovery_proposal(input, api)
            }
            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT => Self::lock_primary_role(input, api),
            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT => Self::unlock_primary_role(input, api),
            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT => Self::stop_timed_recovery(input, api),
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT => {
                Self::initiate_badge_withdraw_attempt_as_primary(input, api)
            }
            ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT => {
                Self::initiate_badge_withdraw_attempt_as_recovery(input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => {
                let receiver = Runtime::get_node_id(api)?;
                Self::quick_confirm_primary_role_badge_withdraw_attempt(&receiver, input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => {
                let receiver = Runtime::get_node_id(api)?;
                Self::quick_confirm_recovery_role_badge_withdraw_attempt(&receiver, input, api)
            }
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => {
                Self::cancel_primary_role_badge_withdraw_attempt(input, api)
            }
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => {
                Self::cancel_recovery_role_badge_withdraw_attempt(input, api)
            }
            ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT => Self::mint_recovery_badges(input, api),
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    fn create_global<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: AccessControllerCreateGlobalInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        // Allocating the address of the access controller - this will be needed for the metadata
        // and access rules of the recovery badge
        let (address_reservation, address) = api.allocate_global_address(BlueprintId {
            package_address: ACCESS_CONTROLLER_PACKAGE,
            blueprint_name: ACCESS_CONTROLLER_BLUEPRINT.to_string(),
        })?;

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

            let access_rules = btreemap! {
                Mint => (
                    rule!(require(global_component_caller_badge.clone())),
                    AccessRule::DenyAll,
                ),
                Burn => (AccessRule::AllowAll, AccessRule::DenyAll),
                Withdraw => (AccessRule::DenyAll, AccessRule::DenyAll),
                Deposit => (AccessRule::AllowAll, AccessRule::DenyAll),
                Recall => (AccessRule::DenyAll, AccessRule::DenyAll),
                UpdateNonFungibleData => (AccessRule::DenyAll, AccessRule::DenyAll),
            };

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
                        owner_role: OwnerRole::Fixed(rule!(require(global_component_caller_badge))),
                        id_type: NonFungibleIdType::Integer,
                        track_total_supply: true,
                        non_fungible_schema,
                        access_rules: access_rules.into(),
                        metadata: metadata! {
                            roles {
                                metadata_setter => AccessRule::DenyAll, locked;
                                metadata_setter_updater => AccessRule::DenyAll, locked;
                                metadata_locker => AccessRule::DenyAll, locked;
                                metadata_locker_updater => AccessRule::DenyAll, locked;
                            },
                            init {
                                "name" => "Recovery Badge".to_owned(), locked;
                                "icon_url" => Url("https://assets.radixdlt.com/icons/icon-recovery_badge.png".to_owned()), locked;
                                "access_controller" => address, locked;
                            }
                        },
                        address_reservation: None,
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
        let roles = btreemap!(ObjectModuleId::Main => roles);
        let access_rules = AccessRules::create(OwnerRole::None, roles, api)?.0;

        let metadata = Metadata::create_with_data(
            metadata_init! {
                "recovery_badge" => GlobalAddress::from(recovery_badge_resource), locked;
            },
            api,
        )?;
        let royalty = ComponentRoyalty::create(ComponentRoyaltyConfig::default(), api)?;

        // Creating a global component address for the access controller RENode
        api.globalize(
            btreemap!(
                ObjectModuleId::Main => object_id,
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ),
            Some(address_reservation),
        )?;

        Ok(IndexedScryptoValue::from_typed(&address))
    }

    fn create_proof<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let _input: AccessControllerCreateProofInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

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
        let input: AccessControllerInitiateRecoveryAsPrimaryInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;
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
        let input: AccessControllerInitiateRecoveryAsRecoveryInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;
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
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

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
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

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
        let input: AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;
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
        let input: AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;
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
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

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
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

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
        let input: AccessControllerTimedConfirmRecoveryInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;
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
        let _input: AccessControllerCancelPrimaryRoleRecoveryProposalInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

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
        let _input: AccessControllerCancelRecoveryRoleRecoveryProposalInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

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
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

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
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

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
        let _input: AccessControllerLockPrimaryRoleInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

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
        let _input: AccessControllerUnlockPrimaryRoleInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

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
        let input: AccessControllerStopTimedRecoveryInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

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
        } = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        let resource_address = {
            let substate_key = AccessControllerField::AccessController.into();
            let handle =
                api.actor_open_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::read_only())?;

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

fn init_roles_from_rule_set(rule_set: RuleSet) -> RolesInit {
    roles2! {
        "this_package" => rule!(require(NonFungibleGlobalId::package_of_direct_caller_badge(ACCESS_CONTROLLER_PACKAGE)));
        "primary" => rule_set.primary_role, updatable;
        "recovery" => rule_set.recovery_role, updatable;
        "confirmation" => rule_set.confirmation_role, updatable;
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
    let handle = api.actor_open_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::read_only())?;

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
    let handle = api.actor_open_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::MUTABLE)?;

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
    attached.set_role(
        ObjectModuleId::Main,
        RoleKey::new("primary"),
        rule_set.primary_role.clone(),
        api,
    )?;
    attached.set_role(
        ObjectModuleId::Main,
        RoleKey::new("recovery"),
        rule_set.recovery_role.clone(),
        api,
    )?;
    attached.set_role(
        ObjectModuleId::Main,
        RoleKey::new("confirmation"),
        rule_set.confirmation_role.clone(),
        api,
    )?;

    Ok(())
}
