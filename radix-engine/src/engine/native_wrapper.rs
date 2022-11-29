use crate::engine::errors::KernelError;
use crate::engine::*;
use crate::types::*;
use radix_engine_interface::api::api::SysInvokableNative;
use radix_engine_interface::api::types::{
    AccessRulesMethod, AuthZoneStackMethod, BucketMethod, EpochManagerFunction, EpochManagerMethod,
    NativeFn, NativeFunction, NativeMethod, PackageFunction, ProofMethod, ResourceManagerFunction,
    ResourceManagerMethod, TransactionProcessorFunction, VaultMethod, WorktopMethod,
};
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::model::*;

// TODO: Cleanup
pub fn parse_and_invoke_native_fn<'a, Y>(
    native_fn: NativeFn,
    args: Vec<u8>,
    system_api: &mut Y,
) -> Result<IndexedScryptoValue, RuntimeError>
where
    Y: SysInvokableNative<RuntimeError>,
{
    match native_fn {
        NativeFn::Function(native_function) => match native_function {
            NativeFunction::EpochManager(EpochManagerFunction::Create) => {
                let invocation: EpochManagerCreateInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .sys_invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            NativeFunction::Clock(ClockFunction::Create) => {
                let invocation: ClockCreateInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .sys_invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            NativeFunction::ResourceManager(ResourceManagerFunction::BurnBucket) => {
                let invocation: ResourceManagerBucketBurnInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .sys_invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            NativeFunction::ResourceManager(ResourceManagerFunction::Create) => {
                let invocation: ResourceManagerCreateInvocation = scrypto_decode(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                system_api
                    .sys_invoke(invocation)
                    .map(|a| IndexedScryptoValue::from_typed(&a))
            }
            NativeFunction::TransactionProcessor(TransactionProcessorFunction::Run) => {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidInvocation,
                ));
            }
            NativeFunction::Package(package_function) => match package_function {
                PackageFunction::PublishNoOwner => {
                    let invocation: PackagePublishNoOwnerInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                PackageFunction::PublishWithOwner => {
                    let invocation: PackagePublishWithOwnerInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
            },
        },
        NativeFn::Method(native_method) => match native_method {
            NativeMethod::Bucket(bucket_method) => match bucket_method {
                BucketMethod::Take => {
                    let invocation: BucketTakeInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                BucketMethod::CreateProof => {
                    let invocation: BucketCreateProofInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                BucketMethod::TakeNonFungibles => {
                    let invocation: BucketTakeNonFungiblesInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                BucketMethod::GetNonFungibleIds => {
                    let invocation: BucketGetNonFungibleIdsInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                BucketMethod::GetAmount => {
                    let invocation: BucketGetAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                BucketMethod::Put => {
                    let invocation: BucketPutInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                BucketMethod::GetResourceAddress => {
                    let invocation: BucketGetResourceAddressInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
            },
            NativeMethod::AuthZoneStack(auth_zone_method) => match auth_zone_method {
                AuthZoneStackMethod::Pop => {
                    let invocation: AuthZonePopInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                AuthZoneStackMethod::Push => {
                    let invocation: AuthZonePushInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                AuthZoneStackMethod::CreateProof => {
                    let invocation: AuthZoneCreateProofInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                AuthZoneStackMethod::CreateProofByAmount => {
                    let invocation: AuthZoneCreateProofByAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| {
                        RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                    })?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                AuthZoneStackMethod::CreateProofByIds => {
                    let invocation: AuthZoneCreateProofByIdsInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                AuthZoneStackMethod::Clear => {
                    let invocation: AuthZoneClearInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                AuthZoneStackMethod::Drain => {
                    let invocation: AuthZoneDrainInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
            },
            NativeMethod::Proof(proof_method) => match proof_method {
                ProofMethod::GetAmount => {
                    let invocation: ProofGetAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ProofMethod::GetNonFungibleIds => {
                    let invocation: ProofGetNonFungibleIdsInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ProofMethod::GetResourceAddress => {
                    let invocation: ProofGetResourceAddressInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ProofMethod::Clone => {
                    let invocation: ProofCloneInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
            },
            NativeMethod::Vault(vault_method) => match vault_method {
                VaultMethod::Take => {
                    let invocation: VaultTakeInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                VaultMethod::Put => {
                    let invocation: VaultPutInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                VaultMethod::LockFee => {
                    let invocation: VaultLockFeeInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                VaultMethod::TakeNonFungibles => {
                    let invocation: VaultTakeNonFungiblesInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                VaultMethod::GetAmount => {
                    let invocation: VaultGetAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                VaultMethod::GetResourceAddress => {
                    let invocation: VaultGetResourceAddressInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                VaultMethod::GetNonFungibleIds => {
                    let invocation: VaultGetNonFungibleIdsInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                VaultMethod::CreateProof => {
                    let invocation: VaultCreateProofInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                VaultMethod::CreateProofByAmount => {
                    let invocation: VaultCreateProofByAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                VaultMethod::CreateProofByIds => {
                    let invocation: VaultCreateProofByIdsInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
            },
            NativeMethod::AccessRules(component_method) => match component_method {
                AccessRulesMethod::AddAccessCheck => {
                    let invocation: AccessRulesAddAccessCheckInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
            },
            NativeMethod::Metadata(metadata_method) => match metadata_method {
                MetadataMethod::Set => {
                    let invocation: MetadataSetInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
            },
            NativeMethod::ResourceManager(resman_method) => match resman_method {
                ResourceManagerMethod::Burn => {
                    let invocation: ResourceManagerBurnInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ResourceManagerMethod::UpdateAuth => {
                    let invocation: ResourceManagerUpdateAuthInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ResourceManagerMethod::LockAuth => {
                    let invocation: ResourceManagerLockAuthInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ResourceManagerMethod::CreateVault => {
                    let invocation: ResourceManagerCreateVaultInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ResourceManagerMethod::CreateBucket => {
                    let invocation: ResourceManagerCreateBucketInvocation = scrypto_decode(&args)
                        .map_err(|e| {
                        RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                    })?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ResourceManagerMethod::Mint => {
                    let invocation: ResourceManagerMintInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ResourceManagerMethod::GetMetadata => {
                    let invocation: ResourceManagerGetMetadataInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ResourceManagerMethod::GetResourceType => {
                    let invocation: ResourceManagerGetResourceTypeInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ResourceManagerMethod::GetTotalSupply => {
                    let invocation: ResourceManagerGetTotalSupplyInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ResourceManagerMethod::UpdateMetadata => {
                    let invocation: ResourceManagerUpdateMetadataInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ResourceManagerMethod::UpdateNonFungibleData => {
                    let invocation: ResourceManagerUpdateNonFungibleDataInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ResourceManagerMethod::NonFungibleExists => {
                    let invocation: ResourceManagerNonFungibleExistsInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ResourceManagerMethod::GetNonFungible => {
                    let invocation: ResourceManagerGetNonFungibleInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ResourceManagerMethod::SetResourceAddress => Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidInvocation,
                )),
            },
            NativeMethod::EpochManager(epoch_manager_method) => match epoch_manager_method {
                EpochManagerMethod::GetCurrentEpoch => {
                    let invocation: EpochManagerGetCurrentEpochInvocation = scrypto_decode(&args)
                        .map_err(|e| {
                        RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                    })?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                EpochManagerMethod::SetEpoch => {
                    let invocation: EpochManagerSetEpochInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
            },
            NativeMethod::Clock(clock_method) => match clock_method {
                ClockMethod::GetCurrentTimeRoundedToMinutes => {
                    let invocation: ClockGetCurrentTimeRoundedToMinutesInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                ClockMethod::SetCurrentTime => {
                    let invocation: ClockSetCurrentTimeInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
            },
            NativeMethod::Worktop(worktop_method) => match worktop_method {
                WorktopMethod::TakeNonFungibles => {
                    let invocation: WorktopTakeNonFungiblesInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                WorktopMethod::Put => {
                    let invocation: WorktopPutInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                WorktopMethod::Drain => {
                    let invocation: WorktopDrainInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                WorktopMethod::AssertContainsNonFungibles => {
                    let invocation: WorktopAssertContainsNonFungiblesInvocation =
                        scrypto_decode(&args).map_err(|e| {
                            RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                        })?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                WorktopMethod::AssertContains => {
                    let invocation: WorktopAssertContainsInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                WorktopMethod::AssertContainsAmount => {
                    let invocation: WorktopAssertContainsAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| {
                        RuntimeError::KernelError(KernelError::InvalidSborValue(e))
                    })?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                WorktopMethod::TakeAll => {
                    let invocation: WorktopTakeAllInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
                WorktopMethod::TakeAmount => {
                    let invocation: WorktopTakeAmountInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
            },
            NativeMethod::Component(component_method) => match component_method {
                ComponentMethod::SetRoyaltyConfig => {
                    let invocation: ComponentSetRoyaltyConfigInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
            },
            NativeMethod::Package(package_method) => match package_method {
                PackageMethod::SetRoyaltyConfig => {
                    let invocation: PackageSetRoyaltyConfigInvocation = scrypto_decode(&args)
                        .map_err(|e| RuntimeError::KernelError(KernelError::InvalidSborValue(e)))?;
                    system_api
                        .sys_invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a))
                }
            },
        },
    }
}
