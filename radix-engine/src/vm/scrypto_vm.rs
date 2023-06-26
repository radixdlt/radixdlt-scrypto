use crate::errors::{RuntimeError, SystemUpstreamError};
use crate::types::*;
use crate::vm::vm::VmInvoke;
use crate::vm::wasm::*;
use crate::vm::wasm_runtime::ScryptoRuntime;
use radix_engine_interface::api::{ClientApi, ClientLimitsApi};
use resources_tracker_macro::trace_resources;

pub struct ScryptoVm<W: WasmEngine> {
    pub wasm_engine: W,
    /// WASM Instrumenter
    pub wasm_instrumenter: WasmInstrumenter,
    /// WASM metering config
    pub wasm_instrumenter_config: WasmInstrumenterConfig,
}

impl<W: WasmEngine + Default> Default for ScryptoVm<W> {
    fn default() -> Self {
        Self {
            wasm_engine: W::default(),
            wasm_instrumenter: WasmInstrumenter::default(),
            wasm_instrumenter_config: WasmInstrumenterConfig::v1(),
        }
    }
}

impl<W: WasmEngine> ScryptoVm<W> {
    pub fn create_instance(
        &self,
        package_address: &PackageAddress,
        code: &[u8],
    ) -> ScryptoVmInstance<W::WasmInstance> {
        let instrumented_code = self
            .wasm_instrumenter
            .instrument(*package_address, code, self.wasm_instrumenter_config)
            .expect("Failed to re-instrument");
        ScryptoVmInstance {
            instance: self.wasm_engine.instantiate(&instrumented_code),
            package_address: *package_address,
        }
    }
}

pub struct ScryptoVmInstance<I: WasmInstance> {
    instance: I,
    package_address: PackageAddress,
}

impl<I: WasmInstance> VmInvoke for ScryptoVmInstance<I> {
    #[trace_resources(log=self.package_address.to_hex(),log=export_name)]
    fn invoke<Y>(
        &mut self,
        export_name: &str,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + ClientLimitsApi<RuntimeError>,
    {
        let rtn = {
            let mut runtime: Box<dyn WasmRuntime> = Box::new(ScryptoRuntime::new(
                api,
                self.package_address,
                export_name.to_string(),
            ));

            let mut input = Vec::new();
            input.push(
                runtime
                    .allocate_buffer(args.as_slice().to_vec())
                    .expect("Failed to allocate buffer"),
            );
            self.instance
                .invoke_export(export_name, input, &mut runtime)?
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
