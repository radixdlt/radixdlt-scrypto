use radix_engine_lib::model::*;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::core::*;
use radix_engine_lib::data::ScryptoCustomTypeId;

/// Represents a published package.
#[derive(Debug)]
pub struct BorrowedPackage(pub(crate) PackageAddress);

impl BorrowedPackage {
    /// Invokes a function on this package.
    pub fn call<T: Decode<ScryptoCustomTypeId>>(
        &self,
        blueprint_name: &str,
        function: &str,
        args: Vec<u8>,
    ) -> T {
        Runtime::call_function(self.0, blueprint_name, function, args)
    }
}
