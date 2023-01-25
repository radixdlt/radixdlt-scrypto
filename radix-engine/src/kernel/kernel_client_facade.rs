use crate::errors::RuntimeError;
use crate::kernel::module::BaseModule;
use crate::kernel::*;
use crate::system::invocation::invoke_native::invoke_native_fn;
use crate::system::invocation::invoke_scrypto::invoke_scrypto_fn;
use crate::system::invocation::resolve_function;
use crate::system::invocation::resolve_function::resolve_function;
use crate::system::invocation::resolve_method::resolve_method;
use crate::system::kernel_modules::fee::FeeReserve;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::{LockHandle, RENodeId, ScryptoReceiver};
use radix_engine_interface::api::{
    ClientActorApi, ClientApi, ClientComponentApi, ClientDerefApi, ClientPackageApi,
    ClientStaticInvokeApi, Invokable,
};
use radix_engine_interface::data::*;

impl<'g, 's, W, R, M> ClientDerefApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn deref(&mut self, node_id: RENodeId) -> Result<Option<(RENodeId, LockHandle)>, RuntimeError> {
        self.node_method_deref(node_id)
    }
}

impl<'g, 's, W, R, M> ClientActorApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn fn_identifier(&mut self) -> Result<FnIdentifier, RuntimeError> {
        Ok(self.current_frame.actor.identifier.clone())
    }
}

impl<'g, 's, W, R, M> ClientStaticInvokeApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
}

impl<'g, 's, W, R, M> ClientPackageApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: String,
        function_name: String,
        args: Vec<u8>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        // TODO: Use execution mode?
        let invocation =
            resolve_function(package_address, blueprint_name, function_name, args, self)?;
        Ok(match invocation {
            CallTableInvocation::Native(native) => {
                IndexedScryptoValue::from_typed(invoke_native_fn(native, self)?.as_ref())
            }
            CallTableInvocation::Scrypto(scrypto) => invoke_scrypto_fn(scrypto, self)?,
        })
    }

    fn get_code(&mut self, package_address: PackageAddress) -> Result<PackageCode, RuntimeError> {
        todo!()
    }

    fn get_abi(
        &mut self,
        package_address: PackageAddress,
    ) -> Result<BTreeMap<String, BlueprintAbi>, RuntimeError> {
        todo!()
    }
}

impl<'g, 's, W, R, M> ClientComponentApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn call_method(
        &mut self,
        receiver: ScryptoReceiver,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        // TODO: Use execution mode?
        let invocation = resolve_method(receiver, method_name, &args, self)?;
        Ok(match invocation {
            CallTableInvocation::Native(native) => {
                IndexedScryptoValue::from_typed(invoke_native_fn(native, self)?.as_ref())
            }
            CallTableInvocation::Scrypto(scrypto) => invoke_scrypto_fn(scrypto, self)?,
        })
    }
}

impl<'g, 's, W, R, M> ClientApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
}
