use crate::engine::*;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance, WasmInstrumenter, WasmMeteringConfig, WasmRuntime};
use radix_engine_interface::api::api::{EngineApi, SysInvokableNative, SysInvokableNativeMethod};
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::data::{match_schema_with_value, IndexedScryptoValue};

pub struct ScryptoExecutor<I: WasmInstance> {
    instance: I,
    args: IndexedScryptoValue,
}

impl<I: WasmInstance> Executor for ScryptoExecutor<I> {
    type Output = IndexedScryptoValue;

    fn args(&self) -> &IndexedScryptoValue {
        &self.args
    }

    fn execute<Y>(
        mut self,
        system_api: &mut Y,
    ) -> Result<(IndexedScryptoValue, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>
            + SysInvokableNativeMethod<RuntimeError>,
    {
        let (export_name, return_type) = match system_api.get_actor() {
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
            let mut runtime: Box<dyn WasmRuntime> =
                Box::new(RadixEngineWasmRuntime::new(system_api));
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm::DefaultWasmEngine;

    const _: () = {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        // RFC 2056
        fn assert_all() {
            assert_send::<ScryptoInterpreter<DefaultWasmEngine>>();
            // TODO: make sure engine is indeed multi-thread safe!
            #[cfg(not(feature = "alloc"))]
            assert_sync::<ScryptoInterpreter<DefaultWasmEngine>>();
        }
    };
}
