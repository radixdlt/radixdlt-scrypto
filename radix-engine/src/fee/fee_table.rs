use crate::wasm::WasmMeteringParams;

pub const WASM_ENGINE_CALL_COST: u32 = 10_000;
pub const ENGINE_RUN_COST: u32 = 20_000;

pub const WASM_METERING_V1: u8 = 1;
pub const WASM_INSTRUCTION_COST: u32 = 1;
pub const WASM_GROW_MEMORY_COST: u32 = 100;
pub const WASM_MAX_STACK_SIZE: u32 = 512;

pub struct FeeTable {
    engine_call_cost: u32,
    engine_run_cost: u32,
    wasm_metering_params: WasmMeteringParams,
}

impl FeeTable {
    pub fn new() -> Self {
        Self {
            engine_call_cost: WASM_ENGINE_CALL_COST,
            engine_run_cost: ENGINE_RUN_COST,
            wasm_metering_params: WasmMeteringParams::new(
                WASM_METERING_V1,
                WASM_INSTRUCTION_COST,
                WASM_GROW_MEMORY_COST,
                WASM_MAX_STACK_SIZE,
            ),
        }
    }

    pub fn engine_call_cost(&self) -> u32 {
        self.engine_call_cost
    }

    pub fn engine_run_cost(&self) -> u32 {
        self.engine_run_cost
    }

    pub fn wasm_metering_params(&self) -> WasmMeteringParams {
        self.wasm_metering_params.clone()
    }
}
