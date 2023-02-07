use crate::api::component::*;
use crate::api::node_modules::auth::*;
use crate::api::node_modules::metadata::*;
use crate::api::package::*;
use crate::api::types::*;
use crate::blueprints::access_controller::*;
use crate::blueprints::account::*;
use crate::blueprints::clock::*;
use crate::blueprints::epoch_manager::*;
use crate::blueprints::logger::*;
use crate::blueprints::resource::*;
use crate::blueprints::transaction_hash::*;

pub trait Invokable<I: Invocation, E> {
    fn invoke(&mut self, invocation: I) -> Result<I::Output, E>;
}

pub trait ClientStaticInvokeApi<E>:
    Invokable<ScryptoInvocation, E>
    //+ Invokable<EpochManagerCreateInvocation, E>
    + Invokable<EpochManagerNextRoundInvocation, E>
    + Invokable<EpochManagerGetCurrentEpochInvocation, E>
    + Invokable<EpochManagerSetEpochInvocation, E>
    + Invokable<EpochManagerUpdateValidatorInvocation, E>
    + Invokable<ValidatorRegisterInvocation, E>
    + Invokable<ValidatorUnregisterInvocation, E>
    + Invokable<ValidatorStakeInvocation, E>
    + Invokable<ValidatorUnstakeInvocation, E>
    + Invokable<ValidatorClaimXrdInvocation, E>
    + Invokable<ValidatorUpdateKeyInvocation, E>
    + Invokable<ValidatorUpdateAcceptDelegatedStakeInvocation, E>
    + Invokable<EpochManagerCreateValidatorInvocation, E>
    + Invokable<ClockCreateInvocation, E>
    + Invokable<ClockSetCurrentTimeInvocation, E>
    + Invokable<ClockGetCurrentTimeInvocation, E>
    + Invokable<ClockCompareCurrentTimeInvocation, E>
    + Invokable<MetadataSetInvocation, E>
    + Invokable<MetadataGetInvocation, E>
    + Invokable<AccessRulesAddAccessCheckInvocation, E>
    + Invokable<AccessRulesSetMethodAccessRuleInvocation, E>
    + Invokable<AccessRulesSetMethodMutabilityInvocation, E>
    + Invokable<AccessRulesSetGroupAccessRuleInvocation, E>
    + Invokable<AccessRulesSetGroupMutabilityInvocation, E>
    + Invokable<AccessRulesGetLengthInvocation, E>
    + Invokable<AuthZonePopInvocation, E>
    + Invokable<AuthZonePushInvocation, E>
    + Invokable<AuthZoneCreateProofInvocation, E>
    + Invokable<AuthZoneCreateProofByAmountInvocation, E>
    + Invokable<AuthZoneCreateProofByIdsInvocation, E>
    + Invokable<AuthZoneClearInvocation, E>
    + Invokable<AuthZoneDrainInvocation, E>
    + Invokable<AuthZoneAssertAccessRuleInvocation, E>
    + Invokable<AccessRulesAddAccessCheckInvocation, E>
    + Invokable<ComponentGlobalizeInvocation, E>
    + Invokable<ComponentGlobalizeWithOwnerInvocation, E>
    + Invokable<ComponentSetRoyaltyConfigInvocation, E>
    + Invokable<ComponentClaimRoyaltyInvocation, E>
    + Invokable<PackageSetRoyaltyConfigInvocation, E>
    + Invokable<PackageClaimRoyaltyInvocation, E>
    + Invokable<PackagePublishInvocation, E>
    + Invokable<BucketTakeInvocation, E>
    + Invokable<BucketPutInvocation, E>
    + Invokable<BucketTakeNonFungiblesInvocation, E>
    + Invokable<BucketGetNonFungibleLocalIdsInvocation, E>
    + Invokable<BucketGetAmountInvocation, E>
    + Invokable<BucketGetResourceAddressInvocation, E>
    + Invokable<BucketCreateProofInvocation, E>
    + Invokable<BucketCreateProofInvocation, E>
    + Invokable<ProofCloneInvocation, E>
    + Invokable<ProofGetAmountInvocation, E>
    + Invokable<ProofGetNonFungibleLocalIdsInvocation, E>
    + Invokable<ProofGetResourceAddressInvocation, E>
    + Invokable<ResourceManagerBurnBucketInvocation, E>
    + Invokable<ResourceManagerCreateNonFungibleInvocation, E>
    + Invokable<ResourceManagerCreateFungibleInvocation, E>
    + Invokable<ResourceManagerCreateNonFungibleWithInitialSupplyInvocation, E>
    + Invokable<ResourceManagerCreateUuidNonFungibleWithInitialSupplyInvocation, E>
    + Invokable<ResourceManagerCreateFungibleWithInitialSupplyInvocation, E>
    + Invokable<ResourceManagerBurnInvocation, E>
    + Invokable<ResourceManagerUpdateVaultAuthInvocation, E>
    + Invokable<ResourceManagerSetVaultAuthMutabilityInvocation, E>
    + Invokable<ResourceManagerCreateVaultInvocation, E>
    + Invokable<ResourceManagerCreateBucketInvocation, E>
    + Invokable<ResourceManagerMintNonFungibleInvocation, E>
    + Invokable<ResourceManagerMintUuidNonFungibleInvocation, E>
    + Invokable<ResourceManagerMintFungibleInvocation, E>
    + Invokable<ResourceManagerGetResourceTypeInvocation, E>
    + Invokable<ResourceManagerGetTotalSupplyInvocation, E>
    + Invokable<ResourceManagerUpdateNonFungibleDataInvocation, E>
    + Invokable<ResourceManagerNonFungibleExistsInvocation, E>
    + Invokable<ResourceManagerGetNonFungibleInvocation, E>
    + Invokable<VaultTakeInvocation, E>
    + Invokable<VaultPutInvocation, E>
    + Invokable<VaultLockFeeInvocation, E>
    + Invokable<VaultTakeNonFungiblesInvocation, E>
    + Invokable<VaultGetAmountInvocation, E>
    + Invokable<VaultGetResourceAddressInvocation, E>
    + Invokable<VaultGetNonFungibleLocalIdsInvocation, E>
    + Invokable<VaultCreateProofInvocation, E>
    + Invokable<VaultCreateProofByAmountInvocation, E>
    + Invokable<VaultCreateProofByIdsInvocation, E>
    + Invokable<VaultRecallInvocation, E>
    + Invokable<VaultRecallNonFungiblesInvocation, E>
    + Invokable<WorktopPutInvocation, E>
    + Invokable<WorktopTakeAmountInvocation, E>
    + Invokable<WorktopTakeAllInvocation, E>
    + Invokable<WorktopTakeNonFungiblesInvocation, E>
    + Invokable<WorktopAssertContainsInvocation, E>
    + Invokable<WorktopAssertContainsAmountInvocation, E>
    + Invokable<WorktopAssertContainsNonFungiblesInvocation, E>
    + Invokable<WorktopDrainInvocation, E>
    + Invokable<TransactionRuntimeGetHashInvocation, E>
    + Invokable<TransactionRuntimeGenerateUuidInvocation, E>
    + Invokable<LoggerLogInvocation, E>
    + Invokable<AccessControllerCreateGlobalInvocation, E>
    + Invokable<AccessControllerCreateProofInvocation, E>
    + Invokable<AccessControllerInitiateRecoveryAsPrimaryInvocation, E>
    + Invokable<AccessControllerInitiateRecoveryAsRecoveryInvocation, E>
    + Invokable<AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInvocation, E>
    + Invokable<AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInvocation, E>
    + Invokable<AccessControllerTimedConfirmRecoveryInvocation, E>
    + Invokable<AccessControllerCancelPrimaryRoleRecoveryProposalInvocation, E>
    + Invokable<AccessControllerCancelRecoveryRoleRecoveryProposalInvocation, E>
    + Invokable<AccessControllerLockPrimaryRoleInvocation, E>
    + Invokable<AccessControllerUnlockPrimaryRoleInvocation, E>
    + Invokable<AccessControllerStopTimedRecoveryInvocation, E>
    + Invokable<AccountCreateInvocation, E>
    + Invokable<AccountNewInvocation, E>
    + Invokable<AccountBalanceInvocation, E>
    + Invokable<AccountLockFeeInvocation, E>
    + Invokable<AccountLockContingentFeeInvocation, E>
    + Invokable<AccountDepositInvocation, E>
    + Invokable<AccountDepositBatchInvocation, E>
    + Invokable<AccountWithdrawInvocation, E>
    + Invokable<AccountWithdrawByAmountInvocation, E>
    + Invokable<AccountWithdrawByIdsInvocation, E>
    + Invokable<AccountLockFeeAndWithdrawInvocation, E>
    + Invokable<AccountLockFeeAndWithdrawByAmountInvocation, E>
    + Invokable<AccountLockFeeAndWithdrawByIdsInvocation, E>
    + Invokable<AccountCreateProofInvocation, E>
    + Invokable<AccountCreateProofByAmountInvocation, E>
    + Invokable<AccountCreateProofByIdsInvocation, E>
{
}
