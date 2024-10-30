use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::locker::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_tracker::*;
use radix_engine_interface::object_modules::metadata::*;
use radix_engine_interface::object_modules::role_assignment::*;
use radix_engine_interface::object_modules::royalty::*;

use super::*;
use radix_common::prelude::*;
use radix_engine_interface::prelude::*;

/// This macro defines a TypedManifestNativeInvocation type for our native blueprints. To make this
/// easier to define this macro doesn't care about what package they live in when being INVOKED.
/// Blueprints from different packages are all defined in a single map.
macro_rules! define_manifest_typed_invocation {
    (
        $(
            $blueprint_ident: ident => {
                blueprint_id: ($package_address: expr, $blueprint_name: expr),
                functions: {
                    $(
                        $function_ident: ident => ($function_input: ty, $function_name: expr $(,)?)
                    ),* $(,)?
                },
                methods: {
                    $(
                        $method_ident: ident => ($method_input: ty, $method_name: expr $(,)?)
                    ),* $(,)?
                },
                direct_methods: {
                    $(
                        $direct_method_ident: ident => ($direct_method_input: ty, $direct_method_name: expr $(,)?)
                    ),* $(,)?
                } $(,)?
            }
        ),* $(,)?
    ) => {
        paste::paste! {
            /* The TypedManifestNativeInvocation enum */
            #[derive(Debug, ManifestSbor)]
            pub enum TypedManifestNativeInvocation {
                $(
                    [< $blueprint_ident BlueprintInvocation >]([< $blueprint_ident BlueprintInvocation >])
                ),*
            }

            impl TypedManifestNativeInvocation {
                pub fn from_function_invocation(
                    ::radix_common::prelude::BlueprintId {
                        package_address: invoked_package_address,
                        blueprint_name: invoked_blueprint_name
                    }: &::radix_common::prelude::BlueprintId,
                    function_name: &str,
                    args: &::radix_common::prelude::ManifestValue
                ) -> Result<Option<Self>, TypedManifestNativeInvocationError> {
                    // Match over the the invoked package and blueprint name and decode as the
                    // appropriate types.
                    match (*invoked_package_address, invoked_blueprint_name.as_str()) {
                        $(
                            ($package_address, $blueprint_name) => {
                                [< $blueprint_ident BlueprintFunction >]::decode_invocation(
                                    function_name,
                                    args
                                )
                                .map([< $blueprint_ident BlueprintInvocation >]::Function)
                                .map(TypedManifestNativeInvocation::[< $blueprint_ident BlueprintInvocation >])
                                .map(Some)
                            },
                        )*
                        // This means that this isn't a blueprint that we support. We don't have
                        // support for all native blueprints at the moment. As an example, Worktop
                        // and Authzone are not currently (if ever) supported.
                        _ => Ok(None)
                    }
                }

                pub fn from_method_invocation<N: AsRef<NodeId>>(
                    address: &ResolvedDynamicAddress<N>,
                    module_id: ModuleId,
                    method_name: &str,
                    args: &::radix_common::prelude::ManifestValue
                ) -> Result<Option<Self>, TypedManifestNativeInvocationError> {
                    // Getting the blueprint being invoked in this method.
                    let Some(::radix_common::prelude::BlueprintId {
                        package_address: invoked_package_address,
                        blueprint_name: invoked_blueprint_name
                    }) = address.invoked_blueprint_id(module_id) else {
                        return Ok(None)
                    };

                    // Match over the the invoked package and blueprint name and decode as the
                    // appropriate types.
                    match (*invoked_package_address, invoked_blueprint_name.as_str()) {
                        $(
                            ($package_address, $blueprint_name) => {
                                [< $blueprint_ident BlueprintMethod >]::decode_invocation(
                                    method_name,
                                    args
                                )
                                .map([< $blueprint_ident BlueprintInvocation >]::Method)
                                .map(TypedManifestNativeInvocation::[< $blueprint_ident BlueprintInvocation >])
                                .map(Some)
                            },
                        )*
                        // This means that this isn't a blueprint that we support. We don't have
                        // support for all native blueprints at the moment. As an example, Worktop
                        // and Authzone are not currently (if ever) supported.
                        _ => Ok(None)
                    }
                }

                pub fn from_direct_method_invocation<N: AsRef<NodeId>>(
                    address: &ResolvedDynamicAddress<N>,
                    method_name: &str,
                    args: &::radix_common::prelude::ManifestValue
                ) -> Result<Option<Self>, TypedManifestNativeInvocationError> {
                    // Getting the blueprint being invoked in this method.
                    let Some(::radix_common::prelude::BlueprintId {
                        package_address: invoked_package_address,
                        blueprint_name: invoked_blueprint_name
                    }) = address.main_module_blueprint_id() else {
                        return Ok(None)
                    };

                    // Match over the the invoked package and blueprint name and decode as the
                    // appropriate types.
                    match (*invoked_package_address, invoked_blueprint_name.as_str()) {
                        $(
                            ($package_address, $blueprint_name) => {
                                [< $blueprint_ident BlueprintDirectMethod >]::decode_invocation(
                                    method_name,
                                    args
                                )
                                .map([< $blueprint_ident BlueprintInvocation >]::DirectMethod)
                                .map(TypedManifestNativeInvocation::[< $blueprint_ident BlueprintInvocation >])
                                .map(Some)
                            },
                        )*
                        // This means that this isn't a blueprint that we support. We don't have
                        // support for all native blueprints at the moment. As an example, Worktop
                        // and Authzone are not currently (if ever) supported.
                        _ => Ok(None)
                    }
                }
            }

            /* The Method and Function types for each blueprint */
            $(
                #[derive(Debug, ManifestSbor)]
                pub enum [< $blueprint_ident BlueprintInvocation >] {
                    Method([< $blueprint_ident BlueprintMethod >]),
                    DirectMethod([< $blueprint_ident BlueprintDirectMethod >]),
                    Function([< $blueprint_ident BlueprintFunction >]),
                }

                /* The Method Invocations */
                #[derive(Debug, ManifestSbor)]
                pub enum [< $blueprint_ident BlueprintMethod >] {
                    $(
                        $method_ident($method_input)
                    ),*
                }

                impl [< $blueprint_ident BlueprintMethod >] {
                    #![allow(unreachable_patterns, unused_variables)]
                    pub fn decode_invocation(
                        method_name: &str,
                        args: &ManifestValue
                    ) -> Result<Self, TypedManifestNativeInvocationError> {
                        match method_name {
                            $(
                                $method_name => decode_args(args)
                                    .map(Self::$method_ident)
                                    .map_err(|error| {
                                        TypedManifestNativeInvocationError::FailedToDecodeMethodInvocation {
                                            blueprint_id: ::radix_common::prelude::BlueprintId::new(&$package_address, $blueprint_name),
                                            method_name: method_name.to_owned(),
                                            args: args.clone(),
                                            error
                                        }
                                    }),
                            )*
                            // If we get here then it means that an invalid method was called. We
                            // have all of the methods on all blueprints we have supported so this
                            // should be an error.
                            _ => Err(TypedManifestNativeInvocationError::InvokedMethodNotFoundOnNativeBlueprint {
                                blueprint_id: ::radix_common::prelude::BlueprintId::new(&$package_address, $blueprint_name),
                                method: method_name.to_owned()
                            })
                        }
                    }
                }

                #[derive(Debug, ManifestSbor)]
                pub enum [< $blueprint_ident BlueprintDirectMethod >] {
                    $(
                        $direct_method_ident($direct_method_input)
                    ),*
                }

                impl [< $blueprint_ident BlueprintDirectMethod >] {
                    #![allow(unreachable_patterns, unused_variables)]
                    pub fn decode_invocation(
                        direct_method_name: &str,
                        args: &ManifestValue
                    ) -> Result<Self, TypedManifestNativeInvocationError> {
                        match direct_method_name {
                            $(
                                $direct_method_name => decode_args(args)
                                    .map(Self::$direct_method_ident)
                                    .map_err(|error| {
                                        TypedManifestNativeInvocationError::FailedToDecodeDirectMethodInvocation {
                                            blueprint_id: ::radix_common::prelude::BlueprintId::new(&$package_address, $blueprint_name),
                                            method_name: direct_method_name.to_owned(),
                                            args: args.clone(),
                                            error
                                        }
                                    }),
                            )*
                            // If we get here then it means that an invalid method was called. We
                            // have all of the methods on all blueprints we have supported so this
                            // should be an error.
                            _ => Err(TypedManifestNativeInvocationError::InvokedDirectMethodNotFoundOnNativeBlueprint {
                                blueprint_id: ::radix_common::prelude::BlueprintId::new(&$package_address, $blueprint_name),
                                method: direct_method_name.to_owned()
                            })
                        }
                    }
                }

                /* The Function Invocation */
                #[derive(Debug, ManifestSbor)]
                pub enum [< $blueprint_ident BlueprintFunction >] {
                    $(
                        $function_ident($function_input)
                    ),*
                }

                impl [< $blueprint_ident BlueprintFunction >] {
                    #![allow(unreachable_patterns, unused_variables)]
                    pub fn decode_invocation(
                        function_name: &str,
                        args: &ManifestValue
                    ) -> Result<Self, TypedManifestNativeInvocationError> {
                        match function_name {
                            $(
                                $function_name => decode_args(args)
                                    .map(Self::$function_ident)
                                    .map_err(|error| {
                                        TypedManifestNativeInvocationError::FailedToDecodeFunctionInvocation {
                                            blueprint_id: ::radix_common::prelude::BlueprintId::new(&$package_address, $blueprint_name),
                                            function_name: function_name.to_owned(),
                                            args: args.clone(),
                                            error
                                        }
                                    }),
                            )*
                            // If we get here then it means that an invalid function was called. We
                            // have all of the functions on all blueprints we have supported so this
                            // should be an error.
                            _ => Err(TypedManifestNativeInvocationError::InvokedFunctionNotFoundOnNativeBlueprint {
                                blueprint_id: ::radix_common::prelude::BlueprintId::new(&$package_address, $blueprint_name),
                                function: function_name.to_owned()
                            })
                        }
                    }
                }
            )*

            #[macro_export]
            macro_rules! uniform_match_on_manifest_typed_invocation {
                (
                    $typed_invocation: expr => ($input: ident) => $action: expr
                ) => {
                    match $typed_invocation {
                        $(
                            $(
                                TypedManifestNativeInvocation::[<$blueprint_ident BlueprintInvocation>](
                                    [<$blueprint_ident BlueprintInvocation>]::Method(
                                        [<$blueprint_ident BlueprintMethod>]::$method_ident($input)
                                    )
                                ) => $action,
                            )*
                        )*
                        $(
                            $(
                                TypedManifestNativeInvocation::[<$blueprint_ident BlueprintInvocation>](
                                    [<$blueprint_ident BlueprintInvocation>]::DirectMethod(
                                        [<$blueprint_ident BlueprintDirectMethod>]::$direct_method_ident($input)
                                    )
                                ) => $action,
                            )*
                        )*
                        $(
                            $(
                                TypedManifestNativeInvocation::[<$blueprint_ident BlueprintInvocation>](
                                    [<$blueprint_ident BlueprintInvocation>]::Function(
                                        [<$blueprint_ident BlueprintFunction>]::$function_ident($input)
                                    )
                                ) => $action,
                            )*
                        )*
                        // AVOIDS [E0004] when the enum is empty "note: references are always considered inhabited"
                        // https://github.com/rust-lang/unsafe-code-guidelines/issues/413
                        _ => unreachable!("[E0004]")
                    }
                }
            }
            pub use uniform_match_on_manifest_typed_invocation;
        }
    };
}

