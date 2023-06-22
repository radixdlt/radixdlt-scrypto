use crate::types::*;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref NATIVE_FUNCTION_BASE_COSTS: IndexMap<&'static str, IndexMap<&'static str, u32>> = {
        let mut costs: IndexMap<&'static str, IndexMap<&'static str, u32>> = index_map_new();
        include_str!("../../../../../assets/native_function_base_costs.csv")
            .split("\n")
            .filter(|x| x.len() > 0)
            .for_each(|x| {
                let mut tokens = x.split(",");
                let blueprint_name = tokens.next().unwrap();
                let function_name = tokens.next().unwrap();
                let cost = tokens.next().unwrap();
                costs
                    .entry(blueprint_name)
                    .or_default()
                    .insert(function_name, u32::from_str(cost).unwrap());
            });
        costs
    };
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct FeeTable {
    tx_base_cost: u32,
    tx_payload_cost_per_byte: u32,
    tx_signature_verification_cost_per_sig: u32,
}

impl FeeTable {
    pub fn new() -> Self {
        Self {
            tx_base_cost: 50_000,
            tx_payload_cost_per_byte: 5,
            tx_signature_verification_cost_per_sig: 100_000,
        }
    }

    pub fn tx_base_cost(&self) -> u32 {
        self.tx_base_cost
    }

    pub fn tx_payload_cost_per_byte(&self) -> u32 {
        self.tx_payload_cost_per_byte
    }

    pub fn tx_signature_verification_cost_per_sig(&self) -> u32 {
        self.tx_signature_verification_cost_per_sig
    }

    pub fn native_function_base_cost(
        &self,
        blueprint_name: &str,
        function_name: &str,
    ) -> Option<u32> {
        NATIVE_FUNCTION_BASE_COSTS
            .get(blueprint_name)
            .and_then(|x| x.get(function_name).cloned())
    }
}
