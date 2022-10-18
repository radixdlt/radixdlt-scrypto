use crate::engine::*;
use crate::fee::*;
use crate::model::{
    Component, ComponentInfoSubstate, ComponentStateSubstate, InvokeError, KeyValueStore,
    RuntimeSubstate,
};
use crate::types::*;
use crate::wasm::*;
use scrypto::core::FnIdent;

use super::KernelError;

/// A glue between system api (call frame and track abstraction) and WASM.
///
/// Execution is free from a costing perspective, as we assume
/// the system api will bill properly.
pub struct RadixEngineWasmRuntime<'y, 's, Y, R>
where
    Y: SystemApi<'s, R>,
    R: FeeReserve,
{
    actor: ScryptoActor,
    system_api: &'y mut Y,
    phantom1: PhantomData<R>,
    phantom2: PhantomData<&'s ()>,
}

impl<'y, 's, Y, R> RadixEngineWasmRuntime<'y, 's, Y, R>
where
    Y: SystemApi<'s, R>,
    R: FeeReserve,
{
    // TODO: expose API for reading blobs

    // TODO: do we want to allow dynamic creation of blobs?

    // TODO: do we check existence of blobs when being passed as arguments/return?

    pub fn new(actor: ScryptoActor, system_api: &'y mut Y) -> Self {
        RadixEngineWasmRuntime {
            actor,
            system_api,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    // FIXME: limit access to the API
    fn handle_invoke(
        &mut self,
        fn_ident: FnIdent,
        input: Vec<u8>,
    ) -> Result<ScryptoValue, RuntimeError> {
        let call_data = ScryptoValue::from_slice(&input)
            .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
        self.system_api.invoke(fn_ident, call_data)
    }

    fn handle_node_create(
        &mut self,
        scrypto_node: ScryptoRENode,
    ) -> Result<ScryptoValue, RuntimeError> {
        let node = match scrypto_node {
            ScryptoRENode::Component(package_address, blueprint_name, state) => {
                // TODO: Move these two checks into kernel
                if !blueprint_name.eq(self.actor.blueprint_name()) {
                    return Err(RuntimeError::KernelError(
                        KernelError::RENodeCreateInvalidPermission,
                    ));
                }
                if !package_address.eq(self.actor.package_address()) {
                    return Err(RuntimeError::KernelError(
                        KernelError::RENodeCreateInvalidPermission,
                    ));
                }

                // TODO: Check state against blueprint schema

                // Create component
                HeapRENode::Component(Component {
                    info: ComponentInfoSubstate::new(package_address, blueprint_name, Vec::new()),
                    state: Some(ComponentStateSubstate::new(state)),
                })
            }
            ScryptoRENode::KeyValueStore => HeapRENode::KeyValueStore(KeyValueStore::new()),
        };

        let id = self.system_api.create_node(node)?;
        Ok(ScryptoValue::from_typed(&id))
    }

    fn handle_get_visible_node_ids(&mut self) -> Result<ScryptoValue, RuntimeError> {
        let node_ids = self.system_api.get_visible_node_ids()?;
        Ok(ScryptoValue::from_typed(&node_ids))
    }

    fn handle_node_globalize(&mut self, node_id: RENodeId) -> Result<ScryptoValue, RuntimeError> {
        let global_address = self.system_api.node_globalize(node_id)?;
        Ok(ScryptoValue::from_typed(&global_address))
    }

    fn handle_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        mutable: bool,
    ) -> Result<ScryptoValue, RuntimeError> {
        let flags = if mutable {
            LockFlags::MUTABLE
        } else {
            // TODO: Do we want to expose full flag functionality to Scrypto?
            LockFlags::read_only()
        };

        self.system_api
            .lock_substate(node_id, offset, flags)
            .map(|handle| ScryptoValue::from_typed(&handle))
    }

    fn handle_read(&mut self, lock_handle: LockHandle) -> Result<ScryptoValue, RuntimeError> {
        self.system_api
            .get_ref(lock_handle)
            .map(|substate_ref| substate_ref.to_scrypto_value())
    }

    fn handle_write(
        &mut self,
        lock_handle: LockHandle,
        buffer: Vec<u8>,
    ) -> Result<ScryptoValue, RuntimeError> {
        let mut substate_mut = self.system_api.get_ref_mut(lock_handle)?;
        let substate = RuntimeSubstate::decode_from_buffer(substate_mut.offset(), &buffer)?;
        let mut raw_mut = substate_mut.get_raw_mut();

        match substate {
            RuntimeSubstate::ComponentState(next) => *raw_mut.component_state() = next,
            RuntimeSubstate::KeyValueStoreEntry(next) => {
                *raw_mut.kv_store_entry() = next;
            }
            RuntimeSubstate::NonFungible(next) => {
                *raw_mut.non_fungible() = next;
            }
            _ => return Err(RuntimeError::KernelError(KernelError::InvalidOverwrite)),
        }

        substate_mut.flush()?;

        Ok(ScryptoValue::unit())
    }

    fn handle_drop_lock(&mut self, lock_handle: LockHandle) -> Result<ScryptoValue, RuntimeError> {
        self.system_api
            .drop_lock(lock_handle)
            .map(|unit| ScryptoValue::from_typed(&unit))
    }

    fn handle_get_actor(&mut self) -> Result<ScryptoActor, RuntimeError> {
        return Ok(self.actor.clone());
    }

    fn handle_generate_uuid(&mut self) -> Result<u128, RuntimeError> {
        self.system_api.generate_uuid()
    }

    fn handle_emit_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        self.system_api.emit_log(level, message)
    }
}

