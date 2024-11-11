use super::*;
use radix_common::prelude::*;

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

pub trait StaticInvocationResourcesOutput
where
    Self: ManifestEncode + ManifestDecode,
{
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError>;
}

pub struct InvocationDetails<'a> {
    pub receiver: InvocationReceiver,
    pub sent_resources: &'a TrackedResources,
    pub source: ChangeSource,
}

#[derive(Debug)]
pub enum InvocationReceiver {
    GlobalMethod(ResolvedDynamicAddress<GlobalAddress>),
    DirectAccess(InternalAddress),
    BlueprintFunction(BlueprintId),
}

macro_rules! no_output_static_invocation_resources_output_impl {
    (
        $(
            $output_ident: ident
        ),* $(,)?
    ) => {
        $(
            impl StaticInvocationResourcesOutput for $output_ident {
                fn output(
                    &self,
                    _details: InvocationDetails
                ) -> Result<TrackedResources, StaticResourceMovementsError> {
                    Ok(TrackedResources::new_empty())
                }
            }
        )*
    };
}

macro_rules! unknown_output_static_invocation_resources_output_impl {
    (
        $(
            $output_ident: ident
        ),* $(,)?
    ) => {
        $(
            impl StaticInvocationResourcesOutput for $output_ident {
                fn output(
                    &self,
                    details: InvocationDetails
                ) -> Result<TrackedResources, StaticResourceMovementsError> {
                    Ok(TrackedResources::new_with_possible_balance_of_unspecified_resources([
                        details.source
                    ]))
                }
            }
        )*
    };
}

