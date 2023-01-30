use crate::errors::{InterpreterError, RuntimeError};
use crate::{blueprints::transaction_processor::NativeOutput, types::*};
use radix_engine_interface::api::component::*;
use radix_engine_interface::api::node_modules::{auth::*, metadata::*};
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::ClientStaticInvokeApi;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::kv_store::*;
use radix_engine_interface::blueprints::resource::WorktopAssertContainsInvocation;
use radix_engine_interface::blueprints::{
    clock::*, epoch_manager::*, identity::*, logger::*, resource::*, transaction_hash::*,
};

pub fn resolve_and_invoke_native_fn<Y>(
    native_fn: NativeFn,
    invocation: Vec<u8>,
    api: &mut Y,
) -> Result<Box<dyn NativeOutput>, RuntimeError>
where
    Y: ClientStaticInvokeApi<RuntimeError>,
{
    match native_fn {
        NativeFn::Component(component_fn) => match component_fn {
            ComponentFn::Globalize => {
                let invocation = scrypto_decode::<ComponentGlobalizeInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ComponentFn::GlobalizeWithOwner => {
                let invocation =
                    scrypto_decode::<ComponentGlobalizeWithOwnerInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ComponentFn::SetRoyaltyConfig => {
                let invocation = scrypto_decode::<ComponentSetRoyaltyConfigInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ComponentFn::ClaimRoyalty => {
                let invocation = scrypto_decode::<ComponentClaimRoyaltyInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::Package(package_fn) => match package_fn {
            PackageFn::Publish => {
                let invocation = scrypto_decode::<PackagePublishInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            PackageFn::SetRoyaltyConfig => {
                let invocation = scrypto_decode::<PackageSetRoyaltyConfigInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            PackageFn::ClaimRoyalty => {
                let invocation = scrypto_decode::<PackageClaimRoyaltyInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::Bucket(bucket_fn) => match bucket_fn {
            BucketFn::Take => {
                let invocation = scrypto_decode::<BucketTakeInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            BucketFn::CreateProof => {
                let invocation = scrypto_decode::<BucketCreateProofInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            BucketFn::TakeNonFungibles => {
                let invocation = scrypto_decode::<BucketTakeNonFungiblesInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            BucketFn::GetNonFungibleLocalIds => {
                let invocation =
                    scrypto_decode::<BucketGetNonFungibleLocalIdsInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            BucketFn::GetAmount => {
                let invocation = scrypto_decode::<BucketGetAmountInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            BucketFn::Put => {
                let invocation = scrypto_decode::<BucketPutInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            BucketFn::GetResourceAddress => {
                let invocation = scrypto_decode::<BucketGetResourceAddressInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::AuthZoneStack(auth_zone_fn) => match auth_zone_fn {
            AuthZoneStackFn::Pop => {
                let invocation = scrypto_decode::<AuthZonePopInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackFn::Push => {
                let invocation = scrypto_decode::<AuthZonePushInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackFn::CreateProof => {
                let invocation = scrypto_decode::<AuthZoneCreateProofInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackFn::CreateProofByAmount => {
                let invocation =
                    scrypto_decode::<AuthZoneCreateProofByAmountInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackFn::CreateProofByIds => {
                let invocation = scrypto_decode::<AuthZoneCreateProofByIdsInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackFn::Clear => {
                let invocation = scrypto_decode::<AuthZoneClearInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackFn::Drain => {
                let invocation = scrypto_decode::<AuthZoneDrainInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackFn::AssertAccessRule => {
                let invocation = scrypto_decode::<AuthZoneAssertAccessRuleInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::Proof(proof_fn) => match proof_fn {
            ProofFn::GetAmount => {
                let invocation = scrypto_decode::<ProofGetAmountInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ProofFn::GetNonFungibleLocalIds => {
                let invocation =
                    scrypto_decode::<ProofGetNonFungibleLocalIdsInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ProofFn::GetResourceAddress => {
                let invocation = scrypto_decode::<ProofGetResourceAddressInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ProofFn::Clone => {
                let invocation = scrypto_decode::<ProofCloneInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::Vault(vault_fn) => match vault_fn {
            VaultFn::Take => {
                let invocation = scrypto_decode::<VaultTakeInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultFn::Put => {
                let invocation = scrypto_decode::<VaultPutInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultFn::LockFee => {
                let invocation = scrypto_decode::<VaultLockFeeInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultFn::TakeNonFungibles => {
                let invocation = scrypto_decode::<VaultTakeNonFungiblesInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultFn::GetAmount => {
                let invocation = scrypto_decode::<VaultGetAmountInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultFn::GetResourceAddress => {
                let invocation = scrypto_decode::<VaultGetResourceAddressInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultFn::GetNonFungibleLocalIds => {
                let invocation =
                    scrypto_decode::<VaultGetNonFungibleLocalIdsInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultFn::CreateProof => {
                let invocation = scrypto_decode::<VaultCreateProofInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultFn::CreateProofByAmount => {
                let invocation = scrypto_decode::<VaultCreateProofByAmountInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultFn::CreateProofByIds => {
                let invocation = scrypto_decode::<VaultCreateProofByIdsInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultFn::Recall => {
                let invocation = scrypto_decode::<VaultRecallInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultFn::RecallNonFungibles => {
                let invocation = scrypto_decode::<VaultRecallNonFungiblesInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::AccessRulesChain(access_rules_fn) => match access_rules_fn {
            AccessRulesChainFn::AddAccessCheck => {
                let invocation = scrypto_decode::<AccessRulesAddAccessCheckInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessRulesChainFn::SetMethodAccessRule => {
                let invocation =
                    scrypto_decode::<AccessRulesSetMethodAccessRuleInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessRulesChainFn::SetMethodMutability => {
                let invocation =
                    scrypto_decode::<AccessRulesSetMethodMutabilityInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessRulesChainFn::SetGroupAccessRule => {
                let invocation =
                    scrypto_decode::<AccessRulesSetGroupAccessRuleInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessRulesChainFn::SetGroupMutability => {
                let invocation =
                    scrypto_decode::<AccessRulesSetGroupMutabilityInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessRulesChainFn::GetLength => {
                let invocation = scrypto_decode::<AccessRulesGetLengthInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::Metadata(metadata_fn) => match metadata_fn {
            MetadataFn::Set => {
                let invocation = scrypto_decode::<MetadataSetInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            MetadataFn::Get => {
                let invocation = scrypto_decode::<MetadataGetInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::ResourceManager(resource_manager_fn) => match resource_manager_fn {
            ResourceManagerFn::CreateNonFungible => {
                let invocation =
                    scrypto_decode::<ResourceManagerCreateNonFungibleInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::CreateFungible => {
                let invocation =
                    scrypto_decode::<ResourceManagerCreateFungibleInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::CreateNonFungibleWithInitialSupply => {
                let invocation = scrypto_decode::<
                    ResourceManagerCreateNonFungibleWithInitialSupplyInvocation,
                >(&invocation)
                .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::CreateUuidNonFungibleWithInitialSupply => {
                let invocation = scrypto_decode::<
                    ResourceManagerCreateUuidNonFungibleWithInitialSupplyInvocation,
                >(&invocation)
                .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::CreateFungibleWithInitialSupply => {
                let invocation = scrypto_decode::<
                    ResourceManagerCreateFungibleWithInitialSupplyInvocation,
                >(&invocation)
                .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::BurnBucket => {
                let invocation = scrypto_decode::<ResourceManagerBurnBucketInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::Burn => {
                let invocation = scrypto_decode::<ResourceManagerBurnInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::UpdateVaultAuth => {
                let invocation =
                    scrypto_decode::<ResourceManagerUpdateVaultAuthInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::SetVaultAuthMutability => {
                let invocation =
                    scrypto_decode::<ResourceManagerSetVaultAuthMutabilityInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::CreateVault => {
                let invocation =
                    scrypto_decode::<ResourceManagerCreateVaultInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::CreateBucket => {
                let invocation =
                    scrypto_decode::<ResourceManagerCreateBucketInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::MintNonFungible => {
                let invocation =
                    scrypto_decode::<ResourceManagerMintNonFungibleInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::MintUuidNonFungible => {
                let invocation =
                    scrypto_decode::<ResourceManagerMintUuidNonFungibleInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::MintFungible => {
                let invocation =
                    scrypto_decode::<ResourceManagerMintFungibleInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::GetResourceType => {
                let invocation =
                    scrypto_decode::<ResourceManagerGetResourceTypeInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::GetTotalSupply => {
                let invocation =
                    scrypto_decode::<ResourceManagerGetTotalSupplyInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::UpdateNonFungibleData => {
                let invocation =
                    scrypto_decode::<ResourceManagerUpdateNonFungibleDataInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::NonFungibleExists => {
                let invocation =
                    scrypto_decode::<ResourceManagerNonFungibleExistsInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ResourceManagerFn::GetNonFungible => {
                let invocation =
                    scrypto_decode::<ResourceManagerGetNonFungibleInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::EpochManager(epoch_manager_fn) => match epoch_manager_fn {
            EpochManagerFn::Create => {
                let invocation = scrypto_decode::<EpochManagerCreateInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            EpochManagerFn::GetCurrentEpoch => {
                let invocation =
                    scrypto_decode::<EpochManagerGetCurrentEpochInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            EpochManagerFn::NextRound => {
                let invocation = scrypto_decode::<EpochManagerNextRoundInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            EpochManagerFn::SetEpoch => {
                let invocation = scrypto_decode::<EpochManagerSetEpochInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            EpochManagerFn::CreateValidator => {
                let invocation =
                    scrypto_decode::<EpochManagerCreateValidatorInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            EpochManagerFn::UpdateValidator => {
                let invocation =
                    scrypto_decode::<EpochManagerUpdateValidatorInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::Validator(validator_fn) => match validator_fn {
            ValidatorFn::Register => {
                let invocation = scrypto_decode::<ValidatorRegisterInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ValidatorFn::Unregister => {
                let invocation = scrypto_decode::<ValidatorUnregisterInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ValidatorFn::Stake => {
                let invocation = scrypto_decode::<ValidatorStakeInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ValidatorFn::Unstake => {
                let invocation = scrypto_decode::<ValidatorUnstakeInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ValidatorFn::ClaimXrd => {
                let invocation = scrypto_decode::<ValidatorClaimXrdInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::Clock(clock_fn) => match clock_fn {
            ClockFn::Create => {
                let invocation = scrypto_decode::<ClockCreateInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ClockFn::SetCurrentTime => {
                let invocation = scrypto_decode::<ClockSetCurrentTimeInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ClockFn::GetCurrentTime => {
                let invocation = scrypto_decode::<ClockGetCurrentTimeInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ClockFn::CompareCurrentTime => {
                let invocation = scrypto_decode::<ClockCompareCurrentTimeInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::Identity(identity_fn) => match identity_fn {
            IdentityFn::Create => {
                let invocation = scrypto_decode::<IdentityCreateInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::Logger(logger_fn) => match logger_fn {
            LoggerFn::Log => {
                let invocation = scrypto_decode::<LoggerLogInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::Worktop(worktop_fn) => match worktop_fn {
            WorktopFn::TakeNonFungibles => {
                let invocation = scrypto_decode::<WorktopTakeNonFungiblesInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopFn::Put => {
                let invocation = scrypto_decode::<WorktopPutInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopFn::Drain => {
                let invocation = scrypto_decode::<WorktopDrainInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopFn::AssertContainsNonFungibles => {
                let invocation =
                    scrypto_decode::<WorktopAssertContainsNonFungiblesInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopFn::AssertContains => {
                let invocation = scrypto_decode::<WorktopAssertContainsInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopFn::AssertContainsAmount => {
                let invocation =
                    scrypto_decode::<WorktopAssertContainsAmountInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopFn::TakeAll => {
                let invocation = scrypto_decode::<WorktopTakeAllInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopFn::TakeAmount => {
                let invocation = scrypto_decode::<WorktopTakeAmountInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::TransactionRuntime(tx_runtime_fn) => match tx_runtime_fn {
            TransactionRuntimeFn::GetHash => {
                let invocation = scrypto_decode::<TransactionRuntimeGetHashInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            TransactionRuntimeFn::GenerateUuid => {
                let invocation =
                    scrypto_decode::<TransactionRuntimeGenerateUuidInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::AccessController(ac_fn) => match ac_fn {
            AccessControllerFn::CreateGlobal => {
                let invocation =
                    scrypto_decode::<AccessControllerCreateGlobalInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerFn::CreateProof => {
                let invocation =
                    scrypto_decode::<AccessControllerCreateProofInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerFn::InitiateRecoveryAsPrimary => {
                let invocation = scrypto_decode::<
                    AccessControllerInitiateRecoveryAsPrimaryInvocation,
                >(&invocation)
                .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerFn::InitiateRecoveryAsRecovery => {
                let invocation = scrypto_decode::<
                    AccessControllerInitiateRecoveryAsRecoveryInvocation,
                >(&invocation)
                .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerFn::QuickConfirmPrimaryRoleRecoveryProposal => {
                let invocation = scrypto_decode::<
                    AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInvocation,
                >(&invocation)
                .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerFn::QuickConfirmRecoveryRoleRecoveryProposal => {
                let invocation = scrypto_decode::<
                    AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInvocation,
                >(&invocation)
                .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerFn::TimedConfirmRecovery => {
                let invocation =
                    scrypto_decode::<AccessControllerTimedConfirmRecoveryInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerFn::CancelPrimaryRoleRecoveryProposal => {
                let invocation = scrypto_decode::<
                    AccessControllerCancelPrimaryRoleRecoveryProposalInvocation,
                >(&invocation)
                .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerFn::CancelRecoveryRoleRecoveryProposal => {
                let invocation = scrypto_decode::<
                    AccessControllerCancelRecoveryRoleRecoveryProposalInvocation,
                >(&invocation)
                .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerFn::LockPrimaryRole => {
                let invocation =
                    scrypto_decode::<AccessControllerLockPrimaryRoleInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerFn::UnlockPrimaryRole => {
                let invocation =
                    scrypto_decode::<AccessControllerUnlockPrimaryRoleInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerFn::StopTimedRecovery => {
                let invocation =
                    scrypto_decode::<AccessControllerStopTimedRecoveryInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::KeyValueStore(kv_store_fn) => match kv_store_fn {
            KeyValueStoreFn::Create => {
                let invocation = scrypto_decode::<KeyValueStoreCreateInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            KeyValueStoreFn::Get => {
                let invocation = scrypto_decode::<KeyValueStoreGetInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            KeyValueStoreFn::GetMut => {
                let invocation = scrypto_decode::<KeyValueStoreGetMutInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            KeyValueStoreFn::Insert => {
                let invocation = scrypto_decode::<KeyValueStoreInsertInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeFn::TransactionProcessor(_) => Err(RuntimeError::InterpreterError(
            InterpreterError::DisallowedInvocation(native_fn),
        )),
    }
}
