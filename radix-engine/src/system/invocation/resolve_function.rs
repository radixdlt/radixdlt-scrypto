use crate::blueprints::transaction_processor::TransactionProcessorError;
use crate::errors::{ApplicationError, InterpreterError, RuntimeError, ScryptoFnResolvingError};
use crate::kernel::kernel_api::{KernelSubstateApi, LockFlags};
use crate::kernel::*;
use crate::types::*;
use radix_engine_interface::api::types::{ScryptoInvocation, ScryptoReceiver};
use radix_engine_interface::api::ClientDerefApi;
use radix_engine_interface::data::*;

pub fn resolve_function<Y: KernelNodeApi + KernelSubstateApi>(
    package_address: PackageAddress,
    blueprint_name: String,
    function_name: String,
    args: Vec<u8>,
    api: &mut Y,
) -> Result<CallTableInvocation, RuntimeError> {
    Ok(CallTableInvocation::Scrypto(ScryptoInvocation {
        package_address,
        blueprint_name,
        fn_name: function_name,
        receiver: None,
        args,
    }))
}
