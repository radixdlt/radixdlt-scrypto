use crate::engine::errors::KernelError;
use crate::engine::*;
use crate::model::{
    TransactionProcessorRunInput, WorktopAssertContainsAmountInput, WorktopAssertContainsInput,
    WorktopAssertContainsNonFungiblesInput, WorktopDrainInput, WorktopPutInput,
    WorktopTakeAllInput, WorktopTakeAmountInput, WorktopTakeNonFungiblesInput,
};
use crate::types::*;
use scrypto::resource::AuthZoneDrainInput;

// TODO: Cleanup
pub fn parse_and_invoke_native_function<'y, 'a, Y>(
    native_function: NativeFunction,
    args: Vec<u8>,
    system_api: &'y mut Y,
) -> Result<ScryptoValue, RuntimeError>
where
    Y: SystemApi + Invokable<ScryptoInvocation> + InvokableNative<'a>,
{
    match native_function {
        NativeFunction::EpochManager(EpochManagerFunction::Create) => {
            let invocation: EpochManagerCreateInput = scrypto_decode(&args)
                .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
            system_api
                .invoke(invocation)
                .map(|a| ScryptoValue::from_typed(&a))
        }
        NativeFunction::ResourceManager(ResourceManagerFunction::BurnBucket) => {
            let invocation: ResourceManagerBurnInput = scrypto_decode(&args)
                .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
            system_api
                .invoke(invocation)
                .map(|a| ScryptoValue::from_typed(&a))
        }
        NativeFunction::ResourceManager(ResourceManagerFunction::Create) => {
            let invocation: ResourceManagerCreateInput = scrypto_decode(&args)
                .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
            system_api
                .invoke(invocation)
                .map(|a| ScryptoValue::from_typed(&a))
        }
        NativeFunction::TransactionProcessor(TransactionProcessorFunction::Run) => {
            let invocation: TransactionProcessorRunInput = scrypto_decode(&args)
                .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
            system_api
                .invoke(invocation)
                .map(|a| ScryptoValue::from_typed(&a))
        }
        NativeFunction::Package(PackageFunction::Publish) => {
            let invocation: PackagePublishInput = scrypto_decode(&args)
                .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
            system_api
                .invoke(invocation)
                .map(|a| ScryptoValue::from_typed(&a))
        }
    }
}

