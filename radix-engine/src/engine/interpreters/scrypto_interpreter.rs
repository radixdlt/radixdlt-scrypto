use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::PackageSubstate;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance, WasmInstrumenter, WasmMeteringParams, WasmRuntime};

pub struct ScryptoExecutor<I: WasmInstance> {
    instance: I,
}

impl<I: WasmInstance> ScryptoExecutor<I> {
    pub fn run<'s, Y, R>(
        &mut self,
        input: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, RuntimeError>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let (ident, package_address, blueprint_name, export_name, return_type, scrypto_actor) =
            match system_api.get_actor() {
                REActor::Method(
                    ResolvedMethod::Scrypto {
                        package_address,
                        blueprint_name,
                        ident,
                        export_name,
                        return_type,
                    },
                    ResolvedReceiver {
                        receiver: Receiver::Ref(RENodeId::Component(component_id)),
                        ..
                    },
                ) => (
                    ident.to_string(),
                    *package_address,
                    blueprint_name.to_string(),
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
                    ident,
                    export_name,
                    return_type,
                }) => (
                    ident.to_string(),
                    *package_address,
                    blueprint_name.to_string(),
                    export_name.to_string(),
                    return_type.clone(),
                    ScryptoActor::blueprint(*package_address, blueprint_name.clone()),
                ),

                _ => panic!("Should not get here."),
            };

        let output = {
            system_api.execute_in_mode(ExecutionMode::Application, |system_api| {
                let mut runtime: Box<dyn WasmRuntime> =
                    Box::new(RadixEngineWasmRuntime::new(scrypto_actor, system_api));
                self.instance
                    .invoke_export(&export_name, &input, &mut runtime)
                    .map_err(|e| match e {
                        InvokeError::Error(e) => {
                            RuntimeError::KernelError(KernelError::WasmError(e))
                        }
                        InvokeError::Downstream(runtime_error) => runtime_error,
                    })
            })?
        };

        let rtn = if !return_type.matches(&output.dom) {
            Err(RuntimeError::KernelError(
                KernelError::InvalidScryptoFnOutput(ScryptoFnIdent::Function(
                    ScryptoFunctionIdent {
                        package_address,
                        blueprint_name,
                        function_name: ident,
                    },
                )),
            ))
        } else {
            Ok(output)
        };

        rtn
    }
}

pub struct ScryptoInterpreter<I: WasmInstance, W: WasmEngine<I>> {
    pub wasm_engine: W,
    /// WASM Instrumenter
    pub wasm_instrumenter: WasmInstrumenter,
    /// WASM metering params
    pub wasm_metering_params: WasmMeteringParams,
    pub phantom: PhantomData<I>,
}

impl<I: WasmInstance, W: WasmEngine<I>> ScryptoInterpreter<I, W> {
    pub fn create_executor(&mut self, package: PackageSubstate) -> ScryptoExecutor<I> {
        let instrumented_code = self
            .wasm_instrumenter
            .instrument(package.code(), &self.wasm_metering_params);
        let instance = self.wasm_engine.instantiate(instrumented_code);
        ScryptoExecutor { instance }
    }
}
