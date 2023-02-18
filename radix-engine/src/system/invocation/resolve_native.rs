use crate::errors::{InterpreterError, RuntimeError};
use crate::types::*;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::package::*;

pub fn resolve_native(
    native_fn: NativeFn,
    invocation: Vec<u8>,
) -> Result<CallTableInvocation, RuntimeError> {
    match native_fn {
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
        },
        NativeFn::AccessRulesChain(access_rules_fn) => match access_rules_fn {
            AccessRulesChainFn::GetLength => {
                let invocation = scrypto_decode::<AccessRulesGetLengthInvocation>(&invocation)
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
