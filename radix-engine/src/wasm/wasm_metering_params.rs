#[derive(Debug, Clone)]
pub struct WasmMeteringParams {
    /// Wasm fee table is statically applied to the wasm code.
    /// This identifier helps decide whether or not re-instrumentation is required.
    identifier: u8,
    instruction_cost: u32,
    grow_memory_cost: u32,
    max_stack_size: u32,
}

impl WasmMeteringParams {
    pub fn new(
        identifier: u8,
        instruction_cost: u32,
        grow_memory_cost: u32,
        max_stack_size: u32,
    ) -> Self {
        Self {
            identifier,
            instruction_cost,
            grow_memory_cost,
            max_stack_size,
        }
    }

    pub fn identifier(&self) -> u8 {
        self.identifier
    }

    pub fn instruction_cost(&self) -> u32 {
        self.instruction_cost
    }

    pub fn grow_memory_cost(&self) -> u32 {
        self.grow_memory_cost
    }

    pub fn max_stack_size(&self) -> u32 {
        self.max_stack_size
    }
}
