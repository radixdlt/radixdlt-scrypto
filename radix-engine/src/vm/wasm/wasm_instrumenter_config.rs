use crate::types::*;
use parity_wasm::elements::Instruction::{self, *};
use wasm_instrument::gas_metering::MemoryGrowCost;
use wasm_instrument::gas_metering::Rules;

use super::InstructionWeights;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmInstrumenterConfigV1 {
    weights: InstructionWeights,
    grow_memory_cost: u32,
    max_stack_size: u32,
}

impl WasmInstrumenterConfigV1 {
    pub fn new() -> Self {
        Self {
            weights: InstructionWeights::default(),
            grow_memory_cost: 16,
            max_stack_size: 1024,
        }
    }

    pub fn version(&self) -> u8 {
        1
    }

    pub fn max_stack_size(&self) -> u32 {
        self.max_stack_size
    }
}

impl Rules for WasmInstrumenterConfigV1 {
    fn instruction_cost(&self, instruction: &Instruction) -> Option<u32> {
        todo!()
    }

    fn memory_grow_cost(&self) -> MemoryGrowCost {
        MemoryGrowCost::Linear(
            NonZeroU32::new(self.grow_memory_cost).expect("GROW_MEMORY_COST value is zero"),
        )
    }
}
