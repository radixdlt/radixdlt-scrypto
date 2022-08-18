use super::InstructionCostRules;
use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct WasmMeteringParams {
    instruction_cost_rules: InstructionCostRules,
    max_stack_size: u32,
}

impl WasmMeteringParams {
    pub fn new(instruction_cost_rules: InstructionCostRules, max_stack_size: u32) -> Self {
        Self {
            instruction_cost_rules,
            max_stack_size,
        }
    }

    /// Wasm fee table is statically applied to the wasm code.
    /// This identifier helps decide whether or not re-instrumentation is required.
    pub fn identifier(&self) -> Hash {
        let encoded = scrypto_encode(self);
        hash(encoded)
    }

    pub fn instruction_cost_rules(&self) -> &InstructionCostRules {
        &self.instruction_cost_rules
    }

    pub fn max_stack_size(&self) -> u32 {
        self.max_stack_size
    }
}
