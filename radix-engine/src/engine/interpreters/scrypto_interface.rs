use sbor::Decode;
use scrypto::buffer::scrypto_decode;
use scrypto::core::ScryptoActor;
use scrypto::engine::types::{Level, LockHandle, NativeFunction, NativeMethod, Receiver, RENodeId, ScryptoFunctionIdent, ScryptoMethodIdent, ScryptoRENode, SubstateOffset};
use scrypto::engine::utils::ScryptoSyscalls;
use scrypto::values::ScryptoValue;
use crate::engine::{Kernel, KernelError, LockFlags, REActor, RENode, ResolvedFunction, ResolvedMethod, ResolvedReceiver, RuntimeError, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{ComponentInfoSubstate, ComponentStateSubstate, GlobalAddressSubstate, KeyValueStore, RuntimeSubstate};
use crate::types::{NativeInvocation, ScryptoInvocation};
use crate::wasm::{WasmEngine, WasmInstance};



impl<'g, 's, W, I, R> ScryptoSyscalls<RuntimeError> for Kernel<'g, 's, W, I, R>
where W: WasmEngine<I>,
I: WasmInstance,
R: FeeReserve,{
    fn sys_invoke_scrypto_function<V: Decode>(&mut self, fn_ident: ScryptoFunctionIdent, args: Vec<u8>) -> Result<V, RuntimeError> {
        let args = ScryptoValue::from_slice(&args)
            .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
        self.invoke_scrypto(ScryptoInvocation::Function(fn_ident, args))
            .map(|value| scrypto_decode(&value.raw).unwrap())
    }

    fn sys_invoke_scrypto_method<V: Decode>(&mut self, method_ident: ScryptoMethodIdent, args: Vec<u8>) -> Result<V, RuntimeError> {
        let args = ScryptoValue::from_slice(&args)
            .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
        self.invoke_scrypto(ScryptoInvocation::Method(method_ident, args))
            .map(|value| scrypto_decode(&value.raw).unwrap())
    }

    fn sys_invoke_native_function<V: Decode>(&mut self, native_function: NativeFunction, args: Vec<u8>) -> Result<V, RuntimeError> {
        let args = ScryptoValue::from_slice(&args)
            .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;

        self.invoke_native(NativeInvocation::Function(native_function, args))
            .map(|value| scrypto_decode(&value.raw).unwrap())
    }

    fn sys_invoke_native_method<V: Decode>(&mut self, native_method: NativeMethod, receiver: Receiver, args: Vec<u8>) -> Result<V, RuntimeError> {
        let args = ScryptoValue::from_slice(&args)
            .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;

        self.invoke_native(NativeInvocation::Method(native_method, receiver, args))
            .map(|value| scrypto_decode(&value.raw).unwrap())
    }

    fn sys_create_node(&mut self, node: ScryptoRENode) -> Result<RENodeId, RuntimeError> {
        let node = match node {
            ScryptoRENode::GlobalComponent(component_id) => RENode::Global(
                GlobalAddressSubstate::Component(scrypto::component::Component(component_id)),
            ),
            ScryptoRENode::Component(package_address, blueprint_name, state) => {
                // Create component
                RENode::Component(
                    ComponentInfoSubstate::new(package_address, blueprint_name, Vec::new()),
                    ComponentStateSubstate::new(state),
                )
            }
            ScryptoRENode::KeyValueStore => RENode::KeyValueStore(KeyValueStore::new()),
        };

        self.create_node(node)
    }

    fn sys_get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, RuntimeError> {
        self.get_visible_node_ids()
    }

    fn sys_lock_substate(&mut self, node_id: RENodeId, offset: SubstateOffset, mutable: bool) -> Result<LockHandle, RuntimeError> {
        let flags = if mutable {
            LockFlags::MUTABLE
        } else {
            // TODO: Do we want to expose full flag functionality to Scrypto?
            LockFlags::read_only()
        };

        self.lock_substate(node_id, offset, flags)
    }

    fn sys_read<V: Decode>(&mut self, lock_handle: LockHandle) -> Result<V, RuntimeError> {
        self.get_ref(lock_handle)
            .map(|substate_ref| substate_ref.to_scrypto_value())
            .map(|value| scrypto_decode(&value.raw).unwrap())
    }

    fn sys_write(&mut self, lock_handle: LockHandle, buffer: Vec<u8>) -> Result<(), RuntimeError> {
        let offset = self.get_lock_info(lock_handle)?.offset;
        let substate = RuntimeSubstate::decode_from_buffer(&offset, &buffer)?;
        let mut substate_mut = self.get_ref_mut(lock_handle)?;

        match substate {
            RuntimeSubstate::ComponentState(next) => *substate_mut.component_state() = next,
            RuntimeSubstate::KeyValueStoreEntry(next) => {
                *substate_mut.kv_store_entry() = next;
            }
            RuntimeSubstate::NonFungible(next) => {
                *substate_mut.non_fungible() = next;
            }
            _ => return Err(RuntimeError::KernelError(KernelError::InvalidOverwrite)),
        }

        Ok(())
    }

    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        self.drop_lock(lock_handle)
    }

    fn sys_get_actor(&mut self) -> Result<ScryptoActor, RuntimeError> {
        let actor = match self.get_actor() {
            REActor::Method(
                ResolvedMethod::Scrypto {
                    package_address,
                    blueprint_name,
                    ..
                },
                ResolvedReceiver {
                    receiver: Receiver::Ref(RENodeId::Component(component_id)),
                    ..
                },
            ) => ScryptoActor::Component(
                *component_id,
                package_address.clone(),
                blueprint_name.clone(),
            ),
            REActor::Function(ResolvedFunction::Scrypto {
                                  package_address,
                                  blueprint_name,
                                  ..
                              }) => ScryptoActor::blueprint(*package_address, blueprint_name.clone()),

            _ => panic!("Should not get here."),
        };

        Ok(actor)
    }

    fn sys_generate_uuid(&mut self) -> Result<u128, RuntimeError> {
        self.generate_uuid()
    }

    fn sys_emit_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        self.emit_log(level, message)
    }
}