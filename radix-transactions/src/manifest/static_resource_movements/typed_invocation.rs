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

macro_rules! define_typed_invocations {
    (
        $(
            $package_name: ident => {
                package: $package_address: expr,
                $(
                    $blueprint_name: ident => {
                        entity_type_pat: $entity_type_pat: pat,
                        module_id: $module_id: expr,
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
            #[derive(Debug, ManifestSbor)]
            pub enum TypedNativeInvocation {
                $(
                    [< $package_name Package >]([< $package_name Invocations >])
                ),*
            }

            impl TypedNativeInvocation {
                pub fn from_method_invocation(
                    address: &GlobalAddress,
                    module_id: ModuleId,
                    method_name: &str,
                    args: &::radix_common::prelude::ManifestValue
                ) -> Result<Option<Self>, StaticResourceMovementsError> {
                    let typed_invocation = match (address.as_node_id().entity_type(), module_id) {
                        $(
                            $(
                                (Some($entity_type_pat), $module_id) => {
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
                    };
                    Ok(typed_invocation)
                }

                pub fn from_blueprint_method_invocation(
                    package_address: &PackageAddress,
                    blueprint_name: &str,
                    module_id: ModuleId,
                    method_name: &str,
                    args: &::radix_common::prelude::ManifestValue
                ) -> Result<Option<Self>, StaticResourceMovementsError> {
                    let typed_invocation = match (*package_address, blueprint_name, module_id) {
                        $(
                            $(
                                ($package_address, stringify!($blueprint_name), $module_id) => {
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
                    };
                    Ok(typed_invocation)
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
                #[derive(Debug, ManifestSbor)]
                pub enum [< $package_name Invocations >] {
                    $(
                        [< $blueprint_name Blueprint >]([< $blueprint_name BlueprintInvocations >])
                    ),*
                }

                $(
                    // For each blueprint we define a type that's made up of the method and function
                    #[derive(Debug, ManifestSbor)]
                    pub enum [< $blueprint_name BlueprintInvocations >] {
                        Function([< $blueprint_name Function >]),
                        Method([< $blueprint_name Method >]),
                    }

                    #[derive(Debug, ManifestSbor)]
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

                        #[allow(unused_variables, unreachable_code)]
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

                    #[derive(Debug, ManifestSbor)]
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
            module_id: ModuleId::Main,
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
        },
    },
    Account => {
        package: ACCOUNT_PACKAGE,
        Account => {
            entity_type_pat: EntityType::GlobalAccount
                | EntityType::GlobalPreallocatedSecp256k1Account
                | EntityType::GlobalPreallocatedEd25519Account,
            module_id: ModuleId::Main,
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
        },
    },
    ConsensusManager => {
        package: CONSENSUS_MANAGER_PACKAGE,
        ConsensusManager => {
            entity_type_pat: EntityType::GlobalConsensusManager,
            module_id: ModuleId::Main,
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
        },
        Validator => {
            entity_type_pat: EntityType::GlobalValidator,
            module_id: ModuleId::Main,
            functions: {
            },
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
        },
    },
    Identity => {
        package: IDENTITY_PACKAGE,
        Identity => {
            entity_type_pat: EntityType::GlobalIdentity
                | EntityType::GlobalPreallocatedSecp256k1Identity
                | EntityType::GlobalPreallocatedEd25519Identity,
            module_id: ModuleId::Main,
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
        },
    },
    Locker => {
        package: LOCKER_PACKAGE,
        AccountLocker => {
            entity_type_pat: EntityType::GlobalAccountLocker,
            module_id: ModuleId::Main,
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
        },
    },
    Package => {
        package: PACKAGE_PACKAGE,
        Package => {
            entity_type_pat: EntityType::GlobalPackage,
            module_id: ModuleId::Main,
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
        },
    },
    Pool => {
        package: POOL_PACKAGE,
        OneResourcePool => {
            entity_type_pat: EntityType::GlobalOneResourcePool,
            module_id: ModuleId::Main,
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
        },
        TwoResourcePool => {
            entity_type_pat: EntityType::GlobalTwoResourcePool,
            module_id: ModuleId::Main,
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
        },
        MultiResourcePool => {
            entity_type_pat: EntityType::GlobalMultiResourcePool,
            module_id: ModuleId::Main,
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
        },
    },
    Resource => {
        package: RESOURCE_PACKAGE,
        FungibleResourceManager => {
            entity_type_pat: EntityType::GlobalFungibleResourceManager,
            module_id: ModuleId::Main,
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
        },
        NonFungibleResourceManager => {
            entity_type_pat: EntityType::GlobalNonFungibleResourceManager,
            module_id: ModuleId::Main,
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
        },
        FungibleVault => {
            entity_type_pat: EntityType::InternalFungibleVault,
            module_id: ModuleId::Main,
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
        },
        NonFungibleVault => {
            entity_type_pat: EntityType::InternalNonFungibleVault,
            module_id: ModuleId::Main,
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
        },
    },
    TransactionTracker => {
        package: TRANSACTION_TRACKER_PACKAGE,
        TransactionTracker => {
            entity_type_pat: EntityType::GlobalTransactionTracker,
            module_id: ModuleId::Main,
            functions: {
                Create => (
                    TransactionTrackerCreateManifestInput,
                    TRANSACTION_TRACKER_CREATE_IDENT,
                ),
            },
            methods: {
            },
        },
    },
    Metadata => {
        package: METADATA_MODULE_PACKAGE,
        Metadata => {
            entity_type_pat: EntityType::GlobalPackage
                | EntityType::GlobalFungibleResourceManager
                | EntityType::GlobalNonFungibleResourceManager
                | EntityType::GlobalConsensusManager
                | EntityType::GlobalValidator
                | EntityType::GlobalAccessController
                | EntityType::GlobalAccount
                | EntityType::GlobalIdentity
                | EntityType::GlobalGenericComponent
                | EntityType::GlobalPreallocatedSecp256k1Account
                | EntityType::GlobalPreallocatedEd25519Account
                | EntityType::GlobalPreallocatedSecp256k1Identity
                | EntityType::GlobalPreallocatedEd25519Identity
                | EntityType::GlobalOneResourcePool
                | EntityType::GlobalTwoResourcePool
                | EntityType::GlobalMultiResourcePool
                | EntityType::GlobalTransactionTracker
                | EntityType::GlobalAccountLocker,
            module_id: ModuleId::Metadata,
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
        },
    },
    RoleAssignment => {
        package: ROLE_ASSIGNMENT_MODULE_PACKAGE,
        RoleAssignment => {
            entity_type_pat: EntityType::GlobalPackage
                | EntityType::GlobalFungibleResourceManager
                | EntityType::GlobalNonFungibleResourceManager
                | EntityType::GlobalConsensusManager
                | EntityType::GlobalValidator
                | EntityType::GlobalAccessController
                | EntityType::GlobalAccount
                | EntityType::GlobalIdentity
                | EntityType::GlobalGenericComponent
                | EntityType::GlobalPreallocatedSecp256k1Account
                | EntityType::GlobalPreallocatedEd25519Account
                | EntityType::GlobalPreallocatedSecp256k1Identity
                | EntityType::GlobalPreallocatedEd25519Identity
                | EntityType::GlobalOneResourcePool
                | EntityType::GlobalTwoResourcePool
                | EntityType::GlobalMultiResourcePool
                | EntityType::GlobalTransactionTracker
                | EntityType::GlobalAccountLocker,
            module_id: ModuleId::RoleAssignment,
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
        },
    },
    ComponentRoyalty => {
        package: ROYALTY_MODULE_PACKAGE,
        ComponentRoyalty => {
            entity_type_pat: EntityType::GlobalPackage
                | EntityType::GlobalFungibleResourceManager
                | EntityType::GlobalNonFungibleResourceManager
                | EntityType::GlobalConsensusManager
                | EntityType::GlobalValidator
                | EntityType::GlobalAccessController
                | EntityType::GlobalAccount
                | EntityType::GlobalIdentity
                | EntityType::GlobalGenericComponent
                | EntityType::GlobalPreallocatedSecp256k1Account
                | EntityType::GlobalPreallocatedEd25519Account
                | EntityType::GlobalPreallocatedSecp256k1Identity
                | EntityType::GlobalPreallocatedEd25519Identity
                | EntityType::GlobalOneResourcePool
                | EntityType::GlobalTwoResourcePool
                | EntityType::GlobalMultiResourcePool
                | EntityType::GlobalTransactionTracker
                | EntityType::GlobalAccountLocker,
            module_id: ModuleId::Royalty,
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
        },
    },
}