define_manifest_typed_invocation! {
    AccessController => {
        blueprint_id: (ACCESS_CONTROLLER_PACKAGE, ACCESS_CONTROLLER_BLUEPRINT),
        functions: {
            Create => (
                AccessControllerCreateManifestInput,
                ACCESS_CONTROLLER_CREATE_IDENT,
            ),
        },
        methods: {
            CreateProof => (
                AccessControllerCreateProofManifestInput,
                ACCESS_CONTROLLER_CREATE_PROOF_IDENT,
            ),
            InitiateRecoveryAsPrimary => (
                AccessControllerInitiateRecoveryAsPrimaryManifestInput,
                ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT,
            ),
            InitiateRecoveryAsRecovery => (
                AccessControllerInitiateRecoveryAsRecoveryManifestInput,
                ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT,
            ),
            QuickConfirmPrimaryRoleRecoveryProposal => (
                AccessControllerQuickConfirmPrimaryRoleRecoveryProposalManifestInput,
                ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT,
            ),
            QuickConfirmRecoveryRoleRecoveryProposal => (
                AccessControllerQuickConfirmRecoveryRoleRecoveryProposalManifestInput,
                ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT,
            ),
            TimedConfirmRecovery => (
                AccessControllerTimedConfirmRecoveryManifestInput,
                ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT,
            ),
            CancelPrimaryRoleRecoveryProposal => (
                AccessControllerCancelPrimaryRoleRecoveryProposalManifestInput,
                ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT,
            ),
            CancelRecoveryRoleRecoveryProposal => (
                AccessControllerCancelRecoveryRoleRecoveryProposalManifestInput,
                ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT,
            ),
            LockPrimaryRole => (
                AccessControllerLockPrimaryRoleManifestInput,
                ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT,
            ),
            UnlockPrimaryRole => (
                AccessControllerUnlockPrimaryRoleManifestInput,
                ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT,
            ),
            StopTimedRecovery => (
                AccessControllerStopTimedRecoveryManifestInput,
                ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT,
            ),
            InitiateBadgeWithdrawAttemptAsPrimary => (
                AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryManifestInput,
                ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT,
            ),
            InitiateBadgeWithdrawAttemptAsRecovery => (
                AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryManifestInput,
                ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT,
            ),
            QuickConfirmPrimaryRoleBadgeWithdrawAttempt => (
                AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptManifestInput,
                ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT,
            ),
            QuickConfirmRecoveryRoleBadgeWithdrawAttempt => (
                AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptManifestInput,
                ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT,
            ),
            CancelPrimaryRoleBadgeWithdrawAttempt => (
                AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptManifestInput,
                ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT,
            ),
            CancelRecoveryRoleBadgeWithdrawAttempt => (
                AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptManifestInput,
                ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT,
            ),
            MintRecoveryBadges => (
                AccessControllerMintRecoveryBadgesManifestInput,
                ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT,
            ),
            LockRecoveryFee => (
                AccessControllerLockRecoveryFeeManifestInput,
                ACCESS_CONTROLLER_LOCK_RECOVERY_FEE_IDENT,
            ),
            WithdrawRecoveryFee => (
                AccessControllerWithdrawRecoveryFeeManifestInput,
                ACCESS_CONTROLLER_WITHDRAW_RECOVERY_FEE_IDENT,
            ),
            ContributeRecoveryFee => (
                AccessControllerContributeRecoveryFeeManifestInput,
                ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT,
            ),
        },
        direct_methods: {}
    },
    Account => {
        blueprint_id: (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT),
        functions: {
            CreateAdvanced => (
                AccountCreateAdvancedManifestInput,
                ACCOUNT_CREATE_ADVANCED_IDENT,
            ),
            Create => (
                AccountCreateManifestInput,
                ACCOUNT_CREATE_IDENT,
            ),
        },
        methods: {
            Securify => (
                AccountSecurifyManifestInput,
                ACCOUNT_SECURIFY_IDENT,
            ),
            LockFee => (
                AccountLockFeeManifestInput,
                ACCOUNT_LOCK_FEE_IDENT,
            ),
            LockContingentFee => (
                AccountLockContingentFeeManifestInput,
                ACCOUNT_LOCK_CONTINGENT_FEE_IDENT,
            ),
            Deposit => (
                AccountDepositManifestInput,
                ACCOUNT_DEPOSIT_IDENT,
            ),
            DepositBatch => (
                AccountDepositBatchManifestInput,
                ACCOUNT_DEPOSIT_BATCH_IDENT,
            ),
            Withdraw => (
                AccountWithdrawManifestInput,
                ACCOUNT_WITHDRAW_IDENT,
            ),
            WithdrawNonFungibles => (
                AccountWithdrawNonFungiblesManifestInput,
                ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT,
            ),
            Burn => (
                AccountBurnManifestInput,
                ACCOUNT_BURN_IDENT,
            ),
            BurnNonFungibles => (
                AccountBurnNonFungiblesManifestInput,
                ACCOUNT_BURN_NON_FUNGIBLES_IDENT,
            ),
            LockFeeAndWithdraw => (
                AccountLockFeeAndWithdrawManifestInput,
                ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT,
            ),
            LockFeeAndWithdrawNonFungibles => (
                AccountLockFeeAndWithdrawNonFungiblesManifestInput,
                ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT,
            ),
            CreateProofOfAmount => (
                AccountCreateProofOfAmountManifestInput,
                ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT,
            ),
            CreateProofOfNonFungibles => (
                AccountCreateProofOfNonFungiblesManifestInput,
                ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            ),
            SetDefaultDepositRule => (
                AccountSetDefaultDepositRuleManifestInput,
                ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
            ),
            SetResourcePreference => (
                AccountSetResourcePreferenceManifestInput,
                ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT,
            ),
            RemoveResourcePreference => (
                AccountRemoveResourcePreferenceManifestInput,
                ACCOUNT_REMOVE_RESOURCE_PREFERENCE_IDENT,
            ),
            TryDepositOrRefund => (
                AccountTryDepositOrRefundManifestInput,
                ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT,
            ),
            TryDepositBatchOrRefund => (
                AccountTryDepositBatchOrRefundManifestInput,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT,
            ),
            TryDepositOrAbort => (
                AccountTryDepositOrAbortManifestInput,
                ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT,
            ),
            TryDepositBatchOrAbort => (
                AccountTryDepositBatchOrAbortManifestInput,
                ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
            ),
            AddAuthorizedDepositor => (
                AccountAddAuthorizedDepositorManifestInput,
                ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
            ),
            RemoveAuthorizedDepositor => (
                AccountRemoveAuthorizedDepositorManifestInput,
                ACCOUNT_REMOVE_AUTHORIZED_DEPOSITOR_IDENT,
            ),
        },
        direct_methods: {}
    },
    ConsensusManager => {
        blueprint_id: (CONSENSUS_MANAGER_PACKAGE, CONSENSUS_MANAGER_BLUEPRINT),
        functions: {
            Create => (
                ConsensusManagerCreateManifestInput,
                CONSENSUS_MANAGER_CREATE_IDENT,
            ),
        },
        methods: {
            GetCurrentEpoch => (
                ConsensusManagerGetCurrentEpochManifestInput,
                CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
            ),
            Start => (
                ConsensusManagerStartManifestInput,
                CONSENSUS_MANAGER_START_IDENT,
            ),
            GetCurrentTime => (
                ConsensusManagerGetCurrentTimeManifestInputV2,
                CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT,
            ),
            CompareCurrentTime => (
                ConsensusManagerCompareCurrentTimeManifestInputV2,
                CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT,
            ),
            NextRound => (
                ConsensusManagerNextRoundManifestInput,
                CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
            ),
            CreateValidator => (
                ConsensusManagerCreateValidatorManifestInput,
                CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT,
            ),
        },
        direct_methods: {}
    },
    Validator => {
        blueprint_id: (CONSENSUS_MANAGER_PACKAGE, VALIDATOR_BLUEPRINT),
        functions: {},
        methods: {
            Register => (
                ValidatorRegisterManifestInput,
                VALIDATOR_REGISTER_IDENT,
            ),
            Unregister => (
                ValidatorUnregisterManifestInput,
                VALIDATOR_UNREGISTER_IDENT,
            ),
            StakeAsOwner => (
                ValidatorStakeAsOwnerManifestInput,
                VALIDATOR_STAKE_AS_OWNER_IDENT,
            ),
            Stake => (
                ValidatorStakeManifestInput,
                VALIDATOR_STAKE_IDENT,
            ),
            Unstake => (
                ValidatorUnstakeManifestInput,
                VALIDATOR_UNSTAKE_IDENT,
            ),
            ClaimXrd => (
                ValidatorClaimXrdManifestInput,
                VALIDATOR_CLAIM_XRD_IDENT,
            ),
            UpdateKey => (
                ValidatorUpdateKeyManifestInput,
                VALIDATOR_UPDATE_KEY_IDENT,
            ),
            UpdateFee => (
                ValidatorUpdateFeeManifestInput,
                VALIDATOR_UPDATE_FEE_IDENT,
            ),
            UpdateAcceptDelegatedStake => (
                ValidatorUpdateAcceptDelegatedStakeManifestInput,
                VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT,
            ),
            AcceptsDelegatedStake => (
                ValidatorAcceptsDelegatedStakeManifestInput,
                VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT,
            ),
            TotalStakeXrdAmount => (
                ValidatorTotalStakeXrdAmountManifestInput,
                VALIDATOR_TOTAL_STAKE_XRD_AMOUNT_IDENT,
            ),
            TotalStakeUnitSupply => (
                ValidatorTotalStakeUnitSupplyManifestInput,
                VALIDATOR_TOTAL_STAKE_UNIT_SUPPLY_IDENT,
            ),
            GetRedemptionValue => (
                ValidatorGetRedemptionValueManifestInput,
                VALIDATOR_GET_REDEMPTION_VALUE_IDENT,
            ),
            SignalProtocolUpdateReadiness => (
                ValidatorSignalProtocolUpdateReadinessManifestInput,
                VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS_IDENT,
            ),
            GetProtocolUpdateReadiness => (
                ValidatorGetProtocolUpdateReadinessManifestInput,
                VALIDATOR_GET_PROTOCOL_UPDATE_READINESS_IDENT,
            ),
            LockOwnerStakeUnits => (
                ValidatorLockOwnerStakeUnitsManifestInput,
                VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT,
            ),
            StartUnlockOwnerStakeUnits => (
                ValidatorStartUnlockOwnerStakeUnitsManifestInput,
                VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT,
            ),
            FinishUnlockOwnerStakeUnits => (
                ValidatorFinishUnlockOwnerStakeUnitsManifestInput,
                VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT,
            ),
            ApplyEmission => (
                ValidatorApplyEmissionManifestInput,
                VALIDATOR_APPLY_EMISSION_IDENT,
            ),
            ApplyReward => (
                ValidatorApplyRewardManifestInput,
                VALIDATOR_APPLY_REWARD_IDENT,
            ),
        },
        direct_methods: {}
    },
    Identity => {
        blueprint_id: (IDENTITY_PACKAGE, IDENTITY_BLUEPRINT),
        functions: {
            CreateAdvanced => (
                IdentityCreateAdvancedManifestInput,
                IDENTITY_CREATE_ADVANCED_IDENT,
            ),
            Create => (
                IdentityCreateManifestInput,
                IDENTITY_CREATE_IDENT,
            ),
        },
        methods: {
            Securify => (
                IdentitySecurifyToSingleBadgeManifestInput,
                IDENTITY_SECURIFY_IDENT,
            ),
        },
        direct_methods: {}
    },
    AccountLocker => {
        blueprint_id: (LOCKER_PACKAGE, ACCOUNT_LOCKER_BLUEPRINT),
        functions: {
            Instantiate => (
                AccountLockerInstantiateManifestInput,
                ACCOUNT_LOCKER_INSTANTIATE_IDENT,
            ),
            InstantiateSimple => (
                AccountLockerInstantiateSimpleManifestInput,
                ACCOUNT_LOCKER_INSTANTIATE_SIMPLE_IDENT,
            ),
        },
        methods: {
            Store => (
                AccountLockerStoreManifestInput,
                ACCOUNT_LOCKER_STORE_IDENT,
            ),
            Airdrop => (
                AccountLockerAirdropManifestInput,
                ACCOUNT_LOCKER_AIRDROP_IDENT,
            ),
            Recover => (
                AccountLockerRecoverManifestInput,
                ACCOUNT_LOCKER_RECOVER_IDENT,
            ),
            RecoverNonFungibles => (
                AccountLockerRecoverNonFungiblesManifestInput,
                ACCOUNT_LOCKER_RECOVER_NON_FUNGIBLES_IDENT,
            ),
            Claim => (
                AccountLockerClaimManifestInput,
                ACCOUNT_LOCKER_CLAIM_IDENT,
            ),
            ClaimNonFungibles => (
                AccountLockerClaimNonFungiblesManifestInput,
                ACCOUNT_LOCKER_CLAIM_NON_FUNGIBLES_IDENT,
            ),
            GetAmount => (
                AccountLockerGetAmountManifestInput,
                ACCOUNT_LOCKER_GET_AMOUNT_IDENT,
            ),
            GetNonFungibleLocalIds => (
                AccountLockerGetNonFungibleLocalIdsManifestInput,
                ACCOUNT_LOCKER_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
            ),
        },
        direct_methods: {}
    },
    Package => {
        blueprint_id: (PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
        functions: {
            PublishWasm => (
                PackagePublishWasmManifestInput,
                PACKAGE_PUBLISH_WASM_IDENT,
            ),
            PublishWasmAdvanced => (
                PackagePublishWasmAdvancedManifestInput,
                PACKAGE_PUBLISH_WASM_ADVANCED_IDENT,
            ),
            PublishNative => (
                PackagePublishNativeManifestInput,
                PACKAGE_PUBLISH_NATIVE_IDENT,
            ),
        },
        methods: {
            PackageRoyaltyClaimRoyalties => (
                PackageClaimRoyaltiesManifestInput,
                PACKAGE_CLAIM_ROYALTIES_IDENT,
            ),
        },
        direct_methods: {}
    },
    OneResourcePool => {
        blueprint_id: (POOL_PACKAGE, ONE_RESOURCE_POOL_BLUEPRINT),
        functions: {
            Instantiate => (
                OneResourcePoolInstantiateManifestInput,
                ONE_RESOURCE_POOL_INSTANTIATE_IDENT,
            ),
        },
        methods: {
            Contribute => (
                OneResourcePoolContributeManifestInput,
                ONE_RESOURCE_POOL_CONTRIBUTE_IDENT,
            ),
            Redeem => (
                OneResourcePoolRedeemManifestInput,
                ONE_RESOURCE_POOL_REDEEM_IDENT,
            ),
            ProtectedDeposit => (
                OneResourcePoolProtectedDepositManifestInput,
                ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
            ),
            ProtectedWithdraw => (
                OneResourcePoolProtectedWithdrawManifestInput,
                ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
            ),
            GetRedemptionValue => (
                OneResourcePoolGetRedemptionValueManifestInput,
                ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
            ),
            GetVaultAmount => (
                OneResourcePoolGetVaultAmountManifestInput,
                ONE_RESOURCE_POOL_GET_VAULT_AMOUNT_IDENT,
            ),
        },
        direct_methods: {}
    },
    TwoResourcePool => {
        blueprint_id: (POOL_PACKAGE, TWO_RESOURCE_POOL_BLUEPRINT),
        functions: {
            Instantiate => (
                TwoResourcePoolInstantiateManifestInput,
                TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
            ),
        },
        methods: {
            Contribute => (
                TwoResourcePoolContributeManifestInput,
                TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
            ),
            Redeem => (
                TwoResourcePoolRedeemManifestInput,
                TWO_RESOURCE_POOL_REDEEM_IDENT,
            ),
            ProtectedDeposit => (
                TwoResourcePoolProtectedDepositManifestInput,
                TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
            ),
            ProtectedWithdraw => (
                TwoResourcePoolProtectedWithdrawManifestInput,
                TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
            ),
            GetRedemptionValue => (
                TwoResourcePoolGetRedemptionValueManifestInput,
                TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
            ),
            GetVaultAmounts => (
                TwoResourcePoolGetVaultAmountsManifestInput,
                TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT,
            ),
        },
        direct_methods: {}
    },
    MultiResourcePool => {
        blueprint_id: (POOL_PACKAGE, MULTI_RESOURCE_POOL_BLUEPRINT),
        functions: {
            Instantiate => (
                MultiResourcePoolInstantiateManifestInput,
                MULTI_RESOURCE_POOL_INSTANTIATE_IDENT,
            ),
        },
        methods: {
            Contribute => (
                MultiResourcePoolContributeManifestInput,
                MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
            ),
            Redeem => (
                MultiResourcePoolRedeemManifestInput,
                MULTI_RESOURCE_POOL_REDEEM_IDENT,
            ),
            ProtectedDeposit => (
                MultiResourcePoolProtectedDepositManifestInput,
                MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
            ),
            ProtectedWithdraw => (
                MultiResourcePoolProtectedWithdrawManifestInput,
                MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
            ),
            GetRedemptionValue => (
                MultiResourcePoolGetRedemptionValueManifestInput,
                MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
            ),
            GetVaultAmounts => (
                MultiResourcePoolGetVaultAmountsManifestInput,
                MULTI_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT,
            ),
        },
        direct_methods: {}
    },
    FungibleResourceManager => {
        blueprint_id: (RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
        functions: {
            Create => (
                FungibleResourceManagerCreateManifestInput,
                FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            ),
            CreateWithInitialSupply => (
                FungibleResourceManagerCreateWithInitialSupplyManifestInput,
                FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
            ),
        },
        methods: {
            Mint => (
                FungibleResourceManagerMintManifestInput,
                FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
            ),
            Burn => (
                ResourceManagerBurnManifestInput,
                RESOURCE_MANAGER_BURN_IDENT,
            ),
            PackageBurn => (
                ResourceManagerPackageBurnManifestInput,
                RESOURCE_MANAGER_PACKAGE_BURN_IDENT,
            ),
            CreateEmptyVault => (
                FungibleResourceManagerCreateEmptyVaultManifestInput,
                RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT,
            ),
            CreateEmptyBucket => (
                FungibleResourceManagerCreateEmptyBucketManifestInput,
                RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT,
            ),
            GetResourceType => (
                FungibleResourceManagerGetResourceTypeManifestInput,
                RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT,
            ),
            GetTotalSupply => (
                FungibleResourceManagerGetTotalSupplyManifestInput,
                RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT,
            ),
            AmountForWithdrawal => (
                FungibleResourceManagerAmountForWithdrawalManifestInput,
                RESOURCE_MANAGER_GET_AMOUNT_FOR_WITHDRAWAL_IDENT,
            ),
            DropEmptyBucket => (
                ResourceManagerDropEmptyBucketManifestInput,
                RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT,
            ),
        },
        direct_methods: {}
    },
    NonFungibleResourceManager => {
        blueprint_id: (RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT),
        functions: {
            Create => (
                NonFungibleResourceManagerCreateManifestInput,
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            ),
            CreateWithInitialSupply => (
                NonFungibleResourceManagerCreateWithInitialSupplyManifestInput,
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
            ),
            CreateRuidNonFungibleWithInitialSupply => (
                NonFungibleResourceManagerCreateRuidWithInitialSupplyManifestInput,
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT,
            ),
        },
        methods: {
            Mint => (
                NonFungibleResourceManagerMintManifestInput,
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
            ),
            GetNonFungible => (
                NonFungibleResourceManagerGetNonFungibleManifestInput,
                NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT,
            ),
            UpdateNonFungibleData => (
                NonFungibleResourceManagerUpdateDataManifestInput,
                NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT,
            ),
            NonFungibleExists => (
                NonFungibleResourceManagerExistsManifestInput,
                NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT,
            ),
            MintRuid => (
                NonFungibleResourceManagerMintRuidManifestInput,
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT,
            ),
            MintSingleRuid => (
                NonFungibleResourceManagerMintSingleRuidManifestInput,
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_RUID_IDENT,
            ),
            PackageBurn => (
                ResourceManagerPackageBurnManifestInput,
                RESOURCE_MANAGER_PACKAGE_BURN_IDENT,
            ),
            Burn => (
                ResourceManagerBurnManifestInput,
                RESOURCE_MANAGER_BURN_IDENT,
            ),
            CreateEmptyVault => (
                NonFungibleResourceManagerCreateEmptyVaultManifestInput,
                RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT,
            ),
            CreateEmptyBucket => (
                NonFungibleResourceManagerCreateEmptyBucketManifestInput,
                RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT,
            ),
            GetResourceType => (
                NonFungibleResourceManagerGetResourceTypeManifestInput,
                RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT,
            ),
            GetTotalSupply => (
                NonFungibleResourceManagerGetTotalSupplyManifestInput,
                RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT,
            ),
            AmountForWithdrawal => (
                NonFungibleResourceManagerAmountForWithdrawalManifestInput,
                RESOURCE_MANAGER_GET_AMOUNT_FOR_WITHDRAWAL_IDENT,
            ),
            DropEmptyBucket => (
                ResourceManagerDropEmptyBucketManifestInput,
                RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT,
            ),
        },
        direct_methods: {}
    },
    FungibleVault => {
        blueprint_id: (RESOURCE_PACKAGE, FUNGIBLE_VAULT_BLUEPRINT),
        functions: {
        },
        methods: {
            Take => (
                VaultTakeManifestInput,
                VAULT_TAKE_IDENT,
            ),
            TakeAdvanced => (
                VaultTakeAdvancedManifestInput,
                VAULT_TAKE_ADVANCED_IDENT,
            ),
            Put => (
                VaultPutManifestInput,
                VAULT_PUT_IDENT,
            ),
            GetAmount => (
                FungibleVaultGetAmountManifestInput,
                VAULT_GET_AMOUNT_IDENT,
            ),
            LockFee => (
                FungibleVaultLockFeeManifestInput,
                FUNGIBLE_VAULT_LOCK_FEE_IDENT,
            ),
            CreateProofOfAmount => (
                FungibleVaultCreateProofOfAmountManifestInput,
                FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_IDENT,
            ),
            LockAmount => (
                FungibleVaultLockFungibleAmountManifestInput,
                FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT,
            ),
            UnlockAmount => (
                FungibleVaultUnlockFungibleAmountManifestInput,
                FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT,
            ),
            Burn => (
                VaultBurnManifestInput,
                VAULT_BURN_IDENT,
            ),
        },
        direct_methods: {
            Recall => (
                VaultRecallManifestInput,
                VAULT_RECALL_IDENT,
            ),
            Freeze => (
                VaultFreezeManifestInput,
                VAULT_FREEZE_IDENT,
            ),
            Unfreeze => (
                VaultUnfreezeManifestInput,
                VAULT_UNFREEZE_IDENT,
            ),
        }
    },
    NonFungibleVault => {
        blueprint_id: (RESOURCE_PACKAGE, NON_FUNGIBLE_VAULT_BLUEPRINT),
        functions: {
        },
        methods: {
            Take => (
                VaultTakeManifestInput,
                VAULT_TAKE_IDENT,
            ),
            TakeAdvanced => (
                VaultTakeAdvancedManifestInput,
                VAULT_TAKE_ADVANCED_IDENT,
            ),
            TakeNonFungibles => (
                NonFungibleVaultTakeNonFungiblesManifestInput,
                NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT,
            ),
            Put => (
                VaultPutManifestInput,
                VAULT_PUT_IDENT,
            ),
            GetAmount => (
                VaultGetAmountManifestInput,
                VAULT_GET_AMOUNT_IDENT,
            ),
            GetNonFungibleLocalIds => (
                NonFungibleVaultGetNonFungibleLocalIdsManifestInput,
                NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
            ),
            ContainsNonFungible => (
                NonFungibleVaultContainsNonFungibleManifestInput,
                NON_FUNGIBLE_VAULT_CONTAINS_NON_FUNGIBLE_IDENT,
            ),
            CreateProofOfNonFungibles => (
                NonFungibleVaultCreateProofOfNonFungiblesManifestInput,
                NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            ),
            LockNonFungibles => (
                NonFungibleVaultLockNonFungiblesManifestInput,
                NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT,
            ),
            UnlockNonFungibles => (
                NonFungibleVaultUnlockNonFungiblesManifestInput,
                NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT,
            ),
            Burn => (
                VaultBurnManifestInput,
                VAULT_BURN_IDENT,
            ),
            BurnNonFungibles => (
                NonFungibleVaultBurnNonFungiblesManifestInput,
                NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT,
            ),
        },
        direct_methods: {
            Recall => (
                VaultRecallManifestInput,
                VAULT_RECALL_IDENT,
            ),
            Freeze => (
                VaultFreezeManifestInput,
                VAULT_FREEZE_IDENT,
            ),
            Unfreeze => (
                VaultUnfreezeManifestInput,
                VAULT_UNFREEZE_IDENT,
            ),
            RecallNonFungibles => (
                NonFungibleVaultRecallNonFungiblesManifestInput,
                NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT,
            ),
        }
    },
    TransactionTracker => {
        blueprint_id: (TRANSACTION_TRACKER_PACKAGE, TRANSACTION_TRACKER_BLUEPRINT),
        functions: {
            Create => (
                TransactionTrackerCreateManifestInput,
                TRANSACTION_TRACKER_CREATE_IDENT,
            ),
        },
        methods: {
        },
        direct_methods: {}
    },
    Metadata => {
        blueprint_id: (METADATA_MODULE_PACKAGE, METADATA_BLUEPRINT),
        functions: {
            Create => (
                MetadataCreateManifestInput,
                METADATA_CREATE_IDENT,
            ),
            CreateWithData => (
                MetadataCreateWithDataManifestInput,
                METADATA_CREATE_WITH_DATA_IDENT,
            ),
        },
        methods: {
            Set => (
                MetadataSetManifestInput,
                METADATA_SET_IDENT,
            ),
            Lock => (
                MetadataLockManifestInput,
                METADATA_LOCK_IDENT,
            ),
            Get => (
                MetadataGetManifestInput,
                METADATA_GET_IDENT,
            ),
            Remove => (
                MetadataRemoveManifestInput,
                METADATA_REMOVE_IDENT,
            ),
        },
        direct_methods: {}
    },
    RoleAssignment => {
        blueprint_id: (ROLE_ASSIGNMENT_MODULE_PACKAGE, ROLE_ASSIGNMENT_BLUEPRINT),
        functions: {
            Create => (
                RoleAssignmentCreateManifestInput,
                ROLE_ASSIGNMENT_CREATE_IDENT,
            ),
        },
        methods: {
            SetOwner => (
                RoleAssignmentSetOwnerManifestInput,
                ROLE_ASSIGNMENT_SET_OWNER_IDENT,
            ),
            LockOwner => (
                RoleAssignmentLockOwnerManifestInput,
                ROLE_ASSIGNMENT_LOCK_OWNER_IDENT,
            ),
            Set => (
                RoleAssignmentSetManifestInput,
                ROLE_ASSIGNMENT_SET_IDENT,
            ),
            Get => (
                RoleAssignmentGetManifestInput,
                ROLE_ASSIGNMENT_GET_IDENT,
            ),
        },
        direct_methods: {}
    },
    ComponentRoyalty => {
        blueprint_id: (ROYALTY_MODULE_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT),
        functions: {
            Create => (
                ComponentRoyaltyCreateManifestInput,
                COMPONENT_ROYALTY_CREATE_IDENT,
            ),
        },
        methods: {
            SetRoyalty => (
                ComponentRoyaltySetManifestInput,
                COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
            ),
            LockRoyalty => (
                ComponentRoyaltyLockManifestInput,
                COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT,
            ),
            ClaimRoyalties => (
                ComponentClaimRoyaltiesManifestInput,
                COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT,
            ),
        },
        direct_methods: {}
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedManifestNativeInvocationError {
    /// This error is returned when the arguments of some invocation could not be decoded as that
    /// invocation's type.
    FailedToDecodeFunctionInvocation {
        blueprint_id: BlueprintId,
        function_name: String,
        args: ManifestValue,
        error: String,
    },
    /// This error is returned when the arguments of some invocation could not be decoded as that
    /// invocation's type.
    FailedToDecodeMethodInvocation {
        blueprint_id: BlueprintId,
        method_name: String,
        args: ManifestValue,
        error: String,
    },
    /// This error is returned when the arguments of some invocation could not be decoded as that
    /// invocation's type.
    FailedToDecodeDirectMethodInvocation {
        blueprint_id: BlueprintId,
        method_name: String,
        args: ManifestValue,
        error: String,
    },
    /// This error is returned when the function doesn't exist on some blueprint.
    InvokedFunctionNotFoundOnNativeBlueprint {
        blueprint_id: BlueprintId,
        function: String,
    },
    /// This error is returned when the method doesn't exist on some blueprint.
    InvokedMethodNotFoundOnNativeBlueprint {
        blueprint_id: BlueprintId,
        method: String,
    },
    /// This error is returned when the direct method doesn't exist on some blueprint.
    InvokedDirectMethodNotFoundOnNativeBlueprint {
        blueprint_id: BlueprintId,
        method: String,
    },
}

fn decode_args<M: ManifestDecode>(args: &ManifestValue) -> Result<M, String> {
    let encoded = manifest_encode(&args).map_err(|error| format!("{error:#?}"))?;
    manifest_decode(&encoded).map_err(|error| format!("{error:#?}"))
}
