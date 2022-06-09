pub trait FeeTable {
    fn engine_call_cost(&self) -> u32;
}

pub struct BasicFeeTable {
    flat_engine_call_cost: u32,
}

impl BasicFeeTable {
    pub fn new(flat_engine_call_cost: u32) -> Self {
        Self {
            flat_engine_call_cost,
        }
    }
}

impl FeeTable for BasicFeeTable {
    fn engine_call_cost(&self) -> u32 {
        self.flat_engine_call_cost
    }
}