no_output_static_invocation_resources_output_impl![
    // AccessController
    AccessControllerCreateManifestInput,
    AccessControllerCreateProofManifestInput,
    AccessControllerInitiateRecoveryAsPrimaryManifestInput,
    AccessControllerInitiateRecoveryAsRecoveryManifestInput,
    AccessControllerQuickConfirmPrimaryRoleRecoveryProposalManifestInput,
    AccessControllerQuickConfirmRecoveryRoleRecoveryProposalManifestInput,
    AccessControllerTimedConfirmRecoveryManifestInput,
    AccessControllerCancelPrimaryRoleRecoveryProposalManifestInput,
    AccessControllerCancelRecoveryRoleRecoveryProposalManifestInput,
    AccessControllerLockPrimaryRoleManifestInput,
    AccessControllerUnlockPrimaryRoleManifestInput,
    AccessControllerStopTimedRecoveryManifestInput,
    AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryManifestInput,
    AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryManifestInput,
    AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptManifestInput,
    AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptManifestInput,
    AccessControllerLockRecoveryFeeManifestInput,
    AccessControllerContributeRecoveryFeeManifestInput,
    // Account
    AccountCreateAdvancedManifestInput,
    AccountLockFeeManifestInput,
    AccountLockContingentFeeManifestInput,
    AccountDepositManifestInput,
    AccountDepositBatchManifestInput,
    AccountBurnManifestInput,
    AccountBurnNonFungiblesManifestInput,
    AccountCreateProofOfAmountManifestInput,
    AccountCreateProofOfNonFungiblesManifestInput,
    AccountSetDefaultDepositRuleManifestInput,
    AccountSetResourcePreferenceManifestInput,
    AccountRemoveResourcePreferenceManifestInput,
    AccountTryDepositOrAbortManifestInput,
    AccountTryDepositBatchOrAbortManifestInput,
    AccountAddAuthorizedDepositorManifestInput,
    AccountRemoveAuthorizedDepositorManifestInput,
    // ConsensusManager
    ConsensusManagerCreateManifestInput,
    ConsensusManagerGetCurrentEpochManifestInput,
    ConsensusManagerStartManifestInput,
    ConsensusManagerGetCurrentTimeManifestInputV1,
    ConsensusManagerGetCurrentTimeManifestInputV2,
    ConsensusManagerCompareCurrentTimeManifestInputV1,
    ConsensusManagerCompareCurrentTimeManifestInputV2,
    ConsensusManagerNextRoundManifestInput,
    // Validator
    ValidatorRegisterManifestInput,
    ValidatorUnregisterManifestInput,
    ValidatorUpdateKeyManifestInput,
    ValidatorUpdateFeeManifestInput,
    ValidatorUpdateAcceptDelegatedStakeManifestInput,
    ValidatorAcceptsDelegatedStakeManifestInput,
    ValidatorTotalStakeXrdAmountManifestInput,
    ValidatorTotalStakeUnitSupplyManifestInput,
    ValidatorGetRedemptionValueManifestInput,
    ValidatorSignalProtocolUpdateReadinessManifestInput,
    ValidatorGetProtocolUpdateReadinessManifestInput,
    ValidatorLockOwnerStakeUnitsManifestInput,
    ValidatorStartUnlockOwnerStakeUnitsManifestInput,
    ValidatorApplyEmissionManifestInput,
    ValidatorApplyRewardManifestInput,
    // Identity
    IdentityCreateAdvancedManifestInput,
    // AccountLocker
    AccountLockerInstantiateManifestInput,
    AccountLockerStoreManifestInput,
    AccountLockerGetAmountManifestInput,
    AccountLockerGetNonFungibleLocalIdsManifestInput,
    // Package
    PackagePublishWasmAdvancedManifestInput,
    PackagePublishNativeManifestInput,
    // OneResourcePool
    OneResourcePoolInstantiateManifestInput,
    OneResourcePoolProtectedDepositManifestInput,
    OneResourcePoolGetRedemptionValueManifestInput,
    OneResourcePoolGetVaultAmountManifestInput,
    // TwoResourcePool
    TwoResourcePoolInstantiateManifestInput,
    TwoResourcePoolProtectedDepositManifestInput,
    TwoResourcePoolGetRedemptionValueManifestInput,
    TwoResourcePoolGetVaultAmountsManifestInput,
    // MultiResourcePool
    MultiResourcePoolInstantiateManifestInput,
    MultiResourcePoolProtectedDepositManifestInput,
    MultiResourcePoolGetRedemptionValueManifestInput,
    MultiResourcePoolGetVaultAmountsManifestInput,
    // ResourceManager
    ResourceManagerBurnManifestInput,
    ResourceManagerPackageBurnManifestInput,
    ResourceManagerGetTotalSupplyManifestInput,
    ResourceManagerGetResourceTypeManifestInput,
    ResourceManagerCreateEmptyVaultManifestInput,
    ResourceManagerGetAmountForWithdrawalManifestInput,
    ResourceManagerDropEmptyBucketManifestInput,
    // FungibleResourceManager
    FungibleResourceManagerCreateManifestInput,
    // NonFungibleResourceManager
    NonFungibleResourceManagerCreateManifestInput,
    NonFungibleResourceManagerGetNonFungibleManifestInput,
    NonFungibleResourceManagerUpdateDataManifestInput,
    NonFungibleResourceManagerExistsManifestInput,
    // Vault
    VaultGetAmountManifestInput,
    VaultFreezeManifestInput,
    VaultUnfreezeManifestInput,
    VaultBurnManifestInput,
    VaultPutManifestInput,
    // FungibleVault
    FungibleVaultLockFeeManifestInput,
    FungibleVaultCreateProofOfAmountManifestInput,
    FungibleVaultLockFungibleAmountManifestInput,
    FungibleVaultUnlockFungibleAmountManifestInput,
    // NonFungibleVault
    NonFungibleVaultGetNonFungibleLocalIdsManifestInput,
    NonFungibleVaultContainsNonFungibleManifestInput,
    NonFungibleVaultCreateProofOfNonFungiblesManifestInput,
    NonFungibleVaultLockNonFungiblesManifestInput,
    NonFungibleVaultUnlockNonFungiblesManifestInput,
    NonFungibleVaultBurnNonFungiblesManifestInput,
    // TransactionTracker
    TransactionTrackerCreateManifestInput,
    // Metadata
    MetadataCreateManifestInput,
    MetadataCreateWithDataManifestInput,
    MetadataSetManifestInput,
    MetadataLockManifestInput,
    MetadataGetManifestInput,
    MetadataRemoveManifestInput,
    // RoleAssignment
    RoleAssignmentCreateManifestInput,
    RoleAssignmentSetOwnerManifestInput,
    RoleAssignmentLockOwnerManifestInput,
    RoleAssignmentSetManifestInput,
    RoleAssignmentGetManifestInput,
    // ComponentRoyalty
    ComponentRoyaltyCreateManifestInput,
    ComponentRoyaltySetManifestInput,
    ComponentRoyaltyLockManifestInput,
];

