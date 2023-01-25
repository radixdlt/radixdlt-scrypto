use crate::engine::*;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance, WasmInstrumenter, WasmMeteringConfig, WasmRuntime};
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::{ActorApi, ComponentApi, EngineApi, InvokableModel};
use radix_engine_interface::data::{match_schema_with_value, ScryptoValue};

pub struct ScryptoExecutor {
    pub package_address: PackageAddress,
    pub export_name: String,
    pub component_id: Option<ComponentId>,
    pub args: ScryptoValue,
}

impl Executor for ScryptoExecutor {
    type Output = ScryptoValue;

    fn execute<Y, W>(self, api: &mut Y) -> Result<(ScryptoValue, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + EngineApi<RuntimeError>
            + InvokableModel<RuntimeError>
            + ActorApi<RuntimeError>
            + ComponentApi<RuntimeError>
            + VmApi<W>,
        W: WasmEngine,
    {
        let package = {
            let handle = api.lock_substate(
                RENodeId::Global(GlobalAddress::Package(self.package_address)),
                SubstateOffset::Package(PackageOffset::Info),
                LockFlags::read_only(),
            )?;
            let substate_ref = api.get_ref(handle)?;
            let package = substate_ref.package_info().clone(); // TODO: Remove clone()
            api.drop_lock(handle)?;

            package
        };

        let fn_abi = package
            .fn_abi(&self.export_name)
            .expect("TODO: Remove this expect");
        let rtn_type = fn_abi.output.clone();

        // Emit event
        api.on_wasm_instantiation(package.code())?;
        let mut instance = api
            .vm()
            .create_instance(self.package_address, &package.code);

        let output = {
            let mut runtime: Box<dyn WasmRuntime> = Box::new(RadixEngineWasmRuntime::new(api));

            let mut input = Vec::new();
            if let Some(component_id) = self.component_id {
                input.push(
                    runtime
                        .allocate_buffer(
                            scrypto_encode(&component_id).expect("Failed to encode component id"),
                        )
                        .expect("Failed to allocate buffer"),
                );
            }
            input.push(
                runtime
                    .allocate_buffer(scrypto_encode(&self.args).expect("Failed to encode args"))
                    .expect("Failed to allocate buffer"),
            );

            instance.invoke_export(&self.export_name, input, &mut runtime)?
        };
        let output = IndexedScryptoValue::from_vec(output).map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::InvalidScryptoReturn(e))
        })?;

        let rtn = if !match_schema_with_value(&rtn_type, output.as_value()) {
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
            Ok((output.into(), update))
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
    pub fn create_instance(&self, package_address: PackageAddress, code: &[u8]) -> W::WasmInstance {
        let instrumented_code =
            self.wasm_instrumenter
                .instrument(package_address, code, self.wasm_metering_config);
        self.wasm_engine.instantiate(&instrumented_code)
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
