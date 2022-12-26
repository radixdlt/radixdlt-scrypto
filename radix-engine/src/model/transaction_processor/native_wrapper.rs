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
use radix_engine_interface::wasm::*;

pub fn invoke_native_fn<Y, E>(native_invocation: NativeFnInvocation, api: &mut Y) -> Result<Box<dyn NativeOutput>, E>
    where
        Y: InvokableModel<E>,
{
    match native_invocation {
        NativeFnInvocation::Function(native_function) => match native_function {
            NativeFunctionInvocation::EpochManager(invocation) => match invocation {
                EpochManagerFunctionInvocation::Create(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeFunctionInvocation::Clock(invocation) => match invocation {
                ClockFunctionInvocation::Create(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeFunctionInvocation::ResourceManager(invocation) => match invocation {
                ResourceManagerFunctionInvocation::Create(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ResourceManagerFunctionInvocation::BurnBucket(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeFunctionInvocation::Package(invocation) => match invocation {
                PackageFunctionInvocation::Publish(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeFunctionInvocation::Component(invocation) => match invocation {
                ComponentFunctionInvocation::Globalize(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ComponentFunctionInvocation::GlobalizeWithOwner(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
        },
        NativeFnInvocation::Method(native_method) => match native_method {
            NativeMethodInvocation::Component(component_method) => match component_method {
                ComponentMethodInvocation::SetRoyaltyConfig(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ComponentMethodInvocation::ClaimRoyalty(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeMethodInvocation::Package(package_method) => match package_method {
                PackageMethodInvocation::SetRoyaltyConfig(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                PackageMethodInvocation::ClaimRoyalty(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeMethodInvocation::Bucket(bucket_method) => match bucket_method {
                BucketMethodInvocation::Take(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                BucketMethodInvocation::CreateProof(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                BucketMethodInvocation::TakeNonFungibles(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                BucketMethodInvocation::GetNonFungibleIds(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                BucketMethodInvocation::GetAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                BucketMethodInvocation::Put(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                BucketMethodInvocation::GetResourceAddress(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeMethodInvocation::AuthZoneStack(auth_zone_method) => match auth_zone_method {
                AuthZoneStackMethodInvocation::Pop(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                AuthZoneStackMethodInvocation::Push(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                AuthZoneStackMethodInvocation::CreateProof(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                AuthZoneStackMethodInvocation::CreateProofByAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                AuthZoneStackMethodInvocation::CreateProofByIds(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                AuthZoneStackMethodInvocation::Clear(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                AuthZoneStackMethodInvocation::Drain(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                AuthZoneStackMethodInvocation::AssertAuthRule(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeMethodInvocation::Proof(proof_method) => match proof_method {
                ProofMethodInvocation::GetAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ProofMethodInvocation::GetNonFungibleIds(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ProofMethodInvocation::GetResourceAddress(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ProofMethodInvocation::Clone(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeMethodInvocation::Vault(vault_method) => match vault_method {
                VaultMethodInvocation::Take(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                VaultMethodInvocation::Put(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                VaultMethodInvocation::LockFee(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                VaultMethodInvocation::TakeNonFungibles(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                VaultMethodInvocation::GetAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                VaultMethodInvocation::GetResourceAddress(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                VaultMethodInvocation::GetNonFungibleIds(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                VaultMethodInvocation::CreateProof(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                VaultMethodInvocation::CreateProofByAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                VaultMethodInvocation::CreateProofByIds(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                VaultMethodInvocation::Recall(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                VaultMethodInvocation::RecallNonFungibles(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeMethodInvocation::AccessRulesChain(access_rules_method) => {
                match access_rules_method {
                    AccessRulesChainMethodInvocation::AddAccessCheck(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    },
                    AccessRulesChainMethodInvocation::SetMethodAccessRule(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    },
                    AccessRulesChainMethodInvocation::SetMethodMutability(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    },
                    AccessRulesChainMethodInvocation::SetGroupAccessRule(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    },
                    AccessRulesChainMethodInvocation::SetGroupMutability(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    },
                    AccessRulesChainMethodInvocation::GetLength(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    },
                }
            }
            NativeMethodInvocation::Metadata(metadata_method) => match metadata_method {
                MetadataMethodInvocation::Set(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                MetadataMethodInvocation::Get(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeMethodInvocation::ResourceManager(resman_method) => match resman_method {
                ResourceManagerMethodInvocation::Burn(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ResourceManagerMethodInvocation::UpdateVaultAuth(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ResourceManagerMethodInvocation::LockVaultAuth(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ResourceManagerMethodInvocation::CreateVault(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ResourceManagerMethodInvocation::CreateBucket(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ResourceManagerMethodInvocation::Mint(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ResourceManagerMethodInvocation::GetResourceType(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ResourceManagerMethodInvocation::GetTotalSupply(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ResourceManagerMethodInvocation::UpdateNonFungibleData(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ResourceManagerMethodInvocation::NonFungibleExists(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ResourceManagerMethodInvocation::GetNonFungible(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeMethodInvocation::EpochManager(epoch_manager_method) => {
                match epoch_manager_method {
                    EpochManagerMethodInvocation::GetCurrentEpoch(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    },
                    EpochManagerMethodInvocation::SetEpoch(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    },
                }
            }
            NativeMethodInvocation::Clock(clock_method) => match clock_method {
                ClockMethodInvocation::SetCurrentTime(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ClockMethodInvocation::GetCurrentTime(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                ClockMethodInvocation::CompareCurrentTime(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeMethodInvocation::Worktop(worktop_method) => match worktop_method {
                WorktopMethodInvocation::TakeNonFungibles(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                WorktopMethodInvocation::Put(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                WorktopMethodInvocation::Drain(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                WorktopMethodInvocation::AssertContainsNonFungibles(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                WorktopMethodInvocation::AssertContains(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                WorktopMethodInvocation::AssertContainsAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                WorktopMethodInvocation::TakeAll(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                WorktopMethodInvocation::TakeAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
            NativeMethodInvocation::TransactionRuntime(method) => match method {
                TransactionRuntimeMethodInvocation::Get(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
                TransactionRuntimeMethodInvocation::GenerateUuid(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                },
            },
        },
    }
}

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
                    let invocation: TransactionRuntimeGetHashInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                TransactionHashMethod::GenerateUuid => {
                    let invocation: TransactionRuntimeGenerateUuidInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
        },
    }
}
