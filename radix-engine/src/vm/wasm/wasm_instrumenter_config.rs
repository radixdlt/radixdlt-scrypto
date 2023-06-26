use super::InstructionCostRules;
use crate::types::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, ScryptoSbor)]
pub struct WasmInstrumenterConfig {
    rules: InstructionCostRules,
    max_stack_size: u32,
}

impl WasmInstrumenterConfig {
    pub fn v1() -> Self {
        todo!()
    }

    pub fn instruction_cost_rules(&self) -> &InstructionCostRules {
        &self.rules
    }

    pub fn max_stack_size(&self) -> u32 {
        self.max_stack_size
    }
}
