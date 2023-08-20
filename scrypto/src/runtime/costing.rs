use radix_engine_common::math::Decimal;
use radix_engine_common::prelude::scrypto_decode;
use crate::engine::wasm_api::{copy_buffer, costing};

/// The transaction runtime.
#[derive(Debug)]
pub struct Costing;

impl Costing {
    pub fn execution_cost_unit_limit() -> u32 {
        unsafe { costing::costing_get_execution_cost_unit_limit() }
    }

    pub fn execution_cost_unit_price(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { costing::costing_get_execution_cost_unit_price() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn finalization_cost_unit_limit(&mut self) -> u32 {
        unsafe { costing::costing_get_finalization_cost_unit_limit() }
    }

    pub fn finalization_cost_unit_price(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { costing::costing_get_finalization_cost_unit_price() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn usd_price(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { costing::costing_get_usd_price() });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn tip_percentage(&mut self) -> u32 {
        unsafe { costing::costing_get_tip_percentage() }
    }

    pub fn fee_balance(&mut self) -> Decimal {
        let bytes = copy_buffer(unsafe { costing::costing_get_fee_balance() });
        scrypto_decode(&bytes).unwrap()
    }
}