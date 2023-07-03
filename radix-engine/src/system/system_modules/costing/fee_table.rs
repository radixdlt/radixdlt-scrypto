use crate::{
    blueprints::package::*,
    kernel::actor::Actor,
    track::interface::{StoreAccess, StoreAccessInfo, StoreCommit},
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
    pub static ref NATIVE_FUNCTION_BASE_COSTS_SIZE_DEPENDENT: IndexMap<PackageAddress, IndexMap<&'static str, (u32, u32)>> = {
        let mut costs: IndexMap<PackageAddress, IndexMap<&'static str, (u32, u32)>> =
            index_map_new();
        // FIXME: investigate the coefficients more, as it's blocking package publishing within 100M cost units
        costs
            .entry(PACKAGE_PACKAGE)
            .or_default()
            .insert(PACKAGE_PUBLISH_NATIVE_IDENT, (1 /*1001*/, 7424624));
        costs
            .entry(PACKAGE_PACKAGE)
            .or_default()
            .insert(PACKAGE_PUBLISH_WASM_ADVANCED_IDENT, (1 /*1055*/, 14305886));
        costs
    };
}

/// Fee table specifies how each costing entry should be costed.
///
/// ## High Level Guideline
/// - Max cost unit limit: 100,000,000
/// - Cost unit price: 0.000001 XRD per cost unit
/// - Max execution costing: 100 XRD + tipping + state expansion + royalty
/// - Execution time for 100,000,000 cost units' worth of computation: <= 1 second
/// - Baseline: 1 microsecond = 100 cost units
/// - Non-time based costing will make the actual execution time less than anticipated
///
/// FIXME: fee table is actively adjusted at this point of time!
#[derive(Debug, Clone, ScryptoSbor)]
pub struct FeeTable;

impl FeeTable {
    pub fn new() -> Self {
        Self
    }

    fn transient_data_cost(size: usize) -> u32 {
        // Rationality:
        // To limit transient data to 64 MB, the cost for a byte should be 100,000,000 / 64,000,000 = 1.56.
        mul(cast(size), 2)
    }

    fn data_processing_cost(size: usize) -> u32 {
        // FIXME: add payload against schema validation costs

        // Based on benchmark `bench_decode_sbor`
        // Time for processing a byte: 10.244 µs / 1068 = 0.00959176029

        // Based on benchmark `bench_validate_sbor_payload`
        // Time for processing a byte: 10.075 µs / 1169 = 0.00861847733

        mul(cast(size), 2)
    }

    fn store_access_cost(store_access: &StoreAccessInfo) -> u32 {
        let mut sum = 0;
        for info in store_access {
            let cost = match info {
                StoreAccess::ReadFromDb(size) => {
                    // Execution time (µs): 0.0009622109 * size + 389.5155
                    // Execution cost: (0.0009622109 * size + 389.5155) * 100 = 0.1 * size + 40,000
                    // See: https://radixdlt.atlassian.net/wiki/spaces/S/pages/3091562563/RocksDB+metrics
                    add(cast(*size) / 10, 40_000)
                }
                StoreAccess::ReadFromDbNotFound => {
                    // Execution time (µs): varies, using max 1,600
                    // Execution cost: 1,600 * 100
                    // See: https://radixdlt.atlassian.net/wiki/spaces/S/pages/3091562563/RocksDB+metrics
                    160_000
                }
                StoreAccess::NewEntryInTrack => {
                    // The max number of entries is limited by limits module.
                    0
                }
            };
            sum = add(sum, cost);
        }
        sum
    }

    //======================
    // Commit costs
    //======================

    #[inline]
    pub fn store_commit_cost(&self, store_commit: &StoreCommit) -> u32 {
        // Execution time (µs): 0.0025 * size + 1000
        // Execution cost: (0.0025 * size + 1000) * 100 = 0.25 * size + 100,000
        // See: https://radixdlt.atlassian.net/wiki/spaces/S/pages/3091562563/RocksDB+metrics
        match store_commit {
            StoreCommit::Insert { size, .. } => add(cast(*size) / 4, 100_000),
            StoreCommit::Update { size, .. } => add(cast(*size) / 4, 100_000),
            StoreCommit::Delete { .. } => 100_000,
        }
    }

    //======================
    // Transaction costs
    //======================

    #[inline]
    pub fn tx_base_cost(&self) -> u32 {
        50_000
    }

    #[inline]
    pub fn tx_payload_cost(&self, size: usize) -> u32 {
        // Rational:
        // Transaction payload is propagated over a P2P network.
        // Larger size may slows down the network performance.
        // The size of a typical transfer transaction is 400 bytes, and the cost will be 400 * 50 = 20,000 cost units
        // The max size of a transaction is 1 MiB, and the cost will be 1,000,000 * 50 = 50,000,000 cost units
        // This is roughly 1/20 of storing data in substate store per current setup.
        mul(cast(size), 50)
    }

    #[inline]
    pub fn tx_signature_verification_cost(&self, n: usize) -> u32 {
        // Based on benchmark `bench_validate_secp256k1`
        // The cost for validating a single signature is: 67.522 µs * 100 units/µs = 7,000 cost units
        mul(cast(n), 7_000)
    }

    //======================
    // VM execution costs
    //======================

