use crate::engine::*;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance, WasmInstrumenter, WasmMeteringConfig, WasmRuntime};

pub struct ScryptoExecutor<I: WasmInstance> {
    instance: I,
    args: ScryptoValue,
}

impl<I: WasmInstance> Executor for ScryptoExecutor<I> {
    type Output = ScryptoValue;

    fn args(&self) -> &ScryptoValue {
        &self.args
    }

    fn execute<'a, Y>(
        mut self,
        system_api: &mut Y,
    ) -> Result<(ScryptoValue, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + Invokable<ScryptoInvocation> + InvokableNative<'a>,
    {
        let (export_name, return_type, scrypto_actor) = match system_api.get_actor() {
            REActor::Method(
                ResolvedMethod::Scrypto {
                    package_address,
                    blueprint_name,
                    export_name,
                    return_type,
                    ..
                },
                ResolvedReceiver {
                    receiver: RENodeId::Component(component_id),
                    ..
                },
            ) => (
                export_name.to_string(),
                return_type.clone(),
                ScryptoActor::Component(
                    *component_id,
                    package_address.clone(),
                    blueprint_name.clone(),
                ),
            ),
            REActor::Function(ResolvedFunction::Scrypto {
                package_address,
                blueprint_name,
                export_name,
                return_type,
                ..
            }) => (
                export_name.to_string(),
                return_type.clone(),
                ScryptoActor::blueprint(*package_address, blueprint_name.clone()),
            ),

            _ => panic!("Should not get here."),
        };

        let output = {
            let mut runtime: Box<dyn WasmRuntime> =
                Box::new(RadixEngineWasmRuntime::new(scrypto_actor, system_api));
            self.instance
                .invoke_export(&export_name, &self.args, &mut runtime)
                .map_err(|e| match e {
                    InvokeError::Error(e) => RuntimeError::KernelError(KernelError::WasmError(e)),
                    InvokeError::Downstream(runtime_error) => runtime_error,
                })?
        };

        let rtn = if !return_type.matches(&output.dom) {
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

pub struct ScryptoInterpreter<I: WasmInstance, W: WasmEngine<I>> {
    pub wasm_engine: W,
    /// WASM Instrumenter
    pub wasm_instrumenter: WasmInstrumenter,
    /// WASM metering config
    pub wasm_metering_config: WasmMeteringConfig,
    pub phantom: PhantomData<I>,
}

impl<I: WasmInstance, W: WasmEngine<I>> ScryptoInterpreter<I, W> {
    pub fn create_executor(&self, code: &[u8], args: ScryptoValue) -> ScryptoExecutor<I> {
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
