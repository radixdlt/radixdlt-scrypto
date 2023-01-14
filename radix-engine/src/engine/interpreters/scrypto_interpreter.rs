use crate::engine::*;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance, WasmInstrumenter, WasmMeteringConfig, WasmRuntime};
use radix_engine_interface::api::api::{ActorApi, EngineApi, InvokableModel};
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::data::{match_schema_with_value, IndexedScryptoValue};

pub struct ScryptoExecutorToParsed<I: WasmInstance> {
    instance: I,
    export_name: String,
    component_id: Option<ComponentId>,
    args: Vec<u8>,
    rtn_type: Type,
}

impl<I: WasmInstance> Executor for ScryptoExecutorToParsed<I> {
    type Output = IndexedScryptoValue;

    fn execute<Y>(
        mut self,
        api: &mut Y,
    ) -> Result<(IndexedScryptoValue, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + EngineApi<RuntimeError>
            + InvokableModel<RuntimeError>
            + ActorApi<RuntimeError>,
    {
        let mut args = Vec::new();
        if let Some(component_id) = self.component_id {
            args.push(scrypto_encode(&component_id).unwrap());
        }
        args.push(self.args);

        let output = {
            let mut runtime: Box<dyn WasmRuntime> = Box::new(RadixEngineWasmRuntime::new(api));
            self.instance
                .invoke_export(&self.export_name, args, &mut runtime)
                .map_err(|e| match e {
                    InvokeError::Error(e) => RuntimeError::KernelError(KernelError::WasmError(e)),
                    InvokeError::Downstream(runtime_error) => runtime_error,
                })?
        };

        let rtn = if !match_schema_with_value(&self.rtn_type, output.as_value()) {
            Err(RuntimeError::KernelError(
                KernelError::InvalidScryptoFnOutput,
            ))
        } else {
            let update = CallFrameUpdate {
                node_refs_to_copy: output
                    .global_references()
                    .into_iter()
                    .map(|a| RENodeId::Global(a))
                    .collect(),
                nodes_to_move: output
                    .owned_node_ids()
                    .map_err(|e| RuntimeError::KernelError(KernelError::ReadOwnedNodesError(e)))?
                    .into_iter()
                    .collect(),
            };
            Ok((output, update))
        };

        rtn
    }
}

pub struct ScryptoExecutor<I: WasmInstance> {
    instance: I,
    component_id: Option<ComponentId>,
    args: Vec<u8>,
    export_name: String,
    rtn_type: Type,
}

impl<I: WasmInstance> Executor for ScryptoExecutor<I> {
    type Output = Vec<u8>;

    fn execute<Y>(self, api: &mut Y) -> Result<(Vec<u8>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + EngineApi<RuntimeError>
            + InvokableModel<RuntimeError>
            + ActorApi<RuntimeError>,
    {
        ScryptoExecutorToParsed {
            instance: self.instance,
            args: self.args,
            component_id: self.component_id,
            export_name: self.export_name,
            rtn_type: self.rtn_type,
        }
        .execute(api)
        .map(|(indexed, update)| (indexed.into_vec(), update))
    }
}

pub struct ScryptoInterpreter<W: WasmEngine> {
    pub wasm_engine: W,
    /// WASM Instrumenter
    pub wasm_instrumenter: WasmInstrumenter,
    /// WASM metering config
    pub wasm_metering_config: WasmMeteringConfig,
}

impl<W: WasmEngine + Default> Default for ScryptoInterpreter<W> {
    fn default() -> Self {
        Self {
            wasm_engine: W::default(),
            wasm_instrumenter: WasmInstrumenter::default(),
            wasm_metering_config: WasmMeteringConfig::default(),
        }
    }
}

impl<W: WasmEngine> ScryptoInterpreter<W> {
    pub fn create_executor(
        &self,
        package_address: PackageAddress,
        code: &[u8],
        export_name: String,
        component_id: Option<ComponentId>,
        args: Vec<u8>,
        rtn_type: Type,
    ) -> ScryptoExecutor<W::WasmInstance> {
        let instrumented_code =
            self.wasm_instrumenter
                .instrument(package_address, code, self.wasm_metering_config);
        let instance = self.wasm_engine.instantiate(&instrumented_code);
        ScryptoExecutor {
            instance,
            component_id,
            args,
            export_name,
            rtn_type,
        }
    }

    pub fn create_executor_to_parsed(
        &self,
        package_address: PackageAddress,
        code: &[u8],
        export_name: String,
        component_id: Option<ComponentId>,
        args: Vec<u8>,
        rtn_type: Type,
    ) -> ScryptoExecutorToParsed<W::WasmInstance> {
        let instrumented_code =
            self.wasm_instrumenter
                .instrument(package_address, code, self.wasm_metering_config);
        let instance = self.wasm_engine.instantiate(&instrumented_code);
        ScryptoExecutorToParsed {
            instance,
            export_name,
            component_id,
            args,
            rtn_type,
        }
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
            assert_sync::<crate::engine::ScryptoInterpreter<crate::wasm::DefaultWasmEngine>>();
        }
    };
}
