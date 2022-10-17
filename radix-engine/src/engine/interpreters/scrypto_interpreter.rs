use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::PackageSubstate;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance, WasmInstrumenter, WasmMeteringParams, WasmRuntime};

pub struct ScryptoExecutor<I: WasmInstance> {
    instance: I,
}

impl<I: WasmInstance> ScryptoExecutor<I> {
    pub fn run<'s, W, Y, R>(
        &mut self,
        input: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, RuntimeError>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        R: FeeReserve,
    {
        let (ident, package_address, blueprint_name, export_name, scrypto_actor) =
            match system_api.get_actor() {
                REActor::Method(ResolvedReceiverMethod {
                    receiver:
                        ResolvedReceiver {
                            receiver: Receiver::Ref(RENodeId::Component(component_id)),
                            ..
                        },
                    method:
                        ResolvedMethod::Scrypto {
                            package_address,
                            blueprint_name,
                            ident,
                            export_name,
                        },
                }) => (
                    ident.to_string(),
                    *package_address,
                    blueprint_name.to_string(),
                    export_name.to_string(),
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
                }) => (
                    ident.to_string(),
                    *package_address,
                    blueprint_name.to_string(),
                    export_name.to_string(),
                    ScryptoActor::blueprint(*package_address, blueprint_name.clone()),
                ),

                _ => panic!("Should not get here."),
            };

        let output = {
            system_api.execute_in_mode(KernelActor::Application, |system_api| {
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

        // TODO: Remove reloading of package rules
        let package_id = RENodeId::Global(GlobalAddress::Package(package_address));
        let package_offset = SubstateOffset::Package(PackageOffset::Package);
        let package_handle =
            system_api.lock_substate(package_id, package_offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(package_handle)?;
        let package = substate_ref.package();
        let blueprint_abi = package
            .blueprint_abi(&blueprint_name)
            .expect("Blueprint not found"); // TODO: assumption will break if auth module is optional
        let fn_abi = blueprint_abi
            .get_fn_abi(&ident)
            .expect("Function not found");
        let rtn = if !fn_abi.output.matches(&output.dom) {
            Err(RuntimeError::KernelError(KernelError::InvalidFnOutput {
                fn_identifier: FunctionIdent::Scrypto {
                    package_address,
                    blueprint_name,
                    ident,
                },
            }))
        } else {
            Ok(output)
        };

        system_api.drop_lock(package_handle)?;

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

    pub fn load_scrypto_actor<'s, Y, R>(
        ident: ScryptoFnIdent,
        input: &ScryptoValue,
        system_api: &mut Y,
    ) -> Result<(REActor, RENodeId), InvokeError<ScryptoActorError>>
    where
        Y: SystemApi<'s, W, I, R>,
        R: FeeReserve,
    {
        let (receiver, package_address, blueprint_name, ident) = match ident {
            ScryptoFnIdent::Method(receiver, ident) => {
                if !matches!(
                    receiver,
                    ResolvedReceiver {
                        receiver: Receiver::Ref(RENodeId::Component(..)),
                        ..
                    }
                ) {
                    return Err(InvokeError::Error(ScryptoActorError::InvalidReceiver));
                }

                let node_id = receiver.receiver().node_id();
                let offset = SubstateOffset::Component(ComponentOffset::Info);
                let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
                let substate_ref = system_api.get_ref(handle)?;
                let info = substate_ref.component_info();
                let rtn = (
                    Some(receiver),
                    info.package_address.clone(),
                    info.blueprint_name.clone(),
                    ident,
                );
                system_api.drop_lock(handle)?;

                rtn
            }
            ScryptoFnIdent::Function(package_address, blueprint_name, ident) => {
                (None, package_address, blueprint_name, ident)
            }
        };

        let package_node_id = RENodeId::Global(GlobalAddress::Package(package_address));
        let offset = SubstateOffset::Package(PackageOffset::Package);
        let handle = system_api.lock_substate(package_node_id, offset, LockFlags::empty())?;
        let substate_ref = system_api.get_ref(handle)?;
        let package = substate_ref.package();
        let abi = package
            .blueprint_abi(&blueprint_name)
            .ok_or(InvokeError::Error(ScryptoActorError::BlueprintNotFound))?;

        let fn_abi = abi
            .get_fn_abi(&ident)
            .ok_or(InvokeError::Error(ScryptoActorError::IdentNotFound))?;

        if fn_abi.mutability.is_some() != receiver.is_some() {
            return Err(InvokeError::Error(ScryptoActorError::InvalidReceiver));
        }

        if !fn_abi.input.matches(&input.dom) {
            return Err(InvokeError::Error(ScryptoActorError::InvalidInput));
        }

        let export_name = fn_abi.export_name.to_string();
        system_api.drop_lock(handle)?;

        let actor = if let Some(receiver) = receiver {
            REActor::Method(ResolvedReceiverMethod {
                receiver,
                method: ResolvedMethod::Scrypto {
                    package_address,
                    blueprint_name: blueprint_name.clone(),
                    ident: ident.to_string(),
                    export_name,
                },
            })
        } else {
            REActor::Function(ResolvedFunction::Scrypto {
                package_address,
                blueprint_name: blueprint_name.clone(),
                ident: ident.clone(),
                export_name,
            })
        };

        // TODO: Make package node visible in a different way
        Ok((actor, package_node_id))
    }
}
