use crate::errors::{InterpreterError, RuntimeError};
use crate::types::*;
use radix_engine_interface::api::component::*;
use radix_engine_interface::api::node_modules::{auth::*, metadata::*};
use radix_engine_interface::api::package::*;

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
        NativeFn::TransactionProcessor(_) => Err(RuntimeError::InterpreterError(
            InterpreterError::DisallowedInvocation(native_fn),
        )),
        NativeFn::Root => Err(RuntimeError::InterpreterError(
            InterpreterError::DisallowedInvocation(native_fn),
        )),
    }
}
