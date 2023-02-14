use crate::errors::{InterpreterError, RuntimeError};
use crate::types::*;
use radix_engine_interface::api::component::*;
use radix_engine_interface::api::node_modules::{auth::*, metadata::*};
use radix_engine_interface::api::package::*;
use radix_engine_interface::blueprints::resource::WorktopAssertContainsInvocation;
use radix_engine_interface::blueprints::{logger::*, resource::*, transaction_runtime::*};

pub fn resolve_native(
    native_fn: NativeFn,
    invocation: Vec<u8>,
) -> Result<CallTableInvocation, RuntimeError> {
    match native_fn {
        NativeFn::Component(component_fn) => match component_fn {
            ComponentFn::Globalize => {
                let invocation = scrypto_decode::<ComponentGlobalizeInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            ComponentFn::GlobalizeWithOwner => {
                let invocation =
                    scrypto_decode::<ComponentGlobalizeWithOwnerInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            ComponentFn::SetRoyaltyConfig => {
                let invocation = scrypto_decode::<ComponentSetRoyaltyConfigInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            ComponentFn::ClaimRoyalty => {
                let invocation = scrypto_decode::<ComponentClaimRoyaltyInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
        },
        NativeFn::Package(package_fn) => match package_fn {
            PackageFn::Publish => {
                let invocation = scrypto_decode::<PackagePublishInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            PackageFn::PublishNative => {
                let invocation = scrypto_decode::<PackagePublishNativeInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            PackageFn::SetRoyaltyConfig => {
                let invocation = scrypto_decode::<PackageSetRoyaltyConfigInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            PackageFn::ClaimRoyalty => {
                let invocation = scrypto_decode::<PackageClaimRoyaltyInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
        },
        NativeFn::Bucket(bucket_fn) => match bucket_fn {
            BucketFn::Take => {
                let invocation = scrypto_decode::<BucketTakeInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            BucketFn::CreateProof => {
                let invocation = scrypto_decode::<BucketCreateProofInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            BucketFn::TakeNonFungibles => {
                let invocation = scrypto_decode::<BucketTakeNonFungiblesInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            BucketFn::GetNonFungibleLocalIds => {
                let invocation =
                    scrypto_decode::<BucketGetNonFungibleLocalIdsInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            BucketFn::GetAmount => {
                let invocation = scrypto_decode::<BucketGetAmountInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            BucketFn::Put => {
                let invocation = scrypto_decode::<BucketPutInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            BucketFn::GetResourceAddress => {
                let invocation = scrypto_decode::<BucketGetResourceAddressInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
        },
        NativeFn::AuthZoneStack(auth_zone_fn) => match auth_zone_fn {
            AuthZoneStackFn::Pop => {
                let invocation = scrypto_decode::<AuthZonePopInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            AuthZoneStackFn::Push => {
                let invocation = scrypto_decode::<AuthZonePushInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            AuthZoneStackFn::CreateProof => {
                let invocation = scrypto_decode::<AuthZoneCreateProofInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            AuthZoneStackFn::CreateProofByAmount => {
                let invocation =
                    scrypto_decode::<AuthZoneCreateProofByAmountInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            AuthZoneStackFn::CreateProofByIds => {
                let invocation = scrypto_decode::<AuthZoneCreateProofByIdsInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            AuthZoneStackFn::Clear => {
                let invocation = scrypto_decode::<AuthZoneClearInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            AuthZoneStackFn::Drain => {
                let invocation = scrypto_decode::<AuthZoneDrainInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            AuthZoneStackFn::AssertAccessRule => {
                let invocation = scrypto_decode::<AuthZoneAssertAccessRuleInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
        },
        NativeFn::Proof(proof_fn) => match proof_fn {
            ProofFn::GetAmount => {
                let invocation = scrypto_decode::<ProofGetAmountInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            ProofFn::GetNonFungibleLocalIds => {
                let invocation =
                    scrypto_decode::<ProofGetNonFungibleLocalIdsInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            ProofFn::GetResourceAddress => {
                let invocation = scrypto_decode::<ProofGetResourceAddressInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
        },
        NativeFn::AccessRulesChain(access_rules_fn) => match access_rules_fn {
            AccessRulesChainFn::AddAccessCheck => {
                let invocation = scrypto_decode::<AccessRulesAddAccessCheckInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            AccessRulesChainFn::SetMethodAccessRule => {
                let invocation =
                    scrypto_decode::<AccessRulesSetMethodAccessRuleInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            AccessRulesChainFn::SetMethodMutability => {
                let invocation =
                    scrypto_decode::<AccessRulesSetMethodMutabilityInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            AccessRulesChainFn::SetGroupAccessRule => {
                let invocation =
                    scrypto_decode::<AccessRulesSetGroupAccessRuleInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            AccessRulesChainFn::SetGroupMutability => {
                let invocation =
                    scrypto_decode::<AccessRulesSetGroupMutabilityInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            AccessRulesChainFn::GetLength => {
                let invocation = scrypto_decode::<AccessRulesGetLengthInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
        },
        NativeFn::Metadata(metadata_fn) => match metadata_fn {
            MetadataFn::Set => {
                let invocation = scrypto_decode::<MetadataSetInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            MetadataFn::Get => {
                let invocation = scrypto_decode::<MetadataGetInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
        },
        NativeFn::Logger(logger_fn) => match logger_fn {
            LoggerFn::Log => {
                let invocation = scrypto_decode::<LoggerLogInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
        },
        NativeFn::Worktop(worktop_fn) => match worktop_fn {
            WorktopFn::TakeNonFungibles => {
                let invocation = scrypto_decode::<WorktopTakeNonFungiblesInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            WorktopFn::Put => {
                let invocation = scrypto_decode::<WorktopPutInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            WorktopFn::Drain => {
                let invocation = scrypto_decode::<WorktopDrainInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            WorktopFn::AssertContainsNonFungibles => {
                let invocation =
                    scrypto_decode::<WorktopAssertContainsNonFungiblesInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            WorktopFn::AssertContains => {
                let invocation = scrypto_decode::<WorktopAssertContainsInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            WorktopFn::AssertContainsAmount => {
                let invocation =
                    scrypto_decode::<WorktopAssertContainsAmountInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            WorktopFn::TakeAll => {
                let invocation = scrypto_decode::<WorktopTakeAllInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            WorktopFn::TakeAmount => {
                let invocation = scrypto_decode::<WorktopTakeAmountInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
        },
        NativeFn::TransactionRuntime(tx_runtime_fn) => match tx_runtime_fn {
            TransactionRuntimeFn::GetHash => {
                let invocation = scrypto_decode::<TransactionRuntimeGetHashInvocation>(&invocation)
                    .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
            TransactionRuntimeFn::GenerateUuid => {
                let invocation =
                    scrypto_decode::<TransactionRuntimeGenerateUuidInvocation>(&invocation)
                        .map_err(|_| InterpreterError::InvalidInvocation)?;
                Ok(invocation.into())
            }
        },
        NativeFn::TransactionProcessor(_) => Err(RuntimeError::InterpreterError(
            InterpreterError::DisallowedInvocation(native_fn),
        )),
        NativeFn::Root => Err(RuntimeError::InterpreterError(
            InterpreterError::DisallowedInvocation(native_fn),
        )),
    }
}
