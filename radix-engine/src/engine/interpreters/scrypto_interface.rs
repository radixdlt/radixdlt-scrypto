use sbor::Encode;
use scrypto::core::ScryptoActor;
use scrypto::engine::api::RadixEngineInput;
use scrypto::engine::types::{Receiver, RENodeId, ScryptoRENode};
use scrypto::values::ScryptoValue;
use crate::engine::{Kernel, KernelError, LockFlags, REActor, RENode, ResolvedFunction, ResolvedMethod, ResolvedReceiver, RuntimeError, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{ComponentInfoSubstate, ComponentStateSubstate, GlobalAddressSubstate, KeyValueStore, RuntimeSubstate};
use crate::types::{NativeInvocation, ScryptoInvocation};
use crate::wasm::{WasmEngine, WasmInstance};

pub trait ScryptoSyscalls<E> {
    fn sys_call(&mut self, input: RadixEngineInput) -> Result<ScryptoValue, E>;
}

fn encode<T: Encode>(output: T) -> ScryptoValue {
    ScryptoValue::from_typed(&output)
}

impl<'g, 's, W, I, R> ScryptoSyscalls<RuntimeError> for Kernel<'g, 's, W, I, R>
where W: WasmEngine<I>,
I: WasmInstance,
R: FeeReserve,{
    fn sys_call(&mut self, input: RadixEngineInput) -> Result<ScryptoValue, RuntimeError> {
        match input {
            RadixEngineInput::InvokeScryptoFunction(fn_ident, args) => {
                let args = ScryptoValue::from_slice(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                self.invoke_scrypto(ScryptoInvocation::Function(fn_ident, args))
            }
            RadixEngineInput::InvokeScryptoMethod(method_ident, args) => {
                let args = ScryptoValue::from_slice(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                self.invoke_scrypto(ScryptoInvocation::Method(method_ident, args))
            }
            RadixEngineInput::InvokeNativeFunction(native_function, args) => {
                let args = ScryptoValue::from_slice(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;

                self.invoke_native(NativeInvocation::Function(native_function, args))
            }
            RadixEngineInput::InvokeNativeMethod(native_method, receiver, args) => {
                let args = ScryptoValue::from_slice(&args)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;

                self.invoke_native(NativeInvocation::Method(native_method, receiver, args))
            }
            RadixEngineInput::CreateNode(node) => {
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

                let id = self.create_node(node)?;
                Ok(ScryptoValue::from_typed(&id))
            },
            RadixEngineInput::GetVisibleNodeIds() => {
                let node_ids = self.get_visible_node_ids()?;
                Ok(ScryptoValue::from_typed(&node_ids))
            },

            RadixEngineInput::LockSubstate(node_id, offset, mutable) => {
                let flags = if mutable {
                    LockFlags::MUTABLE
                } else {
                    // TODO: Do we want to expose full flag functionality to Scrypto?
                    LockFlags::read_only()
                };

                let handle = self.lock_substate(node_id, offset.clone(), flags)?;

                Ok(ScryptoValue::from_typed(&handle))
            }
            RadixEngineInput::Read(lock_handle) => {
                self.get_ref(lock_handle)
                    .map(|substate_ref| substate_ref.to_scrypto_value())
            },
            RadixEngineInput::Write(lock_handle, buffer) => {
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

                Ok(ScryptoValue::unit())
            },
            RadixEngineInput::DropLock(lock_handle) => {
                self.drop_lock(lock_handle)
                    .map(|unit| ScryptoValue::from_typed(&unit))
            },

            RadixEngineInput::GetActor() => {
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

                return Ok(ScryptoValue::from_typed(&actor));
            },
            RadixEngineInput::GenerateUuid() => {
                self.generate_uuid().map(encode)
            },
            RadixEngineInput::EmitLog(level, message) => {
                self.emit_log(level, message).map(encode)
            }
        }
    }
}