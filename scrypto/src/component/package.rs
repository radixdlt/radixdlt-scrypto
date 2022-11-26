use radix_engine_interface::data::ScryptoDecode;
use radix_engine_interface::model::*;

use sbor::rust::vec::Vec;

use crate::runtime::*;

/// Represents a published package.
#[derive(Debug)]
pub struct BorrowedPackage(pub(crate) PackageAddress);

impl BorrowedPackage {
    /// Invokes a function on this package.
    pub fn call<T: ScryptoDecode>(&self, blueprint_name: &str, function: &str, args: Vec<u8>) -> T {
        Runtime::call_function(self.0, blueprint_name, function, args)
    }
}
