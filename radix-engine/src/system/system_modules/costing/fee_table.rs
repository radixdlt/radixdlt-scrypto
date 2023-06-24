use crate::{
    kernel::actor::Actor,
    track::interface::{StoreAccess, StoreAccessInfo},
    types::*,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref NATIVE_FUNCTION_BASE_COSTS: IndexMap<PackageAddress, IndexMap<&'static str, u32>> = {
        let mut costs: IndexMap<PackageAddress, IndexMap<&'static str, u32>> = index_map_new();
        include_str!("../../../../../assets/native_function_base_costs.csv")
            .split("\n")
            .filter(|x| x.len() > 0)
            .for_each(|x| {
                let mut tokens = x.split(",");
                let package_address =
                    PackageAddress::try_from_hex(tokens.next().unwrap().trim()).unwrap();
                let export_name = tokens.next().unwrap().trim();
                let cost = u32::from_str(tokens.next().unwrap().trim()).unwrap();
                costs
                    .entry(package_address)
                    .or_default()
                    .insert(export_name, cost);
            });
        costs
    };
}

/// Fee table specifies how each costing entry should be costed.
///
/// ## High Level Guideline
/// - Max cost unit limit: 100,000,000
/// - Cost unit price: 0.000005 XRD per cost unit
/// - Max execution costing, excluding tips: 500 XRD
/// - Basic transfer transaction cost: < 5 XRD
/// - Publishing a WASM package of max size costs: ~ 500 XRD
/// - Execution time for 100,000,000 cost units' worth of computation: <= 1 second
/// - Baseline: 1 microsecond = 100 cost units
/// - Non-time based costing will make the actual execution time less than anticipated
///
/// FIXME: fee table is actively adjusted at this point of time!
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

    fn transient_data_cost(size: usize) -> u32 {
        // Rationality:
        // To limit transient data to 64 MB, the cost for a byte should be 100,000,000 / 64,000,000 = 1.56.
        mul(cast(size), 2)
    }

    fn data_processing_cost(size: usize) -> u32 {
        add(mul(cast(size), 10), 1000)
    }

    fn store_access_cost(store_access: &StoreAccessInfo) -> u32 {
        const COSTING_COEFFICIENT_STORAGE: u32 = 14;
        const COSTING_COEFFICIENT_STORAGE_DIV_BITS: u32 = 8; // used to scale up or down all storage costing

        let mut sum = 0;
        for info in &store_access.0 {
            let cost = match info {
                StoreAccess::ReadFromDb(size) => {
                    if *size <= 25 * 1024 {
                        // apply constant value
                        400u32
                    } else {
                        // apply function: f(size) = 0.0009622109 * size + 389.5155
                        // approximated integer representation: f(size) = (63 * size) / 2^16 + 390
                        let mut value: u64 = *size as u64;
                        value *= 63; // 0.0009622109 * 2^16
                        value += (value >> 16) + 390;
                        value.try_into().unwrap_or(u32::MAX)
                    }
                }
                StoreAccess::ReadFromDbNotFound => 10000u32,
                StoreAccess::ReadFromTrack(size) => {
                    // apply function: f(size) = 0.00012232433 * size + 1.4939442
                    // approximated integer representation: f(size) = (8 * size) / 2^16 + 1
                    let mut value: u64 = *size as u64;
                    value *= 8; // 0.00082827697 * 2^16
                    value += (value >> 16) + 1;
                    value.try_into().unwrap_or(u32::MAX)
                }
                StoreAccess::WriteToTrack(size) => {
                    // apply function: f(size) = 0.0004 * size + 1000
                    // approximated integer representation: f(size) = (262 * size) / 2^16 + 1000
                    let mut value: u64 = *size as u64;
                    value *= 262; // 0.0004 * 2^16
                    value += (value >> 16) + 1000;
                    value.try_into().unwrap_or(u32::MAX)
                }
                StoreAccess::RewriteToTrack(size_old, size_new) => {
                    if size_new <= size_old {
                        // TODO: refund for reduced write size?
                        0
                    } else {
                        // calculate the delta
                        let mut value: u64 = (size_new - size_old) as u64;
                        value *= 262; // 0.0004 * 2^16
                        value += value >> 16;
                        value.try_into().unwrap_or(u32::MAX)
                    }
                }
                StoreAccess::DeleteFromTrack => {
                    191 // Average of P95 points from benchmark
                }
            };
            sum = add(sum, cost);
        }

        mul(sum, COSTING_COEFFICIENT_STORAGE) >> COSTING_COEFFICIENT_STORAGE_DIV_BITS
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
    pub fn run_native_code_cost(&self, package_address: &PackageAddress, export_name: &str) -> u32 {
        const COSTING_COEFFICIENT_CPU: u32 = 335;
        const COSTING_COEFFICIENT_CPU_DIV_BITS: u32 = 4; // used to divide by shift left operator
        const COSTING_COEFFICIENT_CPU_DIV_BITS_ADDON: u32 = 6; // used to scale up or down all cpu instruction costing

        let cpu_instructions = NATIVE_FUNCTION_BASE_COSTS
            .get(package_address)
            .and_then(|x| x.get(export_name).cloned())
            .unwrap_or(411524);

        mul(cpu_instructions, COSTING_COEFFICIENT_CPU)
            >> (COSTING_COEFFICIENT_CPU_DIV_BITS + COSTING_COEFFICIENT_CPU_DIV_BITS_ADDON)
    }

    #[inline]
    pub fn run_wasm_code_cost(
        &self,
        _package_address: &PackageAddress,
        _export_name: &str,
        gas: u32,
    ) -> u32 {
        const COST_UNITS_PER_GAS: u32 = 5;

        mul(COST_UNITS_PER_GAS, gas)
    }

    //======================
    // Kernel costs
    //======================

    #[inline]
    pub fn invoke_cost(&self, _actor: &Actor, input_size: usize) -> u32 {
        Self::data_processing_cost(input_size)
    }

    #[inline]
    pub fn allocate_node_id_cost(&self) -> u32 {
        212
    }

    #[inline]
    pub fn create_node_cost(
        &self,
        node_id: &NodeId,
        total_substate_size: usize,
        store_access: &StoreAccessInfo,
    ) -> u32 {
        // FIXME: add size count
        add3(
            Self::data_processing_cost(total_substate_size),
            Self::store_access_cost(store_access),
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
            },
        )
    }

    #[inline]
    pub fn drop_node_cost(&self, size: usize) -> u32 {
        add(
            324, // average of gathered data
            Self::data_processing_cost(size),
        )
    }

    #[inline]
    pub fn move_modules_cost(&self) -> u32 {
        // FIXME: add rule
        1
    }

    #[inline]
    pub fn open_substate_cost(&self, size: usize, store_access: &StoreAccessInfo) -> u32 {
        add3(
            100,
            Self::data_processing_cost(size),
            Self::store_access_cost(store_access),
        )
    }

    #[inline]
    pub fn read_substate_cost(&self, size: usize, store_access: &StoreAccessInfo) -> u32 {
        add3(
            174,
            Self::data_processing_cost(size),
            Self::store_access_cost(store_access),
        )
    }

    #[inline]
    pub fn write_substate_cost(&self, size: usize, store_access: &StoreAccessInfo) -> u32 {
        add3(
            126,
            Self::data_processing_cost(size),
            Self::store_access_cost(store_access),
        )
    }

    #[inline]
    pub fn close_substate_cost(&self, store_access: &StoreAccessInfo) -> u32 {
        add(100, Self::store_access_cost(store_access))
    }

    #[inline]
    pub fn set_substate_cost(&self, size: usize, store_access: &StoreAccessInfo) -> u32 {
        add3(
            100,
            Self::data_processing_cost(size),
            Self::store_access_cost(store_access),
        )
    }

    #[inline]
    pub fn remove_substate_cost(&self, store_access: &StoreAccessInfo) -> u32 {
        add(100, Self::store_access_cost(store_access))
    }

    #[inline]
    pub fn scan_sorted_substates_cost(&self, store_access: &StoreAccessInfo) -> u32 {
        add(100, Self::store_access_cost(store_access))
    }

    #[inline]
    pub fn scan_substates_cost(&self, store_access: &StoreAccessInfo) -> u32 {
        add(100, Self::store_access_cost(store_access))
    }

    #[inline]
    pub fn take_substates_cost(&self, store_access: &StoreAccessInfo) -> u32 {
        add(100, Self::store_access_cost(store_access))
    }

    //======================
    // System costs
    //======================

    #[inline]
    pub fn lock_fee_cost(&self) -> u32 {
        100
    }

    #[inline]
    pub fn query_fee_reserve_cost(&self) -> u32 {
        100
    }

    #[inline]
    pub fn query_actor_cost(&self) -> u32 {
        100
    }

    #[inline]
    pub fn query_auth_zone_cost(&self) -> u32 {
        100
    }

    #[inline]
    pub fn assert_access_rule_cost(&self) -> u32 {
        1000
    }

    #[inline]
    pub fn query_transaction_hash_cost(&self) -> u32 {
        100
    }

    #[inline]
    pub fn generate_ruid_cost(&self) -> u32 {
        300
    }

    #[inline]
    pub fn emit_event_cost(&self, size: usize) -> u32 {
        10000 + Self::data_processing_cost(size) + Self::transient_data_cost(size)
    }

    #[inline]
    pub fn emit_log_cost(&self, size: usize) -> u32 {
        10000 + Self::data_processing_cost(size) + Self::transient_data_cost(size)
    }

    #[inline]
    pub fn panic_cost(&self, size: usize) -> u32 {
        10000 + Self::data_processing_cost(size) + Self::transient_data_cost(size)
    }

    //======================
    // System module costs
    //======================
    // FIXME: add more costing rules
    // We should account for running system modules, such as auth and royalty.
}

#[inline]
fn cast(a: usize) -> u32 {
    u32::try_from(a).unwrap_or(u32::MAX)
}

#[inline]
fn add(a: u32, b: u32) -> u32 {
    a.checked_add(b).unwrap_or(u32::MAX)
}

#[inline]
fn add3(a: u32, b: u32, c: u32) -> u32 {
    add(add(a, b), c)
}

#[inline]
fn mul(a: u32, b: u32) -> u32 {
    a.checked_mul(b).unwrap_or(u32::MAX)
}
