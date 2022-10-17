use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::PackageSubstate;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance, WasmInstrumenter, WasmMeteringParams};

pub struct ScryptoInterpreter<I: WasmInstance, W: WasmEngine<I>> {
    pub wasm_engine: W,
    /// WASM Instrumenter
    pub wasm_instrumenter: WasmInstrumenter,
    /// WASM metering params
    pub wasm_metering_params: WasmMeteringParams,
    pub phantom: PhantomData<I>,
}

impl<I: WasmInstance, W: WasmEngine<I>> ScryptoInterpreter<I, W> {
    pub fn instance(&mut self, package: PackageSubstate) -> I {
        let instrumented_code = self
            .wasm_instrumenter
            .instrument(package.code(), &self.wasm_metering_params);
        self.wasm_engine.instantiate(instrumented_code)
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
