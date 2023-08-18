use crate::kernel::kernel_callback_api::{
    CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent, MoveModuleEvent,
    OpenSubstateEvent, ReadSubstateEvent, RemoveSubstateEvent, ScanKeysEvent,
    ScanSortedSubstatesEvent, SetSubstateEvent, WriteSubstateEvent,
};
use crate::kernel::substate_io::SubstateDevice;
use crate::system::system_modules::transaction_runtime::Event;
use crate::{
    blueprints::package::*,
    kernel::actor::Actor,
    track::interface::{StoreAccess, StoreCommit},
    types::*,
};
use lazy_static::lazy_static;

// Reference EC2 instance c5.4xlarge has CPU clock 3.4 GHz which means in 1 µs it executes 3400 instructions
// (1 core, single-threaded operation, skipping CPU cache influence).
// Basing on above assumptions converting CPU instructions count to cost units requires divistion CPU instructions
// by 3400 and multiplication by 100 (1 µs = 100 cost units), so it is enough to divide by 34.
const CPU_INSTRUCTIONS_TO_COST_UNIT: u32 = 34;

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
        costs
            .entry(PACKAGE_PACKAGE)
            .or_default()
            .insert(PACKAGE_PUBLISH_NATIVE_IDENT, (794, 9121128));
        costs
            .entry(PACKAGE_PACKAGE)
            .or_default()
            // TODO: publish_wasm_advanced is too expensinve, dividing by 3 to let large package (1MiB) to be published
            .insert(PACKAGE_PUBLISH_WASM_ADVANCED_IDENT, (3273 / 3, 10224507));
    costs
    };
}

/// Fee table specifies how each costing entry should be costed.
///
/// ## High Level Guideline
/// - Max execution cost unit limit: 100,000,000
/// - Max execution cost unit limit: 50,000,000
/// - Transaction fee = Network Execution + Network Finalization + Tip + Network Storage + Royalties
/// - Execution time for 100,000,000 cost units' worth of computation: <= 1 second
/// - Baseline: 1 microsecond = 100 cost units
///
#[derive(Debug, Clone, ScryptoSbor)]
pub struct FeeTable;

impl FeeTable {
    pub fn new() -> Self {
        Self
    }

    //======================
    // Execution costs
    //======================

    fn data_processing_cost(size: usize) -> u32 {
        // Based on benchmark `bench_decode_sbor`
        // Time for processing a byte: 10.244 µs / 1068 = 0.00959176029

        // Based on benchmark `bench_validate_sbor_payload`
        // Time for processing a byte: 10.075 µs / 1169 = 0.00861847733

        mul(cast(size), 2)
    }

    fn store_access_cost(&self, store_access: &StoreAccess) -> u32 {
        match store_access {
            StoreAccess::ReadFromDb(_, size) => {
                // Execution time (µs): 0.0009622109 * size + 389.5155
                // Execution cost: (0.0009622109 * size + 389.5155) * 100 = 0.1 * size + 40,000
                // See: https://radixdlt.atlassian.net/wiki/spaces/S/pages/3091562563/RocksDB+metrics
                add(cast(*size) / 10, 40_000)
            }
            StoreAccess::ReadFromDbNotFound(_) => {
                // Execution time (µs): varies, using max 1,600
                // Execution cost: 1,600 * 100
                // See: https://radixdlt.atlassian.net/wiki/spaces/S/pages/3091562563/RocksDB+metrics
                160_000
            }
            StoreAccess::NewEntryInTrack(_, _) => {
                // The max number of entries is limited by limits module.
                0
            }
        }
    }

    #[inline]
    pub fn verify_tx_signatures_cost(&self, n: usize) -> u32 {
        // Based on benchmark `bench_validate_secp256k1`
        // The cost for validating a single signature is: 67.522 µs * 100 units/µs = 7,000 cost units
        mul(cast(n), 7_000)
    }

    #[inline]
    pub fn validate_tx_payload_cost(&self, size: usize) -> u32 {
        // Rational:
        // Transaction payload is propagated over a P2P network.
        // Larger size may slows down the network performance.
        // The size of a typical transfer transaction is 400 bytes, and the cost will be 400 * 40 = 16,000 cost units
        // The max size of a transaction is 1 MiB, and the cost will be 1,048,576 * 40 = 41,943,040 cost units
        // This is roughly 1/24 of storing data in substate store per current setup.
        mul(cast(size), 40)
    }

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

