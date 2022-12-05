use crate::engine::*;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance, WasmInstrumenter, WasmMeteringConfig, WasmRuntime};
use radix_engine_interface::api::api::{EngineApi, InvokableModel};
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::data::{match_schema_with_value, IndexedScryptoValue};

pub struct ScryptoExecutorToParsed<I: WasmInstance> {
    instance: I,
    args: IndexedScryptoValue,
}

impl<I: WasmInstance> Executor for ScryptoExecutorToParsed<I> {
    type Output = IndexedScryptoValue;

    fn execute<Y>(
        mut self,
        api: &mut Y,
    ) -> Result<(IndexedScryptoValue, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let (export_name, return_type) = match api.get_actor() {
            REActor::Method(
                ResolvedMethod::Scrypto {
                    export_name,
                    return_type,
                    ..
                },
                ResolvedReceiver {
                    receiver: RENodeId::Component(..),
                    ..
                },
            ) => (export_name.to_string(), return_type.clone()),
            REActor::Function(ResolvedFunction::Scrypto {
                export_name,
                return_type,
                ..
            }) => (export_name.to_string(), return_type.clone()),

            _ => panic!("Should not get here."),
        };

        let output = {
            let mut runtime: Box<dyn WasmRuntime> = Box::new(RadixEngineWasmRuntime::new(api));
            self.instance
                .invoke_export(&export_name, &self.args, &mut runtime)
                .map_err(|e| match e {
                    InvokeError::Error(e) => RuntimeError::KernelError(KernelError::WasmError(e)),
                    InvokeError::Downstream(runtime_error) => runtime_error,
                })?
        };

        let rtn = if !match_schema_with_value(&return_type, &output.dom) {
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
                nodes_to_move: output.node_ids().into_iter().collect(),
            };
            Ok((output, update))
        };

        rtn
    }
}

pub struct ScryptoExecutor<I: WasmInstance> {
    instance: I,
    args: IndexedScryptoValue,
}

impl<I: WasmInstance> Executor for ScryptoExecutor<I> {
    type Output = Vec<u8>;

    fn execute<Y>(self, api: &mut Y) -> Result<(Vec<u8>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        ScryptoExecutorToParsed {
            instance: self.instance,
            args: self.args,
        }
        .execute(api)
        .map(|(indexed, update)| (indexed.raw, update))
    }
}

pub struct ScryptoInterpreter<W: WasmEngine> {
    pub wasm_engine: W,
    /// WASM Instrumenter
    pub wasm_instrumenter: WasmInstrumenter,
    /// WASM metering config
    pub wasm_metering_config: WasmMeteringConfig,
}

impl<W: WasmEngine> ScryptoInterpreter<W> {
    pub fn create_executor(
        &self,
        code: &[u8],
        args: IndexedScryptoValue,
    ) -> ScryptoExecutor<W::WasmInstance> {
        let instrumented_code = self
            .wasm_instrumenter
            .instrument(code, &self.wasm_metering_config);
        let instance = self.wasm_engine.instantiate(&instrumented_code);
        ScryptoExecutor {
            instance,
            args: args,
        }
    }

    pub fn create_executor_to_parsed(
        &self,
        code: &[u8],
        args: IndexedScryptoValue,
    ) -> ScryptoExecutorToParsed<W::WasmInstance> {
        let instrumented_code = self
            .wasm_instrumenter
            .instrument(code, &self.wasm_metering_config);
        let instance = self.wasm_engine.instantiate(&instrumented_code);
        ScryptoExecutorToParsed {
            instance,
            args: args,
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
