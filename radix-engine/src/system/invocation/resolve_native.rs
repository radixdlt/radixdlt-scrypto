use crate::errors::{InterpreterError, RuntimeError};
use crate::types::*;

pub fn resolve_native(
    native_fn: NativeFn,
    _invocation: Vec<u8>,
) -> Result<CallTableInvocation, RuntimeError> {
    match native_fn {
        NativeFn::TransactionProcessor(_) => Err(RuntimeError::InterpreterError(
            InterpreterError::DisallowedInvocation(native_fn),
        )),
        NativeFn::Root => Err(RuntimeError::InterpreterError(
            InterpreterError::DisallowedInvocation(native_fn),
        )),
    }
}
