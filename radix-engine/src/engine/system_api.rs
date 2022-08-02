use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::core::Receiver;
use scrypto::engine::types::*;
use scrypto::resource::AccessRule;
use scrypto::values::*;

use crate::engine::values::*;
use crate::engine::*;
use crate::fee::*;
use crate::wasm::*;

use super::call_frame::RENodeRef;

pub trait SystemApi<'p, 's, W, I, C>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    C: CostUnitCounter,
{
    fn cost_unit_counter(&mut self) -> &mut C;

    fn fee_table(&self) -> &FeeTable;

    fn invoke_function(
        &mut self,
        receiver: Receiver,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError>;

    fn globalize_node(&mut self, value_id: &RENodeId) -> Result<(), CostUnitCounterError>;

    fn borrow_node(
        &mut self,
        value_id: &RENodeId,
    ) -> Result<RENodeRef<'_, 's>, CostUnitCounterError>;

    fn borrow_node_mut(
        &mut self,
        value_id: &RENodeId,
    ) -> Result<NativeRENodeRef, CostUnitCounterError>;

    fn return_node_mut(&mut self, val_ref: NativeRENodeRef) -> Result<(), CostUnitCounterError>;

    fn drop_node(&mut self, value_id: &RENodeId) -> Result<HeapRootRENode, CostUnitCounterError>;
    fn create_node<V: Into<RENodeByComplexity>>(&mut self, v: V) -> Result<RENodeId, RuntimeError>;

    fn read_substate(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError>;
    fn write_substate(
        &mut self,
        substate_id: SubstateId,
        value: ScryptoValue,
    ) -> Result<(), RuntimeError>;
    fn take_substate(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError>;

    fn transaction_hash(&mut self) -> Result<Hash, CostUnitCounterError>;

    fn generate_uuid(&mut self) -> Result<u128, CostUnitCounterError>;

    fn emit_log(&mut self, level: Level, message: String) -> Result<(), CostUnitCounterError>;

    fn check_access_rule(
        &mut self,
        access_rule: AccessRule,
        proof_ids: Vec<ProofId>,
    ) -> Result<bool, RuntimeError>;
}
