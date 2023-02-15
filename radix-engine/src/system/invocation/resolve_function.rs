use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::types::*;
use radix_engine_interface::api::types::ScryptoInvocation;

pub fn resolve_function<Y: KernelNodeApi + KernelSubstateApi>(
    package_address: PackageAddress,
    blueprint_name: String,
    function_name: String,
    args: Vec<u8>,
    _api: &mut Y,
) -> Result<CallTableInvocation, RuntimeError> {
    Ok(CallTableInvocation::Scrypto(ScryptoInvocation {
        package_address,
        blueprint_name,
        fn_name: function_name,
        receiver: None,
        args,
    }))
}
