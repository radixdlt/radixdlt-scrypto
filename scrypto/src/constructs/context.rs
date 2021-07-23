use crate::abi::*;
use crate::types::*;
use crate::utils::*;

/// The execution context.
pub struct Context {}

impl Context {
    pub fn address() -> Address {
        let input = GetContextAddressInput {};
        let output: GetContextAddressOutput = syscall(GET_CONTEXT_ADDRESS, input);
        output.address
    }
}