unknown_output_static_invocation_resources_output_impl![
    /* AccessController */
    // The withdrawn badge is of an unknown resource
    AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptManifestInput,
    // The withdrawn badge is of an unknown resource
    AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptManifestInput,
    // The minted badge is of a new / unknown resource
    AccessControllerMintRecoveryBadgesManifestInput,
    // The validator stake unit resource is unknown at static validation time
    /* Validator */
    ValidatorStakeAsOwnerManifestInput,
    // The validator stake unit resource is unknown at static validation time
    ValidatorStakeManifestInput,
    // The validator unstake receipt is unknown at static validation time
    ValidatorUnstakeManifestInput,
    // This can return validator stake units which are an unknown resource at static validation time
    ValidatorFinishUnlockOwnerStakeUnitsManifestInput,
    // This generates and returns a new badge resource, which is unknowable at static time
    /* AccountLocker */
    AccountLockerInstantiateSimpleManifestInput,
    /* OneResourcePool */
    // This returns pool units of an unknown resource address and an unknown amount.
    OneResourcePoolContributeManifestInput,
    // This returns unknown resources of an unknown amount from the redemption.
    OneResourcePoolRedeemManifestInput,
    // This returns an unknown resource but a known amount which we can't do much with.
    OneResourcePoolProtectedWithdrawManifestInput,
    /* TwoResourcePool */
    // This returns pool units of an unknown resource address and an unknown amount.
    TwoResourcePoolContributeManifestInput,
    // This returns unknown resources of an unknown amount from the redemption.
    TwoResourcePoolRedeemManifestInput,
    /* MultiResourcePool */
    // This returns pool units of an unknown resource address and an unknown amount.
    MultiResourcePoolContributeManifestInput,
    // This returns unknown resources of an unknown amount from the redemption.
    MultiResourcePoolRedeemManifestInput,
    /* FungibleResourceManager */
    // This returns this resource so we know the amount but we don't know the resource address
    // so we can't do much with that.
    FungibleResourceManagerCreateWithInitialSupplyManifestInput,
    /* NonFungibleResourceManager */
    // This returns this resource so we know the ids but we don't know the resource address
    // so we can't do much with that.
    NonFungibleResourceManagerCreateWithInitialSupplyManifestInput,
    // This returns this resource so we know the ids but we don't know the resource address
    // so we can't do much with that.
    NonFungibleResourceManagerCreateRuidWithInitialSupplyManifestInput,
    /* Vault */
    // We don't know what resource is in the vault. We know the amount/ids returned but not the
    // resource address.
    VaultTakeManifestInput,
    // We don't know what resource is in the vault. We know the amount/ids returned but not the
    // resource address.
    VaultTakeAdvancedManifestInput,
    // We don't know what resource is in the vault. We know the amount/ids returned but not the
    // resource address.
    VaultRecallManifestInput,
    /* NonFungibleVault */
    // We don't know what resource is in the vault. We know the amount/ids returned but not the
    // resource address.
    NonFungibleVaultTakeNonFungiblesManifestInput,
    // We don't know what resource is in the vault. We know the amount/ids returned but not the
    // resource address.
    NonFungibleVaultRecallNonFungiblesManifestInput,
];

// region:Typed Invocation
impl StaticInvocationResourcesOutput for TypedManifestNativeInvocation {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        uniform_match_on_manifest_typed_invocation!(self => (input) => input.output(details))
    }
}
// endregion:Typed Invocation

// region:AccessController
impl StaticInvocationResourcesOutput for AccessControllerWithdrawRecoveryFeeManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            XRD,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}
// endregion:AccessController

