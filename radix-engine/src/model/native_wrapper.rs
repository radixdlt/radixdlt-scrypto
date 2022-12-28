use crate::model::NativeOutput;
use crate::types::*;
use radix_engine_interface::api::api::InvokableModel;
use radix_engine_interface::wasm::*;

pub fn invoke_native_fn<Y, E>(
    native_invocation: NativeFnInvocation,
    api: &mut Y,
) -> Result<Box<dyn NativeOutput>, E>
where
    Y: InvokableModel<E>,
{
    match native_invocation {
        NativeFnInvocation::Function(native_function) => match native_function {
            NativeFunctionInvocation::EpochManager(invocation) => match invocation {
                EpochManagerFunctionInvocation::Create(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeFunctionInvocation::Clock(invocation) => match invocation {
                ClockFunctionInvocation::Create(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeFunctionInvocation::ResourceManager(invocation) => match invocation {
                ResourceManagerFunctionInvocation::Create(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerFunctionInvocation::BurnBucket(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeFunctionInvocation::Package(invocation) => match invocation {
                PackageFunctionInvocation::Publish(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeFunctionInvocation::Component(invocation) => match invocation {
                ComponentFunctionInvocation::Globalize(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ComponentFunctionInvocation::GlobalizeWithOwner(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
        },
        NativeFnInvocation::Method(native_method) => match native_method {
            NativeMethodInvocation::Component(component_method) => match component_method {
                ComponentMethodInvocation::SetRoyaltyConfig(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ComponentMethodInvocation::ClaimRoyalty(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethodInvocation::Package(package_method) => match package_method {
                PackageMethodInvocation::SetRoyaltyConfig(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                PackageMethodInvocation::ClaimRoyalty(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethodInvocation::Bucket(bucket_method) => match bucket_method {
                BucketMethodInvocation::Take(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                BucketMethodInvocation::CreateProof(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                BucketMethodInvocation::TakeNonFungibles(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                BucketMethodInvocation::GetNonFungibleIds(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                BucketMethodInvocation::GetAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                BucketMethodInvocation::Put(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                BucketMethodInvocation::GetResourceAddress(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethodInvocation::AuthZoneStack(auth_zone_method) => match auth_zone_method {
                AuthZoneStackMethodInvocation::Pop(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethodInvocation::Push(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethodInvocation::CreateProof(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethodInvocation::CreateProofByAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethodInvocation::CreateProofByIds(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethodInvocation::Clear(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethodInvocation::Drain(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AuthZoneStackMethodInvocation::AssertAuthRule(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethodInvocation::Proof(proof_method) => match proof_method {
                ProofMethodInvocation::GetAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ProofMethodInvocation::GetNonFungibleIds(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ProofMethodInvocation::GetResourceAddress(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ProofMethodInvocation::Clone(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethodInvocation::Vault(vault_method) => match vault_method {
                VaultMethodInvocation::Take(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethodInvocation::Put(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethodInvocation::LockFee(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethodInvocation::TakeNonFungibles(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethodInvocation::GetAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethodInvocation::GetResourceAddress(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethodInvocation::GetNonFungibleIds(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethodInvocation::CreateProof(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethodInvocation::CreateProofByAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethodInvocation::CreateProofByIds(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethodInvocation::Recall(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                VaultMethodInvocation::RecallNonFungibles(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethodInvocation::AccessRulesChain(access_rules_method) => {
                match access_rules_method {
                    AccessRulesChainMethodInvocation::AddAccessCheck(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    }
                    AccessRulesChainMethodInvocation::SetMethodAccessRule(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    }
                    AccessRulesChainMethodInvocation::SetMethodMutability(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    }
                    AccessRulesChainMethodInvocation::SetGroupAccessRule(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    }
                    AccessRulesChainMethodInvocation::SetGroupMutability(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    }
                    AccessRulesChainMethodInvocation::GetLength(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    }
                }
            }
            NativeMethodInvocation::Metadata(metadata_method) => match metadata_method {
                MetadataMethodInvocation::Set(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                MetadataMethodInvocation::Get(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethodInvocation::ResourceManager(resman_method) => match resman_method {
                ResourceManagerMethodInvocation::Burn(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethodInvocation::UpdateVaultAuth(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethodInvocation::LockVaultAuth(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethodInvocation::CreateVault(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethodInvocation::CreateBucket(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethodInvocation::Mint(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethodInvocation::GetResourceType(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethodInvocation::GetTotalSupply(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethodInvocation::UpdateNonFungibleData(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethodInvocation::NonFungibleExists(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ResourceManagerMethodInvocation::GetNonFungible(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethodInvocation::EpochManager(epoch_manager_method) => {
                match epoch_manager_method {
                    EpochManagerMethodInvocation::GetCurrentEpoch(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    }
                    EpochManagerMethodInvocation::NextRound(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    }
                    EpochManagerMethodInvocation::SetEpoch(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    }
                    EpochManagerMethodInvocation::CreateValidator(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    }
                    EpochManagerMethodInvocation::UpdateValidator(invocation) => {
                        let rtn = api.invoke(invocation)?;
                        Ok(Box::new(rtn))
                    }
                }
            }
            NativeMethodInvocation::Validator(validator_method) => match validator_method {
                ValidatorMethodInvocation::Register(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ValidatorMethodInvocation::Unregister(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethodInvocation::Clock(clock_method) => match clock_method {
                ClockMethodInvocation::SetCurrentTime(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ClockMethodInvocation::GetCurrentTime(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                ClockMethodInvocation::CompareCurrentTime(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethodInvocation::Worktop(worktop_method) => match worktop_method {
                WorktopMethodInvocation::TakeNonFungibles(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethodInvocation::Put(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethodInvocation::Drain(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethodInvocation::AssertContainsNonFungibles(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethodInvocation::AssertContains(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethodInvocation::AssertContainsAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethodInvocation::TakeAll(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                WorktopMethodInvocation::TakeAmount(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethodInvocation::TransactionRuntime(method) => match method {
                TransactionRuntimeMethodInvocation::Get(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                TransactionRuntimeMethodInvocation::GenerateUuid(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
            NativeMethodInvocation::Logger(method) => match method {
                LoggerInvocation::Log(invocation) => {
                    let rtn = api.invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            },
        },
    }
}
