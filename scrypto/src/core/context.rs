use crate::engine::*;
use crate::types::*;

/// A utility for accessing transaction context.
#[derive(Debug)]
pub struct Context {}

impl Context {
    /// Returns the address of the running package.
    pub fn package_address() -> Address {
        let input = GetPackageAddressInput {};
        let output: GetPackageAddressOutput = call_engine(GET_PACKAGE_ADDRESS, input);
        output.package_address
    }

    /// Returns the transaction hash.
    pub fn transaction_hash() -> H256 {
        let input = GetTransactionHashInput {};
        let output: GetTransactionHashOutput = call_engine(GET_TRANSACTION_HASH, input);
        output.transaction_hash
    }

    /// Returns the current epoch number.
    pub fn current_epoch() -> u64 {
        let input = GetCurrentEpochInput {};
        let output: GetCurrentEpochOutput = call_engine(GET_CURRENT_EPOCH, input);
        output.current_epoch
    }
}