// region:Account
impl StaticInvocationResourcesOutput for AccountCreateManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            ACCOUNT_OWNER_BADGE,
            TrackedResource::exact_amount(1, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for AccountSecurifyManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        match details.receiver {
            InvocationReceiver::GlobalMethod(ResolvedDynamicAddress::StaticAddress(
                global_address,
            )) => {
                let local_id = NonFungibleLocalId::bytes(global_address.as_bytes()).unwrap();
                TrackedResources::new_empty().add_resource(
                    ACCOUNT_OWNER_BADGE,
                    TrackedResource::exact_non_fungibles([local_id], [details.source]),
                )
            }
            InvocationReceiver::GlobalMethod(
                ResolvedDynamicAddress::BlueprintResolvedFromNamedAddress(_),
            ) => TrackedResources::new_empty().add_resource(
                ACCOUNT_OWNER_BADGE,
                TrackedResource::exact_amount(1, [details.source])?,
            ),
            InvocationReceiver::DirectAccess(_) | InvocationReceiver::BlueprintFunction(_) => {
                unreachable!()
            }
        }
    }
}

impl StaticInvocationResourcesOutput for AccountWithdrawManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        let ManifestResourceAddress::Static(resource_address) = self.resource_address else {
            return Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            );
        };
        TrackedResources::new_empty().add_resource(
            resource_address,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for AccountWithdrawNonFungiblesManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        let ManifestResourceAddress::Static(resource_address) = self.resource_address else {
            return Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            );
        };
        TrackedResources::new_empty().add_resource(
            resource_address,
            TrackedResource::exact_non_fungibles(self.ids.clone(), [details.source]),
        )
    }
}

impl StaticInvocationResourcesOutput for AccountLockFeeAndWithdrawManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        let ManifestResourceAddress::Static(resource_address) = self.resource_address else {
            return Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            );
        };
        TrackedResources::new_empty().add_resource(
            resource_address,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for AccountLockFeeAndWithdrawNonFungiblesManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        let ManifestResourceAddress::Static(resource_address) = self.resource_address else {
            return Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            );
        };
        TrackedResources::new_empty().add_resource(
            resource_address,
            TrackedResource::exact_non_fungibles(self.ids.clone(), [details.source]),
        )
    }
}

impl StaticInvocationResourcesOutput for AccountTryDepositOrRefundManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        handle_possible_refund(details)
    }
}

impl StaticInvocationResourcesOutput for AccountTryDepositBatchOrRefundManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        handle_possible_refund(details)
    }
}

fn handle_possible_refund(
    details: InvocationDetails,
) -> Result<TrackedResources, StaticResourceMovementsError> {
    let mut sent_resources = details.sent_resources.clone();
    let mut refunded_resources = TrackedResources::new_empty();

    // Handle the specified resources. First dump the resource keys to work around the borrow checker...
    let known_resources = sent_resources
        .specified_resources()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    for known_resource in known_resources {
        let attempted_deposit = sent_resources.mut_take_resource(
            known_resource,
            ResourceTakeAmount::All,
            details.source,
        )?;
        let (bounds, _history) = attempted_deposit.deconstruct();
        // Either nothing or everything is returned, but we can't currently model a fork in
        // the timeline, so instead we handle it as a return of some amount between 0 and the sent amount.
        let refunded_amount = bounds.replace_lower_bounds_with_zero();
        refunded_resources.mut_add_resource(
            known_resource,
            TrackedResource::general(refunded_amount, [details.source]),
        )?;
    }
    // Handle the possible refund of the remaining unspecified resources
    if sent_resources.unspecified_resources().may_be_present() {
        refunded_resources.mut_add_unspecified_resources([details.source]);
    }

    Ok(refunded_resources)
}
// endregion:Account

// region:ConsensusManager
impl StaticInvocationResourcesOutput for ConsensusManagerCreateValidatorManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            VALIDATOR_OWNER_BADGE,
            TrackedResource::exact_amount(1, [details.source])?,
        )
    }
}
// endregion:ConsensusManager

// region:Validator
impl StaticInvocationResourcesOutput for ValidatorClaimXrdManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty()
            .add_resource(XRD, TrackedResource::zero_or_more([details.source]))
    }
}

// endregion:Validator

