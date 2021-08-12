use crate::kernel::*;
use crate::types::*;

/// The execution context.
#[derive(Debug)]
pub struct Context {}

impl Context {
    pub fn blueprint_address() -> Address {
        let input = GetBlueprintAddressInput {};
        let output: GetBlueprintAddressOutput = call_kernel(GET_BLUEPRINT_ADDRESS, input);
        output.address
    }

    pub fn transaction_hash() -> H256 {
        let input = GetTransactionHashInput {};
        let output: GetTransactionHashOutput = call_kernel(GET_TRANSACTION_HASH, input);
        output.tx_hash
    }
}