    #[inline]
    pub fn run_native_code_cost(
        &self,
        package_address: &PackageAddress,
        export_name: &str,
        input_size: &usize,
    ) -> u32 {
        let native_execution_units = NATIVE_FUNCTION_BASE_COSTS
            .get(package_address)
            .and_then(|x| x.get(export_name).cloned())
            .unwrap_or_else(|| {
                NATIVE_FUNCTION_BASE_COSTS_SIZE_DEPENDENT
                    .get(package_address)
                    .and_then(|x| x.get(export_name))
                    .and_then(|value| Some(add(value.1, mul(value.0, cast(*input_size)))))
                    .expect(&format!(
                        "Native function not found: {:?}::{}. ",
                        package_address, export_name
                    ))
            });

        // Reference EC2 instance c5.4xlarge has CPU clock 3.4 GHz which means in 1 µs it executes 3400 instructions
        // (1 core, single-threaded operation, skipping CPU cache influence).
        // Basing on above assumptions return native function execution time in µs: native_execution_units / 3400
        native_execution_units / 34
    }

    #[inline]
    pub fn run_wasm_code_cost(
        &self,
        _package_address: &PackageAddress,
        _export_name: &str,
        wasm_execution_units: u32,
    ) -> u32 {
        // From `costing::spin_loop`, it takes 5.5391 ms for 1918122691 wasm execution units.
        // Therefore, cost for single unit: 5.5391 *  1000 / 1918122691 * 100 = 0.00028877714

        wasm_execution_units / 3000
    }

    #[inline]
    pub fn instantiate_wasm_code_cost(&self, size: usize) -> u32 {
        // From `costing::instantiate_radiswap`, it takes 3.3271 ms to instantiate WASM of length 288406.
        // Therefore, cost for byte: 3.3271 *  1000 / 203950 * 100 = 1.63133120863

        mul(cast(size), 2)
    }

    //======================
    // Kernel costs
    //======================

    // FIXME: adjust base cost for following ops

    #[inline]
    pub fn before_invoke_cost(&self, _actor: &Actor, input_size: usize) -> u32 {
        add(500, Self::data_processing_cost(input_size))
    }

    #[inline]
    pub fn after_invoke_cost(&self, input_size: usize) -> u32 {
        Self::data_processing_cost(input_size)
    }

    #[inline]
    pub fn allocate_node_id_cost(&self) -> u32 {
        500
    }

    #[inline]
    pub fn create_node_cost(
        &self,
        _node_id: &NodeId,
        total_substate_size: usize,
        store_access: &StoreAccessInfo,
    ) -> u32 {
        add3(
            500,
            Self::data_processing_cost(total_substate_size),
            Self::store_access_cost(store_access),
        )
    }

    #[inline]
    pub fn drop_node_cost(&self, size: usize) -> u32 {
        add(500, Self::data_processing_cost(size))
    }

    #[inline]
    pub fn move_modules_cost(&self, store_access: &StoreAccessInfo) -> u32 {
        add(500, Self::store_access_cost(store_access))
    }

    #[inline]
    pub fn open_substate_cost(&self, size: usize, store_access: &StoreAccessInfo) -> u32 {
        add3(
            500,
            Self::data_processing_cost(size),
            Self::store_access_cost(store_access),
        )
    }

    #[inline]
    pub fn read_substate_cost(&self, size: usize, store_access: &StoreAccessInfo) -> u32 {
        add3(
            500,
            Self::data_processing_cost(size),
            Self::store_access_cost(store_access),
        )
    }

    #[inline]
    pub fn write_substate_cost(&self, size: usize, store_access: &StoreAccessInfo) -> u32 {
        add3(
            500,
            Self::data_processing_cost(size),
            Self::store_access_cost(store_access),
        )
    }

    #[inline]
    pub fn close_substate_cost(&self, store_access: &StoreAccessInfo) -> u32 {
        add(500, Self::store_access_cost(store_access))
    }

    #[inline]
    pub fn set_substate_cost(&self, size: usize, store_access: &StoreAccessInfo) -> u32 {
        add3(
            500,
            Self::data_processing_cost(size),
            Self::store_access_cost(store_access),
        )
    }

    #[inline]
    pub fn remove_substate_cost(&self, store_access: &StoreAccessInfo) -> u32 {
        add(500, Self::store_access_cost(store_access))
    }

    #[inline]
    pub fn scan_sorted_substates_cost(&self, store_access: &StoreAccessInfo) -> u32 {
        add(500, Self::store_access_cost(store_access))
    }

    #[inline]
    pub fn scan_substates_cost(&self, store_access: &StoreAccessInfo) -> u32 {
        add(500, Self::store_access_cost(store_access))
    }

    #[inline]
    pub fn take_substates_cost(&self, store_access: &StoreAccessInfo) -> u32 {
        add(500, Self::store_access_cost(store_access))
    }

    //======================
    // System costs
    //======================

    #[inline]
    pub fn lock_fee_cost(&self) -> u32 {
        500
    }

    #[inline]
    pub fn query_fee_reserve_cost(&self) -> u32 {
        500
    }

    #[inline]
    pub fn query_actor_cost(&self) -> u32 {
        500
    }

    #[inline]
    pub fn query_auth_zone_cost(&self) -> u32 {
        500
    }

    #[inline]
    pub fn assert_access_rule_cost(&self) -> u32 {
        500
    }

    #[inline]
    pub fn query_transaction_hash_cost(&self) -> u32 {
        500
    }

    #[inline]
    pub fn generate_ruid_cost(&self) -> u32 {
        500
    }

    #[inline]
    pub fn emit_event_cost(&self, size: usize) -> u32 {
        500 + Self::data_processing_cost(size) + Self::transient_data_cost(size)
    }

    #[inline]
    pub fn emit_log_cost(&self, size: usize) -> u32 {
        500 + Self::data_processing_cost(size) + Self::transient_data_cost(size)
    }

    #[inline]
    pub fn panic_cost(&self, size: usize) -> u32 {
        500 + Self::data_processing_cost(size) + Self::transient_data_cost(size)
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