        native_execution_units / CPU_INSTRUCTIONS_TO_COST_UNIT
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

    #[inline]
    pub fn before_invoke_cost(&self, _actor: &Actor, input_size: usize) -> u32 {
        // used max cpu instruction counts
        add(
            1041 / CPU_INSTRUCTIONS_TO_COST_UNIT,
            Self::data_processing_cost(input_size),
        )
    }

    #[inline]
    pub fn after_invoke_cost(&self, input_size: usize) -> u32 {
        // used max cpu instruction counts
        add(
            4321 / CPU_INSTRUCTIONS_TO_COST_UNIT,
            Self::data_processing_cost(input_size),
        )
    }

    #[inline]
    pub fn allocate_node_id_cost(&self) -> u32 {
        3560u32 / CPU_INSTRUCTIONS_TO_COST_UNIT
    }

    #[inline]
    pub fn create_node_cost(&self, event: &CreateNodeEvent) -> u32 {
        match event {
            CreateNodeEvent::Start(_, node_substates) => {
                let base_cost: u32 = 5000;
                let total_substate_size = node_substates
                    .values()
                    .map(|x| x.values().map(|x| x.len()).sum::<usize>())
                    .sum::<usize>();
                add(
                    base_cost / CPU_INSTRUCTIONS_TO_COST_UNIT,
                    Self::data_processing_cost(total_substate_size),
                )
            }
            CreateNodeEvent::StoreAccess(store_access) => self.store_access_cost(store_access),
            CreateNodeEvent::End(..) => 0,
        }
    }

    #[inline]
    pub fn pin_node_cost(&self, _node_id: &NodeId) -> u32 {
        // TODO: Add correct cost
        100u32
    }

    #[inline]
    pub fn drop_node_cost(&self, event: &DropNodeEvent) -> u32 {
        match event {
            DropNodeEvent::Start(..) => 0,
            DropNodeEvent::End(_node_id, node_substates) => {
                let total_substate_size = node_substates
                    .values()
                    .map(|x| x.values().map(|x| x.len()).sum::<usize>())
                    .sum::<usize>();
                add(
                    30526u32 / CPU_INSTRUCTIONS_TO_COST_UNIT,
                    Self::data_processing_cost(total_substate_size),
                )
            }
        }
    }

    #[inline]
    pub fn move_module_cost(&self, event: &MoveModuleEvent) -> u32 {
        match event {
            MoveModuleEvent::StoreAccess(store_access) => add(
                2853u32 / CPU_INSTRUCTIONS_TO_COST_UNIT,
                self.store_access_cost(store_access),
            ),
        }
    }

    #[inline]
    pub fn open_substate_cost(&self, event: &OpenSubstateEvent) -> u32 {
        match event {
            OpenSubstateEvent::Start { .. } => 0,
            OpenSubstateEvent::StoreAccess(store_access) => self.store_access_cost(store_access),
            OpenSubstateEvent::End { size, .. } => {
                let base_cost: u32 = 8000;
                add(
                    base_cost / CPU_INSTRUCTIONS_TO_COST_UNIT,
                    Self::data_processing_cost(*size),
                )
            }
        }
    }

    #[inline]
    pub fn read_substate_cost(&self, event: &ReadSubstateEvent) -> u32 {
        match event {
            ReadSubstateEvent::OnRead { value, device, .. } => {
                let base_cost: u32 = match device {
                    SubstateDevice::Heap => 2127,
                    SubstateDevice::Store => 3345,
                };

                add(
                    base_cost / CPU_INSTRUCTIONS_TO_COST_UNIT,
                    Self::data_processing_cost(value.len()),
                )
            }
        }
    }

    #[inline]
    pub fn write_substate_cost(&self, event: &WriteSubstateEvent) -> u32 {
        match event {
            WriteSubstateEvent::StoreAccess(store_access) => self.store_access_cost(store_access),
            WriteSubstateEvent::Start { value, .. } => add(
                2003u32 / CPU_INSTRUCTIONS_TO_COST_UNIT,
                Self::data_processing_cost(value.len()),
            ),
        }
    }

    #[inline]
    pub fn close_substate_cost(&self, event: &CloseSubstateEvent) -> u32 {
        match event {
            CloseSubstateEvent::End(..) => 3596u32 / CPU_INSTRUCTIONS_TO_COST_UNIT,
        }
    }

