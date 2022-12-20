use crate::engine::KernelError;
use crate::engine::*;
use crate::model::NativeOutput;
use crate::types::*;
use native_sdk::resource::{ComponentAuthZone, Worktop};
use radix_engine_interface::api::api::{EngineApi, Invokable, InvokableModel};
use radix_engine_interface::api::types::{
    AccessRulesChainMethod, AuthZoneStackMethod, BucketMethod, EpochManagerFunction,
    EpochManagerMethod, NativeFn, NativeFunction, NativeMethod, PackageFunction, ProofMethod,
    ResourceManagerFunction, ResourceManagerMethod, TransactionProcessorFunction, VaultMethod,
    WorktopMethod,
};
use radix_engine_interface::model::*;

// TODO: Cleanup
pub fn parse_and_invoke_native_fn<'a, Y>(
    native_fn: NativeFn,
    args: Vec<u8>,
    api: &mut Y,
) -> Result<Box<dyn NativeOutput>, RuntimeError>
where
    Y: SystemApi
        + Invokable<ScryptoInvocation, RuntimeError>
        + EngineApi<RuntimeError>
        + InvokableModel<RuntimeError>,
{
    match native_fn {
        NativeFn::Function(native_function) => match native_function {
            NativeFunction::Component(component_function) => match component_function {
                ComponentFunction::Globalize => {
                    let invocation: ComponentGlobalizeInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ComponentFunction::GlobalizeWithOwner => {
                    let invocation: ComponentGlobalizeWithOwnerInvocation = scrypto_decode(&args)
                        .map_err(|e| {
                        RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                    })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeFunction::EpochManager(EpochManagerFunction::Create) => {
                let invocation: EpochManagerCreateInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            NativeFunction::ResourceManager(resman_function) => match resman_function {
                ResourceManagerFunction::BurnBucket => {
                    let invocation: ResourceManagerBucketBurnInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerFunction::Create => {
                    let invocation: ResourceManagerCreateInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    if let (_, Some(bucket)) = &rtn {
                        Worktop::sys_put(Bucket(bucket.0), api)?;
                    }
                    Ok(Box::new(rtn))
                }
            },
            NativeFunction::Clock(ClockFunction::Create) => {
                let invocation: ClockCreateInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            NativeFunction::TransactionProcessor(TransactionProcessorFunction::Run) => {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidInvocation,
                ));
            }
            NativeFunction::Package(package_function) => match package_function {
                PackageFunction::Publish => {
                    let invocation: PackagePublishInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
        },
        NativeFn::Method(native_method) => match native_method {
            NativeMethod::Bucket(bucket_method) => match bucket_method {
                BucketMethod::Take => {
                    let invocation: BucketTakeInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Worktop::sys_put(Bucket(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
                BucketMethod::CreateProof => {
                    let invocation: BucketCreateProofInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    ComponentAuthZone::sys_push(Proof(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
                BucketMethod::TakeNonFungibles => {
                    let invocation: BucketTakeNonFungiblesInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Worktop::sys_put(Bucket(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
                BucketMethod::GetNonFungibleIds => {
                    let invocation: BucketGetNonFungibleIdsInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                BucketMethod::GetAmount => {
                    let invocation: BucketGetAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                BucketMethod::Put => {
                    let invocation: BucketPutInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                BucketMethod::GetResourceAddress => {
                    let invocation: BucketGetResourceAddressInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            // TODO: Integrate with static ids
            NativeMethod::AuthZoneStack(auth_zone_method) => match auth_zone_method {
                AuthZoneStackMethod::Pop => {
                    let invocation: AuthZonePopInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethod::Push => {
                    let invocation: AuthZonePushInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethod::CreateProof => {
                    let invocation: AuthZoneCreateProofInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethod::CreateProofByAmount => {
                    let invocation: AuthZoneCreateProofByAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| {
                        RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                    })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethod::CreateProofByIds => {
                    let invocation: AuthZoneCreateProofByIdsInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethod::Clear => {
                    let invocation: AuthZoneClearInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethod::Drain => {
                    let invocation: AuthZoneDrainInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethod::AssertAccessRule => {
                    let invocation: AuthZoneAssertAccessRuleInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethod::Proof(proof_method) => match proof_method {
                ProofMethod::GetAmount => {
                    let invocation: ProofGetAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ProofMethod::GetNonFungibleIds => {
                    let invocation: ProofGetNonFungibleIdsInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ProofMethod::GetResourceAddress => {
                    let invocation: ProofGetResourceAddressInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ProofMethod::Clone => {
                    let invocation: ProofCloneInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethod::Vault(vault_method) => match vault_method {
                VaultMethod::Take => {
                    let invocation: VaultTakeInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Worktop::sys_put(Bucket(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
                VaultMethod::Put => {
                    let invocation: VaultPutInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethod::LockFee => {
                    let invocation: VaultLockFeeInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethod::TakeNonFungibles => {
                    let invocation: VaultTakeNonFungiblesInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Worktop::sys_put(Bucket(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
                VaultMethod::GetAmount => {
                    let invocation: VaultGetAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethod::GetResourceAddress => {
                    let invocation: VaultGetResourceAddressInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethod::GetNonFungibleIds => {
                    let invocation: VaultGetNonFungibleIdsInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethod::CreateProof => {
                    let invocation: VaultCreateProofInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    ComponentAuthZone::sys_push(Proof(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
                VaultMethod::CreateProofByAmount => {
                    let invocation: VaultCreateProofByAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    ComponentAuthZone::sys_push(Proof(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
                VaultMethod::CreateProofByIds => {
                    let invocation: VaultCreateProofByIdsInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    ComponentAuthZone::sys_push(Proof(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
                VaultMethod::Recall => {
                    let invocation: VaultRecallInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Worktop::sys_put(Bucket(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
                VaultMethod::RecallNonFungibles => {
                    let invocation: VaultRecallNonFungiblesInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Worktop::sys_put(Bucket(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethod::AccessRulesChain(component_method) => match component_method {
                AccessRulesChainMethod::AddAccessCheck => {
                    let invocation: AccessRulesAddAccessCheckInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainMethod::SetMethodAccessRule => {
                    let invocation: AccessRulesSetMethodAccessRuleInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainMethod::SetMethodMutability => {
                    let invocation: AccessRulesSetMethodMutabilityInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainMethod::SetGroupAccessRule => {
                    let invocation: AccessRulesSetGroupAccessRuleInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainMethod::SetGroupMutability => {
                    let invocation: AccessRulesSetGroupMutabilityInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainMethod::GetLength => {
                    let invocation: AccessRulesGetLengthInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethod::Metadata(metadata_method) => match metadata_method {
                MetadataMethod::Set => {
                    let invocation: MetadataSetInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                MetadataMethod::Get => {
                    let invocation: MetadataGetInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethod::ResourceManager(resman_method) => match resman_method {
                ResourceManagerMethod::Burn => {
                    let invocation: ResourceManagerBurnInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethod::UpdateVaultAuth => {
                    let invocation: ResourceManagerUpdateVaultAuthInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethod::LockAuth => {
                    let invocation: ResourceManagerSetVaultAuthMutabilityInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethod::CreateVault => {
                    let invocation: ResourceManagerCreateVaultInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethod::CreateBucket => {
                    let invocation: ResourceManagerCreateBucketInvocation = scrypto_decode(&args)
                        .map_err(|e| {
                        RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                    })?;
                    let rtn = api.invoke(invocation)?;
                    Worktop::sys_put(Bucket(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethod::Mint => {
                    let invocation: ResourceManagerMintInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Worktop::sys_put(Bucket(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethod::GetResourceType => {
                    let invocation: ResourceManagerGetResourceTypeInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethod::GetTotalSupply => {
                    let invocation: ResourceManagerGetTotalSupplyInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethod::UpdateNonFungibleData => {
                    let invocation: ResourceManagerUpdateNonFungibleDataInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethod::NonFungibleExists => {
                    let invocation: ResourceManagerNonFungibleExistsInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethod::GetNonFungible => {
                    let invocation: ResourceManagerGetNonFungibleInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethod::EpochManager(epoch_manager_method) => match epoch_manager_method {
                EpochManagerMethod::GetCurrentEpoch => {
                    let invocation: EpochManagerGetCurrentEpochInvocation = scrypto_decode(&args)
                        .map_err(|e| {
                        RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                    })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                EpochManagerMethod::SetEpoch => {
                    let invocation: EpochManagerSetEpochInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethod::Clock(clock_method) => match clock_method {
                ClockMethod::SetCurrentTime => {
                    let invocation: ClockSetCurrentTimeInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ClockMethod::GetCurrentTime => {
                    let invocation: ClockGetCurrentTimeInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ClockMethod::CompareCurrentTime => {
                    let invocation: ClockCompareCurrentTimeInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            // TODO: Integrate with static ids
            NativeMethod::Worktop(worktop_method) => match worktop_method {
                WorktopMethod::TakeNonFungibles => {
                    let invocation: WorktopTakeNonFungiblesInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethod::Put => {
                    let invocation: WorktopPutInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethod::Drain => {
                    let invocation: WorktopDrainInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethod::AssertContainsNonFungibles => {
                    let invocation: WorktopAssertContainsNonFungiblesInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethod::AssertContains => {
                    let invocation: WorktopAssertContainsInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethod::AssertContainsAmount => {
                    let invocation: WorktopAssertContainsAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| {
                        RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                    })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethod::TakeAll => {
                    let invocation: WorktopTakeAllInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethod::TakeAmount => {
                    let invocation: WorktopTakeAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethod::Component(component_method) => match component_method {
                ComponentMethod::SetRoyaltyConfig => {
                    let invocation: ComponentSetRoyaltyConfigInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ComponentMethod::ClaimRoyalty => {
                    let invocation: ComponentClaimRoyaltyInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Worktop::sys_put(Bucket(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethod::Package(package_method) => match package_method {
                PackageMethod::SetRoyaltyConfig => {
                    let invocation: PackageSetRoyaltyConfigInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                PackageMethod::ClaimRoyalty => {
                    let invocation: PackageClaimRoyaltyInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Worktop::sys_put(Bucket(rtn.0), api)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethod::TransactionHash(method) => match method {
                TransactionHashMethod::Get => {
                    let invocation: TransactionHashGetInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                TransactionHashMethod::GenerateUuid => {
                    let invocation: TransactionHashGenerateUuidInvocation = scrypto_decode(&args)
                        .map_err(|e| {
                        RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                    })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
        },
    }
}
