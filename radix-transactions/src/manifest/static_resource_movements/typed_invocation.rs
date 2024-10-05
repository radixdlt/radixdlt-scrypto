use radix_engine_interface::blueprints::access_controller;
use radix_engine_interface::blueprints::account;
use radix_engine_interface::blueprints::consensus_manager;
use radix_engine_interface::blueprints::identity;
use radix_engine_interface::blueprints::locker;

use super::*;
use radix_common::prelude::*;

macro_rules! define_typed_invocations {
    (
        $(
            $package_name: ident => {
                package: $package_address: expr,
                $(
                    $blueprint_name: ident => {
                        entity_type_pat: $entity_type_pat: pat,
                        functions: {
                            $(
                                $func_ident: ident => ($func_input: ty, $func_name: expr $(,)?)
                            ),* $(,)?
                        },
                        methods: {
                            $(
                                $method_ident: ident => ($method_input: ty, $method_name: expr $(,)?)
                            ),* $(,)?
                        } $(,)?
                    }
                ),* $(,)?
            }
        ),* $(,)?
    ) => {
        paste::paste! {
            // There's a single typed invocation type that captures all of
            // the packages that we support.
            #[derive(Debug)]
            pub enum TypedNativeInvocation {
                $(
                    [< $package_name Package >]([< $package_name Invocations >])
                ),*
            }

            impl TypedNativeInvocation {
                pub fn from_main_module_method_invocation(
                    address: &GlobalAddress,
                    method_name: &str,
                    args: &::radix_common::prelude::ManifestValue
                ) -> Result<Option<Self>, StaticResourceMovementsError> {
                    Ok(match address.as_node_id().entity_type() {
                        $(
                            $(
                                Some($entity_type_pat) => {
                                    let invocation = [< $blueprint_name Method >]::from_invocation(method_name, args)?;
                                    Some(Self::[< $package_name Package >](
                                        [< $package_name Invocations >]::[< $blueprint_name Blueprint >](
                                            [< $blueprint_name BlueprintInvocations >]::Method(invocation)
                                        )
                                    ))
                                }
                            )*
                        )*
                        _ => None
                    })
                }

                pub fn from_blueprint_method_invocation(
                    package_address: &PackageAddress,
                    blueprint_name: &str,
                    method_name: &str,
                    args: &::radix_common::prelude::ManifestValue
                ) -> Result<Option<Self>, StaticResourceMovementsError> {
                    Ok(match *package_address {
                        $($package_address => match blueprint_name {
                            $(stringify!($blueprint_name) => {
                                let invocation = [< $blueprint_name Method >]::from_invocation(method_name, args)?;
                                Some(Self::[< $package_name Package >](
                                    [< $package_name Invocations >]::[< $blueprint_name Blueprint >](
                                        [< $blueprint_name BlueprintInvocations >]::Method(invocation)
                                    )
                                ))
                            },)*
                            _ => return Err(StaticResourceMovementsError::UnknownNativeBlueprint {
                                package: $package_address,
                                blueprint: blueprint_name.to_string(),
                            }),
                        },)*
                        _ => None,
                    })
                }

                pub fn from_function_invocation(
                    package_address: &PackageAddress,
                    blueprint_name: &str,
                    function_name: &str,
                    args: &::radix_common::prelude::ManifestValue
                ) -> Result<Option<Self>, StaticResourceMovementsError> {
                    Ok(match *package_address {
                        $($package_address => match blueprint_name {
                            $(stringify!($blueprint_name) => {
                                let invocation = [< $blueprint_name Function >]::from_invocation(function_name, args)?;
                                Some(Self::[< $package_name Package >](
                                    [< $package_name Invocations >]::[< $blueprint_name Blueprint >](
                                        [< $blueprint_name BlueprintInvocations >]::Function(invocation)
                                    )
                                ))
                            },)*
                            _ => return Err(StaticResourceMovementsError::UnknownNativeBlueprint {
                                package: $package_address,
                                blueprint: blueprint_name.to_string(),
                            }),
                        },)*
                        _ => None,
                    })
                }
            }

            $(
                // For each package we define an invocation type that has all of the blueprints that
                // this package has.
                #[derive(Debug)]
                pub enum [< $package_name Invocations >] {
                    $(
                        [< $blueprint_name Blueprint >]([< $blueprint_name BlueprintInvocations >])
                    ),*
                }

                $(
                    // For each blueprint we define a type that's made up of the method and function
                    #[derive(Debug)]
                    pub enum [< $blueprint_name BlueprintInvocations >] {
                        Function([< $blueprint_name Function >]),
                        Method([< $blueprint_name Method >]),
                    }

                    #[derive(Debug)]
                    pub enum [< $blueprint_name Method >] {
                        $(
                            $method_ident($method_input)
                        ),*
                    }

                    impl [< $blueprint_name Method >] {
                        #[allow(unreachable_patterns)]
                        pub fn method_name(&self) -> &str {
                            match self {
                                $(
                                    Self::$method_ident(..) => $method_name,
                                )*
                                // AVOIDS [E0004] when the enum is empty "note: references are always considered inhabited"
                                // https://github.com/rust-lang/unsafe-code-guidelines/issues/413
                                _ => unreachable!()
                            }
                        }

                        pub fn from_invocation(
                            method_name: &str,
                            args: &::radix_common::prelude::ManifestValue
                        ) -> Result<Self, StaticResourceMovementsError> {
                            Ok(match method_name {
                                $(
                                    $method_name => {
                                        let encoded = ::radix_common::prelude::manifest_encode(args)
                                            .map_err(StaticResourceMovementsError::NativeArgumentsEncodeError)?;
                                        let decoded = ::radix_common::prelude::manifest_decode(&encoded)
                                            .map_err(StaticResourceMovementsError::NativeArgumentsDecodeError)?;
                                        Self::$method_ident(decoded)
                                    }
                                )*
                                _ => return Err(StaticResourceMovementsError::UnknownNativeMethod {
                                    package: $package_address,
                                    blueprint: stringify!($blueprint_name).to_string(),
                                    method: method_name.to_string(),
                                })
                            })
                        }
                    }

                    #[derive(Debug)]
                    pub enum [< $blueprint_name Function >] {
                        $(
                            $func_ident($func_input)
                        ),*
                    }

                    impl [< $blueprint_name Function >] {
                        #[allow(unreachable_patterns)]
                        pub fn function_name(&self) -> &str {
                            match self {
                                $(
                                    Self::$func_ident(..) => $func_name,
                                )*
                                // AVOIDS [E0004] when the enum is empty "note: references are always considered inhabited"
                                // https://github.com/rust-lang/unsafe-code-guidelines/issues/413
                                _ => unreachable!()
                            }
                        }

                        #[allow(unused_variables, unreachable_code)]
                        pub fn from_invocation(
                            function_name: &str,
                            args: &::radix_common::prelude::ManifestValue
                        ) -> Result<Self, StaticResourceMovementsError> {
                            Ok(match function_name {
                                $(
                                    $func_name => {
                                        let encoded = ::radix_common::prelude::manifest_encode(args)
                                            .map_err(StaticResourceMovementsError::NativeArgumentsEncodeError)?;
                                        let decoded = ::radix_common::prelude::manifest_decode(&encoded)
                                            .map_err(StaticResourceMovementsError::NativeArgumentsDecodeError)?;
                                        Self::$func_ident(decoded)
                                    }
                                )*
                                _ => return Err(StaticResourceMovementsError::UnknownNativeFunction {
                                    package: $package_address,
                                    blueprint: stringify!($blueprint_name).to_string(),
                                    function: function_name.to_string(),
                                })
                            })
                        }
                    }
                )*
            )*
        }
    };
}