    #[inline]
    pub fn set_substate_cost(&self, event: &SetSubstateEvent) -> u32 {
        match event {
            SetSubstateEvent::Start(.., value) => add(
                8026u32 / CPU_INSTRUCTIONS_TO_COST_UNIT,
                Self::data_processing_cost(value.len()),
            ),
            SetSubstateEvent::StoreAccess(store_access) => self.store_access_cost(store_access),
        }
    }

    #[inline]
    pub fn remove_substate_cost(&self, event: &RemoveSubstateEvent) -> u32 {
        match event {
            RemoveSubstateEvent::Start(..) => 16440u32 / CPU_INSTRUCTIONS_TO_COST_UNIT,
            RemoveSubstateEvent::StoreAccess(store_access) => self.store_access_cost(store_access),
        }
    }

    #[inline]
    pub fn mark_substate_as_transient_cost(
        &self,
        _node_id: &NodeId,
        _partition_number: &PartitionNumber,
        _substate_key: &SubstateKey,
    ) -> u32 {
        // TODO: Add correct costing
        100u32
    }

    #[inline]
    pub fn scan_keys_cost(&self, event: &ScanKeysEvent) -> u32 {
        match event {
            ScanKeysEvent::Start => 14285u32 / CPU_INSTRUCTIONS_TO_COST_UNIT,
            ScanKeysEvent::StoreAccess(store_access) => self.store_access_cost(store_access),
        }
    }

    #[inline]
    pub fn drain_substates_cost(&self, event: &DrainSubstatesEvent) -> u32 {
        match event {
            DrainSubstatesEvent::Start(count) => {
                let cpu_instructions = add(3140u32, mul(14227u32, *count));
                cpu_instructions / CPU_INSTRUCTIONS_TO_COST_UNIT
            }
            DrainSubstatesEvent::StoreAccess(store_access) => self.store_access_cost(store_access),
        }
    }

    #[inline]
    pub fn scan_sorted_substates_cost(&self, event: &ScanSortedSubstatesEvent) -> u32 {
        match event {
            ScanSortedSubstatesEvent::Start => 6388u32 / CPU_INSTRUCTIONS_TO_COST_UNIT,
            ScanSortedSubstatesEvent::StoreAccess(store_access) => {
                self.store_access_cost(store_access)
            }
        }
    }

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
    pub fn query_transaction_hash_cost(&self) -> u32 {
        500
    }

    #[inline]
    pub fn generate_ruid_cost(&self) -> u32 {
        500
    }

    #[inline]
    pub fn emit_event_cost(&self, size: usize) -> u32 {
        500 + Self::data_processing_cost(size)
    }

    #[inline]
    pub fn emit_log_cost(&self, size: usize) -> u32 {
        500 + Self::data_processing_cost(size)
    }

    #[inline]
    pub fn panic_cost(&self, size: usize) -> u32 {
        500 + Self::data_processing_cost(size)
    }

    //======================
    // Finalization costs
    // This is primarily to account for the additional work on the Node side
    //======================

    #[inline]
    pub fn base_cost(&self) -> u32 {
        50_000
    }

    #[inline]
    pub fn commit_state_updates_cost(&self, store_commit: &StoreCommit) -> u32 {
        // Committing state time (µs): 0.0025 * size + 1000
        // Finalization cost: (0.0025 * size + 1000) * 100 = 0.25 * size + 100,000
        // See: https://radixdlt.atlassian.net/wiki/spaces/S/pages/3091562563/RocksDB+metrics
        match store_commit {
            StoreCommit::Insert { size, .. } => add(cast(*size) / 4, 100_000),
            StoreCommit::Update { size, .. } => add(cast(*size) / 4, 100_000),
            StoreCommit::Delete { .. } => 100_000,
        }
    }

    #[inline]
    pub fn commit_events_cost(&self, events: &Vec<Event>) -> u32 {
        let mut sum = 0;
        for event in events {
            sum += add(cast(event.payload.len()) / 4, 5_000)
        }
        sum
    }

    #[inline]
    pub fn commit_logs_cost(&self, logs: &Vec<(Level, String)>) -> u32 {
        let mut sum = 0;
        for log in logs {
            sum += add(cast(log.1.len()) / 4, 1_000)
        }
        sum
    }
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
fn mul(a: u32, b: u32) -> u32 {
    a.checked_mul(b).unwrap_or(u32::MAX)
}
