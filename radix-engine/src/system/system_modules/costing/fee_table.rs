use crate::{kernel::actor::Actor, track::interface::StoreAccessInfo, types::*};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref NATIVE_FUNCTION_BASE_COSTS: IndexMap<PackageAddress, IndexMap<&'static str, u32>> = {
        let mut costs: IndexMap<PackageAddress, IndexMap<&'static str, u32>> = index_map_new();
        include_str!("../../../../../assets/native_function_base_costs.csv")
            .split("\n")
            .filter(|x| x.len() > 0)
            .for_each(|x| {
                let mut tokens = x.split(",");
                let package_address = PackageAddress::try_from_hex(tokens.next().unwrap()).unwrap();
                let export_name = tokens.next().unwrap();
                let cost = u32::from_str(tokens.next().unwrap()).unwrap();
                costs
                    .entry(package_address)
                    .or_default()
                    .insert(export_name, cost);
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

    #[inline]
    pub fn tx_base_cost(&self) -> u32 {
        self.tx_base_cost
    }

    #[inline]
    pub fn tx_payload_cost_per_byte(&self) -> u32 {
        self.tx_payload_cost_per_byte
    }

    #[inline]
    pub fn tx_signature_verification_cost_per_sig(&self) -> u32 {
        self.tx_signature_verification_cost_per_sig
    }
    //======================
    // VM execution costs
    //======================

    #[inline]
    pub fn run_native_code_cost(
        &self,
        _package_address: &PackageAddress,
        _export_name: &str,
    ) -> u32 {
        // FIXME
        1
    }

    #[inline]
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

    #[inline]
    pub fn invoke_cost(&self, _actor: &Actor, _input_size: usize) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn allocate_node_id_cost(&self) -> u32 {
        212
    }

    #[inline]
    pub fn allocate_global_address_cost(&self) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn create_node_cost(
        &self,
        node_id: &NodeId,
        _total_size: usize,
        _store_access: &StoreAccessInfo,
    ) -> u32 {
        // FIXME: add size count
        if let Some(entity_type) = node_id.entity_type() {
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
        } else {
            1182 // average of above values
        }
    }

    #[inline]
    pub fn drop_node_cost(&self, _size: usize) -> u32 {
        324 // average of gathered data
    }

    #[inline]
    pub fn move_modules_cost(&self) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn open_substate_cost(&self, _size: usize, _store_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn read_substate_cost(&self, _size: usize, _store_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn write_substate_cost(&self, _size: usize, _store_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn close_substate_cost(&self, _store_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn set_substate_cost(&self, _store_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn remove_substate_cost(&self, _store_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn scan_sorted_substates_cost(&self, _store_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn scan_substates_cost(&self, _store_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn take_substates_cost(&self, _store_access: &StoreAccessInfo) -> u32 {
        // FIXME: add rule
        1
    }

    //======================
    // System costs
    //======================

    #[inline]
    pub fn lock_fee_cost(&self) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn query_fee_reserve_cost(&self) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn query_actor_cost(&self) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn query_auth_zone_cost(&self) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn assert_access_rule_cost(&self) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn query_transaction_hash_cost(&self) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn generate_ruid_cost(&self) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn emit_event_cost(&self, _size: usize) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn emit_log_cost(&self, _size: usize) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_x() {
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
    }
}