// TODO: Cleanup
pub fn parse_and_invoke_native_method<'y, 'a, Y>(
    native_method: NativeMethod,
    args: Vec<u8>,
    system_api: &'y mut Y,
) -> Result<ScryptoValue, RuntimeError>
where
    Y: SystemApi + Invokable<ScryptoInvocation> + InvokableNative<'a>,
{
    match native_method {
        NativeMethod::Bucket(bucket_method) => match bucket_method {
            BucketMethod::Take => {
                let invocation: BucketTakeInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            BucketMethod::CreateProof => {
                let invocation: BucketCreateProofInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            BucketMethod::TakeNonFungibles => {
                let invocation: BucketTakeNonFungiblesInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            BucketMethod::GetNonFungibleIds => {
                let invocation: BucketGetNonFungibleIdsInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            BucketMethod::GetAmount => {
                let invocation: BucketGetAmountInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            BucketMethod::Put => {
                let invocation: BucketPutInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            BucketMethod::GetResourceAddress => {
                let invocation: BucketGetResourceAddressInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::AuthZone(auth_zone_method) => match auth_zone_method {
            AuthZoneMethod::Pop => {
                let invocation: AuthZonePopInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            AuthZoneMethod::Push => {
                let invocation: AuthZonePushInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            AuthZoneMethod::CreateProof => {
                let invocation: AuthZoneCreateProofInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            AuthZoneMethod::CreateProofByAmount => {
                let invocation: AuthZoneCreateProofByAmountInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            AuthZoneMethod::CreateProofByIds => {
                let invocation: AuthZoneCreateProofByIdsInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            AuthZoneMethod::Clear => {
                let invocation: AuthZoneClearInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            AuthZoneMethod::Drain => {
                let invocation: AuthZoneDrainInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::Proof(proof_method) => match proof_method {
            ProofMethod::GetAmount => {
                let invocation: ProofGetAmountInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ProofMethod::GetNonFungibleIds => {
                let invocation: ProofGetNonFungibleIdsInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ProofMethod::GetResourceAddress => {
                let invocation: ProofGetResourceAddressInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ProofMethod::Clone => {
                let invocation: ProofCloneInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::Vault(vault_method) => match vault_method {
            VaultMethod::Take => {
                let invocation: VaultTakeInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            VaultMethod::Put => {
                let invocation: VaultPutInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            VaultMethod::LockFee => {
                let invocation: VaultLockFeeInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            VaultMethod::TakeNonFungibles => {
                let invocation: VaultTakeNonFungiblesInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            VaultMethod::GetAmount => {
                let invocation: VaultGetAmountInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            VaultMethod::GetResourceAddress => {
                let invocation: VaultGetResourceAddressInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            VaultMethod::GetNonFungibleIds => {
                let invocation: VaultGetNonFungibleIdsInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            VaultMethod::CreateProof => {
                let invocation: VaultCreateProofInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            VaultMethod::CreateProofByAmount => {
                let invocation: VaultCreateProofByAmountInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            VaultMethod::CreateProofByIds => {
                let invocation: VaultCreateProofByIdsInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::Component(component_method) => match component_method {
            ComponentMethod::AddAccessCheck => {
                let invocation: ComponentAddAccessCheckInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::ResourceManager(resman_method) => match resman_method {
            ResourceManagerMethod::Burn => {
                let invocation: ResourceManagerBurnInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::UpdateAuth => {
                let invocation: ResourceManagerUpdateAuthInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::LockAuth => {
                let invocation: ResourceManagerLockAuthInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::CreateVault => {
                let invocation: ResourceManagerCreateVaultInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::CreateBucket => {
                let invocation: ResourceManagerCreateBucketInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::Mint => {
                let invocation: ResourceManagerMintInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::GetMetadata => {
                let invocation: ResourceManagerGetMetadataInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::GetResourceType => {
                let invocation: ResourceManagerGetResourceTypeInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::GetTotalSupply => {
                let invocation: ResourceManagerGetTotalSupplyInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::UpdateMetadata => {
                let invocation: ResourceManagerUpdateMetadataInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::UpdateNonFungibleData => {
                let invocation: ResourceManagerUpdateNonFungibleDataInput =
                    scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::NonFungibleExists => {
                let invocation: ResourceManagerNonFungibleExistsInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::GetNonFungible => {
                let invocation: ResourceManagerGetNonFungibleInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            ResourceManagerMethod::SetResourceAddress => {
                let invocation: ResourceManagerSetResourceAddressInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::EpochManager(epoch_manager_method) => match epoch_manager_method {
            EpochManagerMethod::GetCurrentEpoch => {
                let invocation: EpochManagerGetCurrentEpochInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            EpochManagerMethod::SetEpoch => {
                let invocation: EpochManagerSetEpochInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
        },
        NativeMethod::Worktop(worktop_method) => match worktop_method {
            WorktopMethod::TakeNonFungibles => {
                let invocation: WorktopTakeNonFungiblesInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            WorktopMethod::Put => {
                let invocation: WorktopPutInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            WorktopMethod::Drain => {
                let invocation: WorktopDrainInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            WorktopMethod::AssertContainsNonFungibles => {
                let invocation: WorktopAssertContainsNonFungiblesInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            WorktopMethod::AssertContains => {
                let invocation: WorktopAssertContainsInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            WorktopMethod::AssertContainsAmount => {
                let invocation: WorktopAssertContainsAmountInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            WorktopMethod::TakeAll => {
                let invocation: WorktopTakeAllInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
            WorktopMethod::TakeAmount => {
                let invocation: WorktopTakeAmountInput = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                system_api
                    .invoke(invocation)
                    .map(|a| ScryptoValue::from_typed(&a))
            }
        },
    }
}