// region:Identity
impl StaticInvocationResourcesOutput for IdentityCreateManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            IDENTITY_OWNER_BADGE,
            TrackedResource::exact_amount(1, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for IdentitySecurifyToSingleBadgeManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        Ok(match details.receiver {
            InvocationReceiver::GlobalMethod(ResolvedDynamicAddress::StaticAddress(
                global_address,
            )) => {
                let local_id = NonFungibleLocalId::bytes(global_address.as_bytes()).unwrap();
                TrackedResources::new_empty().add_resource(
                    IDENTITY_OWNER_BADGE,
                    TrackedResource::exact_non_fungibles([local_id], [details.source]),
                )?
            }
            InvocationReceiver::GlobalMethod(
                ResolvedDynamicAddress::BlueprintResolvedFromNamedAddress(_),
            ) => TrackedResources::new_empty().add_resource(
                IDENTITY_OWNER_BADGE,
                TrackedResource::exact_amount(1, [details.source])?,
            )?,
            InvocationReceiver::DirectAccess(_) | InvocationReceiver::BlueprintFunction(_) => {
                unreachable!()
            }
        })
    }
}
// endregion:Identity

// region:AccountLocker
impl StaticInvocationResourcesOutput for AccountLockerAirdropManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // This behaves roughly like a possible refund...
        // We could be even more exact... We can subtract the claimants from the bucket to calculate what gets returned.
        // But this is good enough for now.
        handle_possible_refund(details)
    }
}

impl StaticInvocationResourcesOutput for AccountLockerRecoverManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        let ManifestResourceAddress::Static(resource_address) = self.resource_address else {
            return Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            );
        };
        TrackedResources::new_empty().add_resource(
            resource_address,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for AccountLockerRecoverNonFungiblesManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        let ManifestResourceAddress::Static(resource_address) = self.resource_address else {
            return Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            );
        };
        TrackedResources::new_empty().add_resource(
            resource_address,
            TrackedResource::exact_non_fungibles(self.ids.clone(), [details.source]),
        )
    }
}

impl StaticInvocationResourcesOutput for AccountLockerClaimManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        let ManifestResourceAddress::Static(resource_address) = self.resource_address else {
            return Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            );
        };
        TrackedResources::new_empty().add_resource(
            resource_address,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for AccountLockerClaimNonFungiblesManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        let ManifestResourceAddress::Static(resource_address) = self.resource_address else {
            return Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            );
        };
        TrackedResources::new_empty().add_resource(
            resource_address,
            TrackedResource::exact_non_fungibles(self.ids.clone(), [details.source]),
        )
    }
}
// endregion:AccountLocker

// region:Package
impl StaticInvocationResourcesOutput for PackagePublishWasmManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            PACKAGE_OWNER_BADGE,
            TrackedResource::exact_amount(1, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for PackageClaimRoyaltiesManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty()
            .add_resource(XRD, TrackedResource::zero_or_more([details.source]))
    }
}
// endregion:Package

// region:TwoResourcePool
impl StaticInvocationResourcesOutput for TwoResourcePoolProtectedWithdrawManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        let ManifestResourceAddress::Static(resource_address) = self.resource_address else {
            return Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            );
        };
        TrackedResources::new_empty().add_resource(
            resource_address,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}
// endregion:TwoResourcePool

// region:MultiResourcePool
impl StaticInvocationResourcesOutput for MultiResourcePoolProtectedWithdrawManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        let ManifestResourceAddress::Static(resource_address) = self.resource_address else {
            return Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            );
        };
        TrackedResources::new_empty().add_resource(
            resource_address,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}
// endregion:MultiResourcePool

// region:FungibleResourceManager
impl StaticInvocationResourcesOutput for FungibleResourceManagerMintManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // If the receiver is a global address then we can return something useful. Otherwise it
        // is a known amount and an unknown resource address.
        match details.receiver {
            InvocationReceiver::GlobalMethod(ResolvedDynamicAddress::StaticAddress(
                global_address,
            )) => {
                // Attempt to convert the global address to a resource address. Error if that fails.
                if let Ok(resource_address) = ResourceAddress::try_from(global_address) {
                    TrackedResources::new_empty().add_resource(
                        resource_address,
                        TrackedResource::exact_amount(self.amount, [details.source])?,
                    )
                } else {
                    Err(StaticResourceMovementsError::NotAResourceAddress(
                        global_address,
                    ))
                }
            }
            InvocationReceiver::GlobalMethod(
                ResolvedDynamicAddress::BlueprintResolvedFromNamedAddress(_),
            )
            | InvocationReceiver::DirectAccess(_)
            | InvocationReceiver::BlueprintFunction(_) => Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            ),
        }
    }
}

