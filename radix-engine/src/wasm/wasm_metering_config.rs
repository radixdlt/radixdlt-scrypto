use super::InstructionCostRules;
use crate::types::*;
use radix_engine_lib::crypto::hash;

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct WasmMeteringConfig {
    params: WasmMeteringParams,
    hash: Hash,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct WasmMeteringParams {
    instruction_cost_rules: InstructionCostRules,
    max_stack_size: u32,
}

impl WasmMeteringConfig {
    pub fn new(instruction_cost_rules: InstructionCostRules, max_stack_size: u32) -> Self {
        let params = WasmMeteringParams {
            instruction_cost_rules,
            max_stack_size,
        };
        let hash = hash(scrypto_encode(&params));
        Self { params, hash }
    }

    /// Wasm fee table is statically applied to the wasm code.
    /// This identifier helps decide whether or not re-instrumentation is required.
    pub fn identifier(&self) -> &Hash {
        &self.hash
    }

    pub fn instruction_cost_rules(&self) -> &InstructionCostRules {
        &self.params.instruction_cost_rules
    }

    pub fn max_stack_size(&self) -> u32 {
        self.params.max_stack_size
    }
}
