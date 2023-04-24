use radix_engine_common::types::*;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

pub trait ClientBlueprintApi<E> {
    /// Calls a function on a blueprint
    fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;
}