fn encode<T: Encode>(output: T) -> ScryptoValue {
    ScryptoValue::from_typed(&output)
}

impl<'y, 's, Y, R> WasmRuntime for RadixEngineWasmRuntime<'y, 's, Y, R>
where
    Y: SystemApi<'s, R>,
    R: FeeReserve,
{
    fn main(&mut self, input: ScryptoValue) -> Result<ScryptoValue, InvokeError<WasmError>> {
        let input: RadixEngineInput = scrypto_decode(&input.raw)
            .map_err(|_| InvokeError::Error(WasmError::InvalidRadixEngineInput))?;
        let rtn = match input {
            RadixEngineInput::Invoke(fn_ident, input_bytes) => {
                self.handle_invoke(fn_ident, input_bytes)?
            }
            RadixEngineInput::RENodeGlobalize(node_id) => self.handle_node_globalize(node_id)?,
            RadixEngineInput::RENodeCreate(node) => self.handle_node_create(node)?,
            RadixEngineInput::GetVisibleNodeIds() => self.handle_get_visible_node_ids()?,

            RadixEngineInput::LockSubstate(node_id, offset, mutable) => {
                self.handle_lock_substate(node_id, offset, mutable)?
            }
            RadixEngineInput::Read(lock_handle) => self.handle_read(lock_handle)?,
            RadixEngineInput::Write(lock_handle, value) => self.handle_write(lock_handle, value)?,
            RadixEngineInput::DropLock(lock_handle) => self.handle_drop_lock(lock_handle)?,

            RadixEngineInput::GetActor() => self.handle_get_actor().map(encode)?,
            RadixEngineInput::GenerateUuid() => self.handle_generate_uuid().map(encode)?,
            RadixEngineInput::EmitLog(level, message) => {
                self.handle_emit_log(level, message).map(encode)?
            }
        };

        Ok(rtn)
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmError>> {
        self.system_api
            .consume_cost_units(n)
            .map_err(InvokeError::downstream)
    }
}

/// A `Nop` runtime accepts any external function calls by doing nothing and returning void.
pub struct NopWasmRuntime {
    fee_reserve: SystemLoanFeeReserve,
}

impl NopWasmRuntime {
    pub fn new(fee_reserve: SystemLoanFeeReserve) -> Self {
        Self { fee_reserve }
    }
}

impl WasmRuntime for NopWasmRuntime {
    fn main(&mut self, _input: ScryptoValue) -> Result<ScryptoValue, InvokeError<WasmError>> {
        Ok(ScryptoValue::unit())
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmError>> {
        self.fee_reserve
            .consume(n, "run_wasm", false)
            .map_err(|e| InvokeError::Error(WasmError::CostingError(e)))
    }
}
