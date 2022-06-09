pub struct WasmFeeTable {
    flat_instrution_cost: u32,
    grow_memory_cost: u32,
}

impl WasmFeeTable {
    pub fn new(flat_instrution_cost: u32, grow_memory_cost: u32) -> Self {
        Self {
            flat_instrution_cost,
            grow_memory_cost,
        }
    }

    pub fn instruction_cost(&self) -> u32 {
        self.flat_instrution_cost
    }

    pub fn grow_memory_cost(&self) -> u32 {
        self.grow_memory_cost
    }
}
