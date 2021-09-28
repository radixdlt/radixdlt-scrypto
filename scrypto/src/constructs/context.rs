use crate::kernel::*;
use crate::types::*;

/// A utility for accessing transaction context.
#[derive(Debug)]
pub struct Context {}

impl Context {
    pub fn package_address() -> Address {
        let input = GetPackageAddressInput {};
        let output: GetPackageAddressOutput = call_kernel(GET_PACKAGE_ADDRESS, input);
        output.address
    }

    pub fn transaction_hash() -> H256 {
        let input = GetTransactionHashInput {};
        let output: GetTransactionHashOutput = call_kernel(GET_TRANSACTION_HASH, input);
        output.tx_hash
    }
}
