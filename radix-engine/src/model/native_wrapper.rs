use crate::engine::{ApplicationError, LockFlags, RuntimeError, SystemApi};
use crate::model::{NativeOutput, TransactionProcessorError};
use crate::types::*;
use radix_engine_interface::api::InvokableModel;

pub fn resolve_method<Y: SystemApi>(
    receiver: ScryptoReceiver,
    method_name: &str,
    args: &[u8],
    api: &mut Y,
) -> Result<CallTableInvocation, RuntimeError> {
    let invocation = match receiver {
        ScryptoReceiver::Global(component_address) => match component_address {
            ComponentAddress::Identity(..)
            | ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
            | ComponentAddress::EddsaEd25519VirtualIdentity(..) => {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::ResolveError(ResolveError::NotAMethod),
                    ),
                ));
            }
            ComponentAddress::EpochManager(..) | ComponentAddress::Validator(..) => {
                let invocation = EpochManagerPackage::resolve_method_invocation(
                    component_address,
                    method_name,
                    args,
                )
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::ResolveError(e),
                    ))
                })?;
                CallTableInvocation::Native(invocation)
            }
            ComponentAddress::Clock(..) => {
                let invocation =
                    ClockPackage::resolve_method_invocation(component_address, method_name, args)
                        .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                            TransactionProcessorError::ResolveError(e),
                        ))
                    })?;
                CallTableInvocation::Native(NativeInvocation::Clock(invocation))
            }
            ComponentAddress::AccessController(..) => {
                let invocation = AccessControllerPackage::resolve_method_invocation(
                    component_address,
                    method_name,
                    args,
                )
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::ResolveError(e),
                    ))
                })?;
                CallTableInvocation::Native(NativeInvocation::AccessController(invocation))
            }
            ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
            | ComponentAddress::EddsaEd25519VirtualAccount(..)
            | ComponentAddress::Normal(..)
            | ComponentAddress::Account(..) => {
                let component_node_id =
                    RENodeId::Global(GlobalAddress::Component(component_address));
                let component_info = {
                    let handle = api.lock_substate(
                        component_node_id,
                        SubstateOffset::Component(ComponentOffset::Info),
                        LockFlags::read_only(),
                    )?;
                    let substate_ref = api.get_ref(handle)?;
                    let component_info = substate_ref.component_info().clone(); // TODO: Remove clone()
                    api.drop_lock(handle)?;

                    component_info
                };

                let method_invocation = ScryptoInvocation {
                    package_address: component_info.package_address,
                    blueprint_name: component_info.blueprint_name,
                    receiver: Some(ScryptoReceiver::Global(component_address.clone())),
                    fn_name: method_name.to_string(),
                    args: args.to_owned(),
                };
                CallTableInvocation::Scrypto(method_invocation)
            }
        },
        ScryptoReceiver::Component(component_id) => {
            let component_node_id = RENodeId::Component(component_id);
            let component_info = {
                let handle = api.lock_substate(
                    component_node_id,
                    SubstateOffset::Component(ComponentOffset::Info),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.get_ref(handle)?;
                let component_info = substate_ref.component_info().clone(); // TODO: Remove clone()
                api.drop_lock(handle)?;

                component_info
            };

            CallTableInvocation::Scrypto(ScryptoInvocation {
                package_address: component_info.package_address,
                blueprint_name: component_info.blueprint_name,
                receiver: Some(ScryptoReceiver::Component(component_id)),
                fn_name: method_name.to_string(),
                args: args.to_owned(),
            })
        }
    };

    Ok(invocation)
}

pub fn invoke_call_table<Y, E>(
    invocation: CallTableInvocation,
    api: &mut Y,
) -> Result<IndexedScryptoValue, E>
where
    Y: InvokableModel<E>,
{
    match invocation {
        CallTableInvocation::Scrypto(invocation) => {
            let rtn = api.invoke(invocation)?;
            Ok(IndexedScryptoValue::from_value(rtn))
        }
        CallTableInvocation::Native(invocation) => {
            let rtn = invoke_native_fn(invocation, api)?;
            Ok(IndexedScryptoValue::from_typed(rtn.as_ref()))
        }
    }
}

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
        },
        NativeInvocation::Validator(invocation) => match invocation {
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
            ValidatorInvocation::UpdateKey(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ValidatorInvocation::UpdateAcceptDelegatedStake(invocation) => {
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
        NativeInvocation::Identity(invocation) => match invocation {
            IdentityInvocation::Create(invocation) => {
                let rtn = api.invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::Logger(invocation) => match invocation {
            LoggerInvocation::Log(invocation) => {
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
    }
}
