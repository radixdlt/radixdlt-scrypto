use crate::{kernel::actor::Actor, track::interface::StoreAccessInfo, types::*};
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

    //======================
    // Transaction costs
    //======================

    pub fn tx_base_cost(&self) -> u32 {
        self.tx_base_cost
    }

    pub fn tx_payload_cost_per_byte(&self) -> u32 {
        self.tx_payload_cost_per_byte
    }

    pub fn tx_signature_verification_cost_per_sig(&self) -> u32 {
        self.tx_signature_verification_cost_per_sig
    }

    //======================
    // VM execution costs
    //======================

    pub fn run_native_code_cost(
        &self,
        _package_address: &PackageAddress,
        _export_name: &str,
    ) -> u32 {
        // FIXME
        1
    }

    pub fn run_wasm_code_cost(
        &self,
        _package_address: &PackageAddress,
        _export_name: &str,
        gas: u32,
    ) -> u32 {
        // FIXME: add multiplier
        gas
    }

    //======================
    // Kernel costs
    //======================

    pub fn invoke_cost(&self, _actor: &Actor, _input_size: usize) -> u32 {
        // FIXME: add rule
        1
    }

    pub fn allocate_node_id_cost(&self) -> u32 {
        212
    }

    pub fn allocate_global_address_cost(&self) -> u32 {
        // FIXME: add rule
        1
    }

    pub fn create_node_cost(&self, entity_type: EntityType, _size: usize) -> u32 {
        // FIXME: add size count

        match entity_type {
            EntityType::GlobalAccessController => 1736,
            EntityType::GlobalAccount => 1640,
            EntityType::GlobalConsensusManager => 1203,
            EntityType::GlobalFungibleResourceManager => 1160,
            EntityType::GlobalGenericComponent => 2370,
            EntityType::GlobalIdentity => 838,
            EntityType::GlobalNonFungibleResourceManager => 1587,
            EntityType::GlobalPackage => 1493,
            EntityType::GlobalValidator => 2374,
            EntityType::GlobalVirtualSecp256k1Account => 1590,
            EntityType::GlobalVirtualSecp256k1Identity => 906,
            EntityType::InternalAccount => 329,
            EntityType::InternalFungibleVault => 368,
            EntityType::InternalGenericComponent => 336,
            EntityType::InternalKeyValueStore => 828,
            EntityType::InternalNonFungibleVault => 356,
            _ => 1182, // average of above values
        }
    }

    pub fn drop_node_cost(&self, _size: usize) -> u32 {
        324 // average of gathered data
    }

    pub fn open_substate_cost(&self, _db_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    pub fn read_substate_cost(&self, _db_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    pub fn write_substate_cost(&self, _db_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    pub fn close_substate_cost(&self, _db_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    pub fn set_substate_cost(&self, _db_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    pub fn remove_substate_cost(&self, _db_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    pub fn scan_sorted_substates_cost(&self, _db_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    pub fn scan_substates_cost(&self, _db_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    pub fn take_substates_cost(&self, _db_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    //======================
    // System costs
    //======================
    pub fn event_cost(&self, _size: usize) -> u32 {
        // FIXME: add rule
        1
    }

    pub fn log_cost(&self, _size: usize) -> u32 {
        // FIXME: add rule
        1
    }

    pub fn panic_cost(&self, _size: usize) -> u32 {
        // FIXME: add rule
        1
    }

    //======================
    // System module costs
    //======================
    // FIXME: add more costing rules
    // We should account for running modules, such as auth and royalty.
}
