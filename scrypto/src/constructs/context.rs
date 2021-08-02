use crate::kernel::*;
use crate::types::*;

/// The execution context.
pub struct Context {}

impl Context {
    pub fn address() -> Address {
        let input = GetContextAddressInput {};
        let output: GetContextAddressOutput = call_kernel(GET_CONTEXT_ADDRESS, input);
        output.address
    }
}