impl StaticInvocationResourcesOutput for ResourceManagerCreateEmptyBucketInput {
    fn output(
        &self,
        _: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // An empty bucket is returned so we just return an empty set of `TrackedResources`. I have
        // this as a manual implementation instead of one of the invocations in the macro invocation
        // because NONE of the invocations there return a bucket while this invocation returns a
        // bucket. To be consistent and have all invocations returning a bucket have a manual impl
        // I'm keeping this manual and hand-written implementation with a comment on why it's here
        // and why this invocation returns nothing.
        Ok(TrackedResources::new_empty())
    }
}
// endregion:FungibleResourceManager

// region:NonFungibleResourceManager
impl StaticInvocationResourcesOutput for NonFungibleResourceManagerMintManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // If the receiver is a global address then we can return something useful. Otherwise it
        // is a known ids and an unknown resource address.
        match details.receiver {
            InvocationReceiver::GlobalMethod(ResolvedDynamicAddress::StaticAddress(
                global_address,
            )) => {
                // Attempt to convert the global address to a resource address. Error if that fails.
                if let Ok(resource_address) = ResourceAddress::try_from(global_address) {
                    TrackedResources::new_empty().add_resource(
                        resource_address,
                        TrackedResource::exact_non_fungibles(
                            self.entries.keys().cloned(),
                            [details.source],
                        ),
                    )
                } else {
                    Err(StaticResourceMovementsError::NotAResourceAddress(
                        global_address,
                    ))
                }
            }
            InvocationReceiver::GlobalMethod(
                ResolvedDynamicAddress::BlueprintResolvedFromNamedAddress(_),
            )
            | InvocationReceiver::DirectAccess(_)
            | InvocationReceiver::BlueprintFunction(_) => Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            ),
        }
    }
}

impl StaticInvocationResourcesOutput for NonFungibleResourceManagerMintRuidManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // If the receiver is a global address then we can return something useful. Otherwise it
        // is a known amount and an unknown resource address.
        match details.receiver {
            InvocationReceiver::GlobalMethod(ResolvedDynamicAddress::StaticAddress(
                global_address,
            )) => {
                // Attempt to convert the global address to a resource address. Error if that fails.
                if let Ok(resource_address) = ResourceAddress::try_from(global_address) {
                    TrackedResources::new_empty().add_resource(
                        resource_address,
                        TrackedResource::exact_amount(self.entries.len(), [details.source])?,
                    )
                } else {
                    Err(StaticResourceMovementsError::NotAResourceAddress(
                        global_address,
                    ))
                }
            }
            InvocationReceiver::GlobalMethod(
                ResolvedDynamicAddress::BlueprintResolvedFromNamedAddress(_),
            )
            | InvocationReceiver::DirectAccess(_)
            | InvocationReceiver::BlueprintFunction(_) => Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            ),
        }
    }
}

impl StaticInvocationResourcesOutput for NonFungibleResourceManagerMintSingleRuidManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // If the receiver is a global address then we can return something useful. Otherwise it
        // is a known amount and an unknown resource address.
        match details.receiver {
            InvocationReceiver::GlobalMethod(ResolvedDynamicAddress::StaticAddress(
                global_address,
            )) => {
                // Attempt to convert the global address to a resource address. Error if that fails.
                if let Ok(resource_address) = ResourceAddress::try_from(global_address) {
                    TrackedResources::new_empty().add_resource(
                        resource_address,
                        TrackedResource::exact_amount(Decimal::ONE, [details.source])?,
                    )
                } else {
                    Err(StaticResourceMovementsError::NotAResourceAddress(
                        global_address,
                    ))
                }
            }
            InvocationReceiver::GlobalMethod(
                ResolvedDynamicAddress::BlueprintResolvedFromNamedAddress(_),
            )
            | InvocationReceiver::DirectAccess(_)
            | InvocationReceiver::BlueprintFunction(_) => Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            ),
        }
    }
}
// endregion:NonFungibleResourceManager

// region:ComponentRoyalty
impl StaticInvocationResourcesOutput for ComponentClaimRoyaltiesManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty()
            .add_resource(XRD, TrackedResource::zero_or_more([details.source]))
    }
}
// endregion:ComponentRoyalty