define_typed_invocations! {
    AccessController => {
        package: ACCESS_CONTROLLER_PACKAGE,
        AccessController => {
            entity_type_pat: EntityType::GlobalAccessController,
            functions: {
                Create => (
                    access_controller::AccessControllerCreateManifestInput,
                    access_controller::ACCESS_CONTROLLER_CREATE_IDENT
                )
            },
            methods: {
                CreateProof => (
                    access_controller::AccessControllerCreateProofInput,
                    access_controller::ACCESS_CONTROLLER_CREATE_PROOF_IDENT
                ),
                InitiateRecoveryAsPrimary => (
                    access_controller::AccessControllerInitiateRecoveryAsPrimaryInput,
                    access_controller::ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT
                ),
                InitiateRecoveryAsRecovery => (
                    access_controller::AccessControllerInitiateRecoveryAsRecoveryInput,
                    access_controller::ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT
                ),
                QuickConfirmPrimaryRoleRecoveryProposal => (
                    access_controller::AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput,
                    access_controller::ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT
                ),
                QuickConfirmRecoveryRoleRecoveryProposal => (
                    access_controller::AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput,
                    access_controller::ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT
                ),
                TimedConfirmRecovery => (
                    access_controller::AccessControllerTimedConfirmRecoveryInput,
                    access_controller::ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT
                ),
                StopTimedRecovery => (
                    access_controller::AccessControllerStopTimedRecoveryInput,
                    access_controller::ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT
                ),
                MintRecoveryBadges => (
                    access_controller::AccessControllerMintRecoveryBadgesInput,
                    access_controller::ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT
                ),
                LockRecoveryFee => (
                    access_controller::AccessControllerLockRecoveryFeeInput,
                    access_controller::ACCESS_CONTROLLER_LOCK_RECOVERY_FEE_IDENT
                ),
                WithdrawRecoveryFee => (
                    access_controller::AccessControllerWithdrawRecoveryFeeInput,
                    access_controller::ACCESS_CONTROLLER_WITHDRAW_RECOVERY_FEE_IDENT
                ),
                ContributeRecoveryFee => (
                    access_controller::AccessControllerContributeRecoveryFeeManifestInput,
                    access_controller::ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT
                ),
                InitiateBadgeWithdrawAttemptAsPrimary => (
                    access_controller::AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput,
                    access_controller::ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT
                ),
                InitiateBadgeWithdrawAttemptAsRecovery => (
                    access_controller::AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput,
                    access_controller::ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT
                ),
                QuickConfirmPrimaryRoleBadgeWithdrawAttempt => (
                    access_controller::AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptInput,
                    access_controller::ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT
                ),
                QuickConfirmRecoveryRoleBadgeWithdrawAttempt => (
                    access_controller::AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptInput,
                    access_controller::ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT
                ),
                CancelPrimaryRoleRecoveryProposal => (
                    access_controller::AccessControllerCancelPrimaryRoleRecoveryProposalInput,
                    access_controller::ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT
                ),
                CancelRecoveryRoleRecoveryProposal => (
                    access_controller::AccessControllerCancelRecoveryRoleRecoveryProposalInput,
                    access_controller::ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT
                ),
                CancelPrimaryRoleBadgeWithdrawAttempt => (
                    access_controller::AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptInput,
                    access_controller::ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT
                ),
                CancelRecoveryRoleBadgeWithdrawAttempt => (
                    access_controller::AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptInput,
                    access_controller::ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT
                ),
                LockPrimaryRole => (
                    access_controller::AccessControllerLockPrimaryRoleInput,
                    access_controller::ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT
                ),
                UnlockPrimaryRole => (
                    access_controller::AccessControllerUnlockPrimaryRoleInput,
                    access_controller::ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT
                ),
            }
        }
    },
    Account => {
        package: ACCOUNT_PACKAGE,
        Account => {
            entity_type_pat:
                EntityType::GlobalAccount
                | EntityType::GlobalPreallocatedEd25519Account
                | EntityType::GlobalPreallocatedSecp256k1Account,
            functions: {
                Create => (
                    account::AccountCreateInput,
                    account::ACCOUNT_CREATE_IDENT
                ),
                CreateAdvanced => (
                    account::AccountCreateAdvancedManifestInput,
                    account::ACCOUNT_CREATE_ADVANCED_IDENT
                ),
            },
            methods: {
                Securify => (
                    account::AccountSecurifyInput,
                    account::ACCOUNT_SECURIFY_IDENT
                ),
                LockFee => (
                    account::AccountLockFeeInput,
                    account::ACCOUNT_LOCK_FEE_IDENT
                ),
                LockContingentFee => (
                    account::AccountLockContingentFeeInput,
                    account::ACCOUNT_LOCK_CONTINGENT_FEE_IDENT
                ),
                Deposit => (
                    account::AccountDepositManifestInput,
                    account::ACCOUNT_DEPOSIT_IDENT
                ),
                DepositBatch => (
                    account::AccountDepositBatchManifestInput,
                    account::ACCOUNT_DEPOSIT_BATCH_IDENT
                ),
                Withdraw => (
                    account::AccountWithdrawInput,
                    account::ACCOUNT_WITHDRAW_IDENT
                ),
                WithdrawNonFungibles => (
                    account::AccountWithdrawNonFungiblesInput,
                    account::ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT
                ),
                LockFeeAndWithdraw => (
                    account::AccountLockFeeAndWithdrawInput,
                    account::ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT
                ),
                LockFeeAndWithdrawNonFungibles => (
                    account::AccountLockFeeAndWithdrawNonFungiblesInput,
                    account::ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT
                ),
                CreateProofOfAmount => (
                    account::AccountCreateProofOfAmountInput,
                    account::ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT
                ),
                CreateProofOfNonFungibles => (
                    account::AccountCreateProofOfNonFungiblesInput,
                    account::ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT
                ),
                SetDefaultDepositRule => (
                    account::AccountSetDefaultDepositRuleInput,
                    account::ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT
                ),
                SetResourcePreference => (
                    account::AccountSetResourcePreferenceInput,
                    account::ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT
                ),
                RemoveResourcePreference => (
                    account::AccountRemoveResourcePreferenceInput,
                    account::ACCOUNT_REMOVE_RESOURCE_PREFERENCE_IDENT
                ),
                TryDepositOrRefund => (
                    account::AccountTryDepositOrRefundManifestInput,
                    account::ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT
                ),
                TryDepositBatchOrRefund => (
                    account::AccountTryDepositBatchOrRefundManifestInput,
                    account::ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT
                ),
                TryDepositOrAbort => (
                    account::AccountTryDepositOrAbortManifestInput,
                    account::ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT
                ),
                TryDepositBatchOrAbort => (
                    account::AccountTryDepositBatchOrAbortManifestInput,
                    account::ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT
                ),
                Burn => (
                    account::AccountBurnInput,
                    account::ACCOUNT_BURN_IDENT
                ),
                BurnNonFungibles => (
                    account::AccountBurnNonFungiblesInput,
                    account::ACCOUNT_BURN_NON_FUNGIBLES_IDENT
                ),
                AddAuthorizedDepositor => (
                    account::AccountAddAuthorizedDepositorInput,
                    account::ACCOUNT_ADD_AUTHORIZED_DEPOSITOR
                ),
                RemoveAuthorizedDepositor => (
                    account::AccountRemoveAuthorizedDepositorInput,
                    account::ACCOUNT_REMOVE_AUTHORIZED_DEPOSITOR
                )
            },
        },
    },
    ConsensusManager => {
        package: CONSENSUS_MANAGER_PACKAGE,
        Validator => {
            entity_type_pat: EntityType::GlobalValidator,
            functions: {},
            methods: {
                Register => (
                    consensus_manager::ValidatorRegisterInput,
                    consensus_manager::VALIDATOR_REGISTER_IDENT,
                ),
                Unregister => (
                    consensus_manager::ValidatorUnregisterInput,
                    consensus_manager::VALIDATOR_UNREGISTER_IDENT,
                ),
                StakeAsOwner => (
                    consensus_manager::ValidatorStakeAsOwnerManifestInput,
                    consensus_manager::VALIDATOR_STAKE_AS_OWNER_IDENT,
                ),
                Stake => (
                    consensus_manager::ValidatorStakeManifestInput,
                    consensus_manager::VALIDATOR_STAKE_IDENT,
                ),
                Unstake => (
                    consensus_manager::ValidatorUnstakeManifestInput,
                    consensus_manager::VALIDATOR_UNSTAKE_IDENT,
                ),
                ClaimXrd => (
                    consensus_manager::ValidatorClaimXrdManifestInput,
                    consensus_manager::VALIDATOR_CLAIM_XRD_IDENT,
                ),
                UpdateKey => (
                    consensus_manager::ValidatorUpdateKeyInput,
                    consensus_manager::VALIDATOR_UPDATE_KEY_IDENT,
                ),
                UpdateFee => (
                    consensus_manager::ValidatorUpdateFeeInput,
                    consensus_manager::VALIDATOR_UPDATE_FEE_IDENT,
                ),
                UpdateAcceptDelegatedStake => (
                    consensus_manager::ValidatorUpdateAcceptDelegatedStakeInput,
                    consensus_manager::VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT,
                ),
                AcceptsDelegatedStake => (
                    consensus_manager::ValidatorAcceptsDelegatedStakeInput,
                    consensus_manager::VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT,
                ),
                TotalStakeXrdAmount => (
                    consensus_manager::ValidatorTotalStakeXrdAmountInput,
                    consensus_manager::VALIDATOR_TOTAL_STAKE_XRD_AMOUNT_IDENT,
                ),
                TotalStakeUnitSupply => (
                    consensus_manager::ValidatorTotalStakeUnitSupplyInput,
                    consensus_manager::VALIDATOR_TOTAL_STAKE_UNIT_SUPPLY_IDENT,
                ),
                GetRedemptionValue => (
                    consensus_manager::ValidatorGetRedemptionValueInput,
                    consensus_manager::VALIDATOR_GET_REDEMPTION_VALUE_IDENT,
                ),
                SignalProtocolUpdateReadiness => (
                    consensus_manager::ValidatorSignalProtocolUpdateReadinessInput,
                    consensus_manager::VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS,
                ),
                GetProtocolUpdateReadiness => (
                    consensus_manager::ValidatorGetProtocolUpdateReadinessInput,
                    consensus_manager::VALIDATOR_GET_PROTOCOL_UPDATE_READINESS_IDENT,
                ),
                LockOwnerStakeUnits => (
                    consensus_manager::ValidatorLockOwnerStakeUnitsManifestInput,
                    consensus_manager::VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT,
                ),
                StartUnlockOwnerStakeUnits => (
                    consensus_manager::ValidatorStartUnlockOwnerStakeUnitsInput,
                    consensus_manager::VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT,
                ),
                FinishUnlockOwnerStakeUnits => (
                    consensus_manager::ValidatorFinishUnlockOwnerStakeUnitsInput,
                    consensus_manager::VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT,
                ),
            }
        },
        ConsensusManager => {
            entity_type_pat: EntityType::GlobalConsensusManager,
            functions: {
                Create => (
                    consensus_manager::ConsensusManagerCreateManifestInput,
                    consensus_manager::CONSENSUS_MANAGER_CREATE_IDENT,
                ),
            },
            methods: {
                GetCurrentEpoch => (
                    consensus_manager::ConsensusManagerGetCurrentEpochInput,
                    consensus_manager::CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
                ),
                Start => (
                    consensus_manager::ConsensusManagerStartInput,
                    consensus_manager::CONSENSUS_MANAGER_START_IDENT,
                ),
                GetCurrentTime => (
                    consensus_manager::ConsensusManagerGetCurrentTimeInputV2,
                    consensus_manager::CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT,
                ),
                NextRound => (
                    consensus_manager::ConsensusManagerNextRoundInput,
                    consensus_manager::CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
                ),
                CreateValidator => (
                    consensus_manager::ConsensusManagerCreateValidatorManifestInput,
                    consensus_manager::CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT,
                ),
            }
        }
    },
    Identity => {
        package: IDENTITY_PACKAGE,
        Identity => {
            entity_type_pat:
                EntityType::GlobalIdentity
                | EntityType::GlobalPreallocatedEd25519Identity
                | EntityType::GlobalPreallocatedSecp256k1Identity,
            functions: {
                Create => (
                    identity::IdentityCreateInput,
                    identity::IDENTITY_CREATE_IDENT
                ),
                CreateAdvanced => (
                    identity::IdentityCreateAdvancedInput,
                    identity::IDENTITY_CREATE_ADVANCED_IDENT
                ),
            },
            methods: {
                Securify => (
                    identity::IdentitySecurifyToSingleBadgeInput,
                    identity::IDENTITY_SECURIFY_IDENT
                )
            },
        },
    },
    Locker => {
        package: LOCKER_PACKAGE,
        AccountLocker => {
            entity_type_pat: EntityType::GlobalAccountLocker,
            functions: {
                Instantiate => (
                    locker::AccountLockerInstantiateManifestInput,
                    locker::ACCOUNT_LOCKER_INSTANTIATE_IDENT,
                ),
                InstantiateSimple => (
                    locker::AccountLockerInstantiateSimpleManifestInput,
                    locker::ACCOUNT_LOCKER_INSTANTIATE_SIMPLE_IDENT,
                ),
            },
            methods: {
                Store => (
                    locker::AccountLockerStoreManifestInput,
                    locker::ACCOUNT_LOCKER_STORE_IDENT,
                ),
                Airdrop => (
                    locker::AccountLockerAirdropManifestInput,
                    locker::ACCOUNT_LOCKER_AIRDROP_IDENT,
                ),
                Recover => (
                    locker::AccountLockerRecoverManifestInput,
                    locker::ACCOUNT_LOCKER_RECOVER_IDENT,
                ),
                RecoverNonFungibles => (
                    locker::AccountLockerRecoverNonFungiblesManifestInput,
                    locker::ACCOUNT_LOCKER_RECOVER_NON_FUNGIBLES_IDENT,
                ),
                Claim => (
                    locker::AccountLockerClaimManifestInput,
                    locker::ACCOUNT_LOCKER_CLAIM_IDENT,
                ),
                ClaimNonFungibles => (
                    locker::AccountLockerClaimNonFungiblesManifestInput,
                    locker::ACCOUNT_LOCKER_CLAIM_NON_FUNGIBLES_IDENT,
                ),
                GetAmount => (
                    locker::AccountLockerGetAmountManifestInput,
                    locker::ACCOUNT_LOCKER_GET_AMOUNT_IDENT,
                ),
                GetNonFungibleLocalIds => (
                    locker::AccountLockerGetNonFungibleLocalIdsManifestInput,
                    locker::ACCOUNT_LOCKER_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
                ),
            },
        }
    }
}
