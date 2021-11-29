use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::rust::vec::Vec;
use crate::types::*;

/// Calls a function.
pub fn call_function(
    package_address: Address,
    blueprint_name: &str,
    function: &str,
    args: Vec<Vec<u8>>,
) -> Vec<u8> {
    let input = CallFunctionInput {
        package_address,
        blueprint_name: blueprint_name.to_owned(),
        function: function.to_owned(),
        args,
    };
    let output: CallFunctionOutput = call_kernel(CALL_FUNCTION, input);

    output.rtn
}

/// Calls a method.
pub fn call_method(component_address: Address, method: &str, args: Vec<Vec<u8>>) -> Vec<u8> {
    let input = CallMethodInput {
        component_address,
        method: method.to_owned(),
        args,
    };
    let output: CallMethodOutput = call_kernel(CALL_METHOD, input);

    output.rtn
}
