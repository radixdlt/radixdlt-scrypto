use crate::model::NativeOutput;
use crate::types::*;
use radix_engine_interface::api::api::InvokableModel;

pub fn invoke_native_fn<Y, E>(
    native_invocation: NativeInvocation,
    api: &mut Y,
) -> Result<Box<dyn NativeOutput>, E>
where
    Y: InvokableModel<E>,
{
    match native_invocation {
        NativeInvocation::Component(component_method) => match component_method {
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
        NativeInvocation::Package(package_method) => match package_method {
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
        NativeInvocation::Bucket(bucket_method) => match bucket_method {
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
            BucketInvocation::GetNonFungibleIds(invocation) => {
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
        NativeInvocation::AuthZoneStack(auth_zone_method) => match auth_zone_method {
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
        NativeInvocation::Proof(proof_method) => match proof_method {
            ProofInvocation::GetAmount(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ProofInvocation::GetNonFungibleIds(invocation) => {
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
        NativeInvocation::Vault(vault_method) => match vault_method {
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
            VaultInvocation::GetNonFungibleIds(invocation) => {
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
        NativeInvocation::AccessRulesChain(access_rules_method) => match access_rules_method {
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
        },
        NativeInvocation::Metadata(metadata_method) => match metadata_method {
            MetadataInvocation::Set(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            MetadataInvocation::Get(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::ResourceManager(resman_method) => match resman_method {
            ResourceInvocation::Create(invocation) => {
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
            ResourceInvocation::LockVaultAuth(invocation) => {
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
            ResourceInvocation::Mint(invocation) => {
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
        },
        NativeInvocation::EpochManager(epoch_manager_method) => match epoch_manager_method {
            EpochManagerInvocation::Create(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            EpochManagerInvocation::GetCurrentEpoch(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            EpochManagerInvocation::SetEpoch(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::Clock(clock_method) => match clock_method {
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
        NativeInvocation::Worktop(worktop_method) => match worktop_method {
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
            TransactionRuntimeInvocation::Get(invocation) => {
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
