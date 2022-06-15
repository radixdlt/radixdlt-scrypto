use crate::wasm::WasmMeteringParams;

pub const TX_VALIDATION_COST_PER_BYTE: u32 = 20;

pub const ENGINE_RUN_COST: u32 = 20_000;

pub const WASM_METERING_V1: u8 = 1;
pub const WASM_INSTRUCTION_COST: u32 = 1;
pub const WASM_GROW_MEMORY_COST: u32 = 100;
pub const WASM_ENGINE_CALL_COST: u32 = 10_000;
pub const WASM_MAX_STACK_SIZE: u32 = 512;

pub struct FeeTable {
    tx_validation_cost_per_byte: u32,
    wasm_engine_call_cost: u32,
    engine_run_cost: u32,
    wasm_metering_params: WasmMeteringParams,
}

impl FeeTable {
    pub fn new() -> Self {
        Self {
            tx_validation_cost_per_byte: TX_VALIDATION_COST_PER_BYTE,
            wasm_engine_call_cost: WASM_ENGINE_CALL_COST,
            engine_run_cost: ENGINE_RUN_COST,
            wasm_metering_params: WasmMeteringParams::new(
                WASM_METERING_V1,
                WASM_INSTRUCTION_COST,
                WASM_GROW_MEMORY_COST,
                WASM_MAX_STACK_SIZE,
            ),
        }
    }

    pub fn tx_validation_cost_per_byte(&self) -> u32 {
        self.tx_validation_cost_per_byte
    }

    pub fn engine_run_cost(&self) -> u32 {
        self.engine_run_cost
    }

    pub fn wasm_engine_call_cost(&self) -> u32 {
        self.wasm_engine_call_cost
    }

    pub fn wasm_metering_params(&self) -> WasmMeteringParams {
        self.wasm_metering_params.clone()
    }
}
