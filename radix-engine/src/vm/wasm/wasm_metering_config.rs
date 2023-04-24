use super::InstructionCostRules;
use crate::types::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, ScryptoSbor)]
pub enum WasmMeteringConfig {
    V0,
}

impl Default for WasmMeteringConfig {
    fn default() -> Self {
        Self::V0
    }
}

impl WasmMeteringConfig {
    pub fn parameters(&self) -> WasmMeteringParams {
        match self {
            Self::V0 => WasmMeteringParams::new(InstructionCostRules::tiered(1, 5, 10, 5000), 1024),
        }
    }
}

#[derive(Debug, Clone, Sbor)]
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

    pub fn instruction_cost_rules(&self) -> &InstructionCostRules {
        &self.instruction_cost_rules
    }

    pub fn max_stack_size(&self) -> u32 {
        self.max_stack_size
    }
}
