use crate::{blueprints::transaction_processor::NativeOutput, types::*};
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
            ValidatorInvocation::ClaimXrd(invocation) => {
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
        NativeInvocation::AccessController(method) => match method {
            AccessControllerInvocation::CreateGlobal(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerInvocation::CreateProof(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerInvocation::InitiateRecoveryAsPrimary(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerInvocation::InitiateRecoveryAsRecovery(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerInvocation::QuickConfirmPrimaryRoleRecoveryProposal(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerInvocation::QuickConfirmRecoveryRoleRecoveryProposal(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerInvocation::TimedConfirmRecovery(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerInvocation::CancelPrimaryRoleRecoveryProposal(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerInvocation::CancelRecoveryRoleRecoveryProposal(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerInvocation::LockPrimaryRole(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerInvocation::UnlockPrimaryRole(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            AccessControllerInvocation::StopTimedRecovery(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::KeyValueStore(method) => match method {
            KeyValueStoreInvocation::Create(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            KeyValueStoreInvocation::Get(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            KeyValueStoreInvocation::GetMut(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            KeyValueStoreInvocation::Insert(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
    }
}
