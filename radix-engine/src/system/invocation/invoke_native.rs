use crate::errors::{InterpreterError, RuntimeError};
use crate::{blueprints::transaction_processor::NativeOutput, types::*};
use radix_engine_interface::api::blueprints::{
    clock::*, epoch_manager::*, identity::*, logger::*, resource::*, transaction_hash::*,
};
use radix_engine_interface::api::component::*;
use radix_engine_interface::api::node_modules::{auth::*, metadata::*};
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::{
    AccessRulesChainInvocation, AuthZoneStackInvocation, BucketInvocation, ClockInvocation,
    ComponentInvocation, EpochManagerInvocation, IdentityInvocation, LoggerInvocation,
    MetadataInvocation, NativeInvocation, PackageInvocation, ProofInvocation, ResourceInvocation,
    TransactionRuntimeInvocation, ValidatorInvocation, VaultInvocation, WorktopInvocation,
};
use radix_engine_interface::api::ClientStaticInvokeApi;

pub fn invoke_native_fn<Y, E>(
    native_invocation: NativeInvocation,
    api: &mut Y,
) -> Result<Box<dyn NativeOutput>, E>
where
    Y: ClientStaticInvokeApi<E>,
{
    match native_invocation {
        NativeInvocation::Component(component_invocation) => match component_invocation {
            ComponentInvocation::Globalize(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ComponentInvocation::GlobalizeWithOwner(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ComponentInvocation::SetRoyaltyConfig(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ComponentInvocation::ClaimRoyalty(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::Package(package_invocation) => match package_invocation {
            PackageInvocation::Publish(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            PackageInvocation::SetRoyaltyConfig(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            PackageInvocation::ClaimRoyalty(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::Bucket(bucket_invocation) => match bucket_invocation {
            BucketInvocation::Take(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            BucketInvocation::CreateProof(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            BucketInvocation::TakeNonFungibles(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            BucketInvocation::GetNonFungibleLocalIds(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            BucketInvocation::GetAmount(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            BucketInvocation::Put(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            BucketInvocation::GetResourceAddress(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::AuthZoneStack(auth_zone_invocation) => match auth_zone_invocation {
            AuthZoneStackInvocation::Pop(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackInvocation::Push(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackInvocation::CreateProof(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackInvocation::CreateProofByAmount(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackInvocation::CreateProofByIds(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackInvocation::Clear(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackInvocation::Drain(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AuthZoneStackInvocation::AssertAuthRule(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::Proof(proof_invocation) => match proof_invocation {
            ProofInvocation::GetAmount(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ProofInvocation::GetNonFungibleLocalIds(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ProofInvocation::GetResourceAddress(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ProofInvocation::Clone(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::Vault(vault_invocation) => match vault_invocation {
            VaultInvocation::Take(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultInvocation::Put(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultInvocation::LockFee(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultInvocation::TakeNonFungibles(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultInvocation::GetAmount(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultInvocation::GetResourceAddress(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultInvocation::GetNonFungibleLocalIds(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultInvocation::CreateProof(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultInvocation::CreateProofByAmount(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultInvocation::CreateProofByIds(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultInvocation::Recall(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            VaultInvocation::RecallNonFungibles(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::AccessRulesChain(access_rules_invocation) => {
            match access_rules_invocation {
                AccessRulesChainInvocation::AddAccessCheck(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainInvocation::SetMethodAccessRule(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainInvocation::SetMethodMutability(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainInvocation::SetGroupAccessRule(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainInvocation::SetGroupMutability(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainInvocation::GetLength(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            }
        }
        NativeInvocation::Metadata(metadata_invocation) => match metadata_invocation {
            MetadataInvocation::Set(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            MetadataInvocation::Get(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::ResourceManager(resource_manager_invocation) => {
            match resource_manager_invocation {
                ResourceInvocation::CreateNonFungible(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::CreateFungible(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::CreateNonFungibleWithInitialSupply(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::CreateUuidNonFungibleWithInitialSupply(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::CreateFungibleWithInitialSupply(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::BurnBucket(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::Burn(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::UpdateVaultAuth(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::SetVaultAuthMutability(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::CreateVault(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::CreateBucket(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::MintNonFungible(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::MintUuidNonFungible(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::MintFungible(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::GetResourceType(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::GetTotalSupply(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::UpdateNonFungibleData(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::NonFungibleExists(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceInvocation::GetNonFungible(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            }
        }
        NativeInvocation::EpochManager(epoch_manager_invocation) => {
            match epoch_manager_invocation {
                EpochManagerInvocation::Create(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                EpochManagerInvocation::GetCurrentEpoch(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                EpochManagerInvocation::NextRound(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                EpochManagerInvocation::SetEpoch(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                EpochManagerInvocation::CreateValidator(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                EpochManagerInvocation::UpdateValidator(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            }
        }
        NativeInvocation::Validator(validator_invocation) => match validator_invocation {
            ValidatorInvocation::Register(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ValidatorInvocation::Unregister(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ValidatorInvocation::Stake(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ValidatorInvocation::Unstake(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::Clock(clock_invocation) => match clock_invocation {
            ClockInvocation::Create(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ClockInvocation::SetCurrentTime(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ClockInvocation::GetCurrentTime(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ClockInvocation::CompareCurrentTime(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::Identity(identity_invocation) => match identity_invocation {
            IdentityInvocation::Create(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::Logger(logger_invocation) => match logger_invocation {
            LoggerInvocation::Log(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::Worktop(worktop_invocation) => match worktop_invocation {
            WorktopInvocation::TakeNonFungibles(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopInvocation::Put(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopInvocation::Drain(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopInvocation::AssertContainsNonFungibles(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopInvocation::AssertContains(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopInvocation::AssertContainsAmount(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopInvocation::TakeAll(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            WorktopInvocation::TakeAmount(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::TransactionRuntime(method) => match method {
            TransactionRuntimeInvocation::GetHash(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            TransactionRuntimeInvocation::GenerateUuid(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
    }
}

pub fn invoke_native_fn_by_identifier<Y>(
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
            AuthZoneStackFn::Pop | AuthZoneStackFn::Push => Err(RuntimeError::InterpreterError(
                InterpreterError::DisallowedInvocation,
            )),
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
        NativeFn::TransactionProcessor(_) => Err(RuntimeError::InterpreterError(
            InterpreterError::DisallowedInvocation,
        )),
    }
}
