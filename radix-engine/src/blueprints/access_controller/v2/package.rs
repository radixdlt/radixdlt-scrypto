use super::internal_prelude::*;
use crate::errors::*;
use crate::internal_prelude::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::package::*;
use sbor::rust::prelude::*;

pub struct AccessControllerV2NativePackage;

impl AccessControllerV2NativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let feature_set = AccessControllerV2FeatureSet::all_features();
        let state = AccessControllerV2StateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = index_map_new();
        functions.insert(
            ACCESS_CONTROLLER_CREATE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccessControllerCreateInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccessControllerCreateOutput>(),
                ),
                export: ACCESS_CONTROLLER_CREATE_IDENT.to_string(),
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
        functions.insert(
            ACCESS_CONTROLLER_LOCK_RECOVERY_FEE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerLockRecoveryFeeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerLockRecoveryFeeOutput>(),
                ),
                export: ACCESS_CONTROLLER_LOCK_RECOVERY_FEE_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_WITHDRAW_RECOVERY_FEE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerWithdrawRecoveryFeeInput>(
                        ),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerWithdrawRecoveryFeeOutput>(
                        ),
                ),
                export: ACCESS_CONTROLLER_WITHDRAW_RECOVERY_FEE_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerContributeRecoveryFeeInput>(
                        ),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccessControllerContributeRecoveryFeeOutput>(
                        ),
                ),
                export: ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT.to_string(),
            },
        );

        let events = event_schema! {
            aggregator,
            [
                // Original Events
                InitiateRecoveryEvent,
                RuleSetUpdateEvent,
                CancelRecoveryProposalEvent,
                LockPrimaryRoleEvent,
                UnlockPrimaryRoleEvent,
                StopTimedRecoveryEvent,
                InitiateBadgeWithdrawAttemptEvent,
                BadgeWithdrawEvent,
                CancelBadgeWithdrawAttemptEvent,
                // Bottlenose Events
                DepositRecoveryXrdEvent,
                WithdrawRecoveryXrdEvent
            ]
        };

        let schema = generate_full_schema(aggregator);
        let blueprint_definition = BlueprintDefinitionInit {
            blueprint_type: BlueprintType::default(),
            is_transient: false,
            feature_set,
            dependencies: indexset!(PACKAGE_OF_DIRECT_CALLER_RESOURCE.into(),),

            schema: BlueprintSchemaInit {
                generics: vec![],
                schema,
                state,
                events,
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit { functions },
                hooks: BlueprintHooksInit::default(),
            },

            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig {
                function_auth: FunctionAuth::AllowAll,
                method_auth: MethodAuthTemplate::StaticRoleDefinition(roles_template!(
                    roles {
                        "primary" => updaters: [SELF_ROLE];
                        "recovery" => updaters: [SELF_ROLE];
                        "confirmation" => updaters: [SELF_ROLE];
                    },
                    methods {
                        ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT => MethodAccessibility::Public;

                        ACCESS_CONTROLLER_CREATE_PROOF_IDENT => ["primary"];

                        ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT => ["primary"];
                        ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => ["primary"];
                        ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT => ["primary"];
                        ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT =>  ["primary"];

                        ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT => ["recovery"];
                        ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => ["recovery"];
                        ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT => ["recovery"];
                        ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => ["recovery"];

                        ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT => ["recovery"];
                        ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT => ["recovery"];

                        ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => ["recovery", "confirmation"];
                        ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => ["recovery", "confirmation"];

                        ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => ["primary", "confirmation"];
                        ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT => ["primary", "confirmation"];

                        ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT => ["primary", "recovery"];

                        ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT => ["primary", "confirmation", "recovery"];

                        ACCESS_CONTROLLER_LOCK_RECOVERY_FEE_IDENT => ["primary", "confirmation", "recovery"];
                        ACCESS_CONTROLLER_WITHDRAW_RECOVERY_FEE_IDENT => ["primary"];
                        ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT => MethodAccessibility::Public;
                    }
                )),
            },
        };

        let blueprints = indexmap!(
            ACCESS_CONTROLLER_BLUEPRINT.to_string() => blueprint_definition
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        AccessControllerV2Blueprint::invoke_export(export_name, input, api)
    }
}
