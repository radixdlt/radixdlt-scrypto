use crate::errors::{RuntimeError, SystemUpstreamError};
use crate::types::*;
use crate::vm::wasm::*;
use crate::vm::{ScryptoRuntime, VmInvoke};
use radix_engine_interface::api::{ClientApi, ClientTransactionLimitsApi};

pub struct ScryptoVm<W: WasmEngine> {
    pub wasm_engine: W,
    /// WASM Instrumenter
    pub wasm_instrumenter: WasmInstrumenter,
    /// WASM metering config
    pub wasm_metering_config: WasmMeteringConfig,
}

impl<W: WasmEngine + Default> Default for ScryptoVm<W> {
    fn default() -> Self {
        Self {
            wasm_engine: W::default(),
            wasm_instrumenter: WasmInstrumenter::default(),
            wasm_metering_config: WasmMeteringConfig::default(),
        }
    }
}

impl<W: WasmEngine> ScryptoVm<W> {
    pub fn create_instance(
        &self,
        package_address: &PackageAddress,
        code: &[u8],
    ) -> ScryptoVmInstance<W::WasmInstance> {
        let instrumented_code =
            self.wasm_instrumenter
                .instrument(*package_address, code, self.wasm_metering_config);
        let instance = self.wasm_engine.instantiate(&instrumented_code);
        ScryptoVmInstance { instance }
    }
}

pub struct ScryptoVmInstance<I: WasmInstance> {
    instance: I,
}

impl<I: WasmInstance> VmInvoke for ScryptoVmInstance<I> {
    fn invoke<Y>(
        &mut self,
        receiver: Option<&NodeId>,
        func_name: &str,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + ClientTransactionLimitsApi<RuntimeError>,
    {
        let rtn = {
            let mut runtime: Box<dyn WasmRuntime> = Box::new(ScryptoRuntime::new(api));

            let mut input = Vec::new();
            if let Some(node_id) = receiver {
                input.push(
                    runtime
                        .allocate_buffer(
                            scrypto_encode(node_id).expect("Failed to encode object id"),
                        )
                        .expect("Failed to allocate buffer"),
                );
            }
            input.push(
                runtime
                    .allocate_buffer(args.as_slice().to_vec())
                    .expect("Failed to allocate buffer"),
            );
            self.instance
                .invoke_export(func_name, input, &mut runtime)?
        };

        let consumed = self.instance.consumed_memory()?;
        api.update_wasm_memory_usage(consumed)?;

        let output = IndexedScryptoValue::from_vec(rtn).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::OutputDecodeError(e))
        })?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    const _: () = {
        fn assert_sync<T: Sync>() {}

        fn assert_all() {
            // The ScryptoInterpreter struct captures the code and module template caches.
            // We therefore share a ScryptoInterpreter as a shared cache across Engine runs on the node.
            // This allows EG multiple mempool submission validations via the Core API at the same time
            // This test ensures the requirement for this cache to be Sync isn't broken
            // (At least when we compile with std, as the node does)
            #[cfg(not(feature = "alloc"))]
            assert_sync::<crate::vm::ScryptoVm<crate::vm::wasm::DefaultWasmEngine>>();
        }
    };
}
