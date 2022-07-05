use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::core::SNodeRef;
use scrypto::engine::types::*;
use scrypto::resource::AccessRule;
use scrypto::values::*;

use crate::engine::call_frame::{REValueRef, SubstateAddress};
use crate::engine::values::*;
use crate::engine::*;
use crate::fee::*;
use crate::ledger::ReadableSubstateStore;
use crate::wasm::*;

pub trait SystemApi<'p, 's, W, I, S>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    S: ReadableSubstateStore,
{
    fn cost_unit_counter(&mut self) -> &mut CostUnitCounter;

    fn fee_table(&self) -> &FeeTable;

    fn invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError>;

    fn globalize_value(&mut self, value_id: &ValueId);
    fn borrow_value(&self, value_id: &ValueId) -> REValueRef<'_, 's, S>;
    fn borrow_value_mut(&mut self, value_id: &ValueId) -> RENativeValueRef<'p>;
    fn return_value_mut(&mut self, val_ref: RENativeValueRef<'p>);
    fn drop_value(&mut self, value_id: &ValueId) -> REValue;
    fn create_value<V: Into<REValueByComplexity>>(&mut self, v: V)
        -> Result<ValueId, RuntimeError>;
    fn read_value_data(&mut self, address: SubstateAddress) -> Result<ScryptoValue, RuntimeError>;
    fn write_value_data(
        &mut self,
        address: SubstateAddress,
        value: ScryptoValue,
    ) -> Result<(), RuntimeError>;
    fn remove_value_data(&mut self, address: SubstateAddress)
        -> Result<ScryptoValue, RuntimeError>;

    fn get_epoch(&mut self) -> u64;

    fn get_transaction_hash(&mut self) -> Hash;

    fn generate_uuid(&mut self) -> u128;

    fn user_log(&mut self, level: Level, message: String);

    fn check_access_rule(
        &mut self,
        access_rule: AccessRule,
        proof_ids: Vec<ProofId>,
    ) -> Result<bool, RuntimeError>;
}
