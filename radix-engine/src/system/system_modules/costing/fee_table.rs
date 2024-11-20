use crate::internal_prelude::*;
use crate::kernel::kernel_callback_api::{
    CheckReferenceEvent, CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent,
    MoveModuleEvent, OpenSubstateEvent, ReadSubstateEvent, RemoveSubstateEvent, ScanKeysEvent,
    ScanSortedSubstatesEvent, SetSubstateEvent, WriteSubstateEvent,
};
use crate::kernel::substate_io::SubstateDevice;
use crate::system::actor::Actor;
use crate::system::system_callback::SystemVersion;
use crate::system::system_modules::transaction_runtime::Event;
use crate::{
    blueprints::package::*,
    track::interface::{IOAccess, StoreCommit},
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
        include_str!("../../../../assets/native_function_base_costs.csv")
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
            .insert(PACKAGE_PUBLISH_NATIVE_IDENT, (875, 11477486));
        costs
            .entry(PACKAGE_PACKAGE)
            .or_default()
            // TODO: publish_wasm_advanced is too expensive, dividing by 6 to let large package (1MiB) to be published, consider using cubic approximation
            .insert(PACKAGE_PUBLISH_WASM_ADVANCED_IDENT, (9063 / 6, 11072798));
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
pub struct FeeTable {
    wasm_execution_units_divider: u32,
}

impl FeeTable {
    pub fn new(version: SystemVersion) -> Self {
        let wasm_execution_units_divider = match version {
            // From `costing::spin_loop`, it takes 5.5391 ms for 1918122691 wasm execution units.
            // Therefore, cost for single unit: 5.5391 *  1000 / 1918122691 * 100 = 0.00028877714
            // 1 / 0.00028877714 = 3462 rounded down gives 3000
            SystemVersion::V1 => 3000,

            // W - WASM execution units
            // C - cost units
            // c - single cost unit
            // T - execution time (1 µs = 100 c => 1 ms = 100,000 c)
            //
            // Cost units might be expressed as
            //  C = T * c
            //
            // To convert W to C, we need a d.
            //   C = W / divider
            //   divider = W / C
            //   divider = W / (T * c)
            //
            // From `costing::spin_loop_v2` it consumes W=438,729,340,586 wasm execution
            // units and it should never take more than T = 1s.
            //   T = 1s = 1000 ms = 1 * 100,000
            //   W = 438,729,340,586
            // Therefore
            //   divider = 438,729,340,586 / (1000 * 100,000) = 4387.293 ~= 4500
            //
            // With divider set to 4500 it takes 543 ms (measured at GH benchmark, git rev c591c4003a,
            // EC2 instance type c6a.4xlarge) which is fine.
            SystemVersion::V2 | SystemVersion::V3 => 4500,
        };

        Self {
            wasm_execution_units_divider,
        }
    }

    pub fn latest() -> Self {
        Self::cuttlefish()
    }

    pub fn cuttlefish() -> Self {
        Self::new(SystemVersion::V2)
    }

    pub fn bottlenose() -> Self {
        Self::new(SystemVersion::V1)
    }

    pub fn anemone() -> Self {
        Self::new(SystemVersion::V1)
    }

    pub fn babylon() -> Self {
        Self::new(SystemVersion::V1)
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

    fn io_access_cost(&self, io_access: &IOAccess) -> u32 {
        match io_access {
            IOAccess::ReadFromDb(_, size) => {
                // Execution time (µs): 0.0009622109 * size + 389.5155
                // Execution cost: (0.0009622109 * size + 389.5155) * 100 = 0.1 * size + 40,000
                // See: https://radixdlt.atlassian.net/wiki/spaces/S/pages/3091562563/RocksDB+metrics
                add(cast(*size) / 10, 40_000)
            }
            IOAccess::ReadFromDbNotFound(_) => {
                // Execution time (µs): varies, using max 1,600
                // Execution cost: 1,600 * 100
                // See: https://radixdlt.atlassian.net/wiki/spaces/S/pages/3091562563/RocksDB+metrics
                160_000
            }
            IOAccess::HeapSubstateUpdated { .. } | IOAccess::TrackSubstateUpdated { .. } => {
                // Heap/track substate total size is limited by limits module.
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
    pub fn check_reference(&self, event: &CheckReferenceEvent) -> u32 {
        match event {
            CheckReferenceEvent::IOAccess(io_access) => self.io_access_cost(io_access),
        }
    }

    #[inline]
    pub fn check_intent_validity(&self) -> u32 {
        // Equivalent to an `IOAccess::ReadFromDbNotFound`
        160000
    }

    #[inline]
    pub fn check_timestamp(&self) -> u32 {
        // Equivalent to an `IOAccess::ReadFromDb`
        40_000
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
                    .unwrap_or_else(|| {
                        panic!(
                            "Native function not found: {:?}::{}. ",
                            package_address, export_name
                        )
                    })
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
        wasm_execution_units / self.wasm_execution_units_divider
    }

    #[inline]
    pub fn instantiate_wasm_code_cost(&self, size: usize) -> u32 {
        // From `costing::instantiate_radiswap`, it takes 3.3271 ms to instantiate WASM of length 288406.
        // Therefore, cost for byte: 3.3271 *  1000 / 203950 * 100 = 1.63133120863

        mul(cast(size), 2)
    }

    #[inline]
    pub fn before_invoke_cost(&self, _actor: &Actor, input_size: usize) -> u32 {
        Self::data_processing_cost(input_size)
    }

    #[inline]
    pub fn after_invoke_cost(&self, input_size: usize) -> u32 {
        Self::data_processing_cost(input_size)
    }

    #[inline]
    pub fn allocate_node_id_cost(&self) -> u32 {
        3312 / CPU_INSTRUCTIONS_TO_COST_UNIT
    }

    #[inline]
    pub fn create_node_cost(&self, event: &CreateNodeEvent) -> u32 {
        match event {
            CreateNodeEvent::Start(_node_id, node_substates) => {
                let total_substate_size = node_substates
                    .values()
                    .map(|x| x.values().map(|x| x.len()).sum::<usize>())
                    .sum::<usize>();
                add(
                    15510 / CPU_INSTRUCTIONS_TO_COST_UNIT,
                    Self::data_processing_cost(total_substate_size),
                )
            }
            CreateNodeEvent::IOAccess(io_access) => self.io_access_cost(io_access),
            CreateNodeEvent::End(..) => 0,
        }
    }

    #[inline]
    pub fn pin_node_cost(&self, _node_id: &NodeId) -> u32 {
        424 / CPU_INSTRUCTIONS_TO_COST_UNIT
    }

    #[inline]
    pub fn drop_node_cost(&self, event: &DropNodeEvent) -> u32 {
        match event {
            DropNodeEvent::Start(..) => 0,
            DropNodeEvent::IOAccess(io_access) => self.io_access_cost(io_access),
            DropNodeEvent::End(_node_id, node_substates) => {
                let total_substate_size = node_substates
                    .values()
                    .map(|x| x.values().map(|x| x.len()).sum::<usize>())
                    .sum::<usize>();
                add(
                    38883 / CPU_INSTRUCTIONS_TO_COST_UNIT,
                    Self::data_processing_cost(total_substate_size),
                )
            }
        }
    }

    #[inline]
    pub fn move_module_cost(&self, event: &MoveModuleEvent) -> u32 {
        match event {
            MoveModuleEvent::IOAccess(io_access) => add(
                4791 / CPU_INSTRUCTIONS_TO_COST_UNIT,
                self.io_access_cost(io_access),
            ),
        }
    }

    #[inline]
    pub fn open_substate_cost(&self, event: &OpenSubstateEvent) -> u32 {
        match event {
            OpenSubstateEvent::Start { .. } => 0,
            OpenSubstateEvent::IOAccess(io_access) => self.io_access_cost(io_access),
            OpenSubstateEvent::End { size, .. } => add(
                10318 / CPU_INSTRUCTIONS_TO_COST_UNIT,
                Self::data_processing_cost(*size),
            ),
        }
    }

    #[inline]
    pub fn read_substate_cost(&self, event: &ReadSubstateEvent) -> u32 {
        match event {
            ReadSubstateEvent::OnRead { value, device, .. } => {
                let base_cost: u32 = match device {
                    SubstateDevice::Heap => 2234,
                    SubstateDevice::Store => 3868,
                };

                add(
                    base_cost / CPU_INSTRUCTIONS_TO_COST_UNIT,
                    Self::data_processing_cost(value.len()),
                )
            }
            ReadSubstateEvent::IOAccess(io_access) => self.io_access_cost(io_access),
        }
    }

    #[inline]
    pub fn write_substate_cost(&self, event: &WriteSubstateEvent) -> u32 {
        match event {
            WriteSubstateEvent::IOAccess(io_access) => self.io_access_cost(io_access),
            WriteSubstateEvent::Start { value, .. } => add(
                7441 / CPU_INSTRUCTIONS_TO_COST_UNIT,
                Self::data_processing_cost(value.len()),
            ),
        }
    }

    #[inline]
    pub fn close_substate_cost(&self, event: &CloseSubstateEvent) -> u32 {
        match event {
            CloseSubstateEvent::Start(..) => 4390 / CPU_INSTRUCTIONS_TO_COST_UNIT,
        }
    }

    #[inline]
    pub fn set_substate_cost(&self, event: &SetSubstateEvent) -> u32 {
        match event {
            SetSubstateEvent::Start(.., value) => add(
                4530 / CPU_INSTRUCTIONS_TO_COST_UNIT,
                Self::data_processing_cost(value.len()),
            ),
            SetSubstateEvent::IOAccess(io_access) => self.io_access_cost(io_access),
        }
    }

    #[inline]
    pub fn remove_substate_cost(&self, event: &RemoveSubstateEvent) -> u32 {
        match event {
            RemoveSubstateEvent::Start(..) => 24389 / CPU_INSTRUCTIONS_TO_COST_UNIT,
            RemoveSubstateEvent::IOAccess(io_access) => self.io_access_cost(io_access),
        }
    }

    #[inline]
    pub fn mark_substate_as_transient_cost(
        &self,
        _node_id: &NodeId,
        _partition_number: &PartitionNumber,
        _substate_key: &SubstateKey,
    ) -> u32 {
        1896 / CPU_INSTRUCTIONS_TO_COST_UNIT
    }

    #[inline]
    pub fn scan_keys_cost(&self, event: &ScanKeysEvent) -> u32 {
        match event {
            ScanKeysEvent::Start => 16938 / CPU_INSTRUCTIONS_TO_COST_UNIT,
            ScanKeysEvent::IOAccess(io_access) => self.io_access_cost(io_access),
        }
    }

    #[inline]
    pub fn drain_substates_cost(&self, event: &DrainSubstatesEvent) -> u32 {
        match event {
            DrainSubstatesEvent::Start(count) => {
                let cpu_instructions = add(9262, mul(9286, *count));
                cpu_instructions / CPU_INSTRUCTIONS_TO_COST_UNIT
            }
            DrainSubstatesEvent::IOAccess(io_access) => self.io_access_cost(io_access),
        }
    }

    #[inline]
    pub fn scan_sorted_substates_cost(&self, event: &ScanSortedSubstatesEvent) -> u32 {
        match event {
            ScanSortedSubstatesEvent::Start => 6369 / CPU_INSTRUCTIONS_TO_COST_UNIT,
            ScanSortedSubstatesEvent::IOAccess(io_access) => self.io_access_cost(io_access),
        }
    }

    #[inline]
    pub fn get_stack_id(&self) -> u32 {
        500
    }

    #[inline]
    pub fn get_owned_nodes(&self) -> u32 {
        500
    }

    #[inline]
    pub fn switch_stack(&self) -> u32 {
        500
    }

    #[inline]
    pub fn send_to_stack(&self, data_len: usize) -> u32 {
        500 + Self::data_processing_cost(data_len)
    }

    #[inline]
    pub fn set_call_frame_data(&self, data_len: usize) -> u32 {
        500 + Self::data_processing_cost(data_len)
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
    pub fn query_costing_module(&self) -> u32 {
        500
    }

    #[inline]
    pub fn query_actor_cost(&self) -> u32 {
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
    pub fn encode_bech32_address_cost(&self) -> u32 {
        500
    }

    #[inline]
    pub fn panic_cost(&self, size: usize) -> u32 {
        500 + Self::data_processing_cost(size)
    }

    #[inline]
    pub fn bls12381_v1_verify_cost(&self, size: usize) -> u32 {
        // Based on  `test_crypto_scrypto_verify_bls12381_v1_costing`
        // - For sizes less than 1024, instruction count remains the same.
        // - For greater sizes following linear equation might be applied:
        //   (used: https://www.socscistatistics.com/tests/regression/default.aspx)
        //   instructions_cnt = 35.83223 * size + 15563087.39
        //   Lets round:
        //    35.83223       -> 36
        //    15563087.39    -> 15650000 (increased slightly to get the positive difference between
        //             calculated and measured number of instructions)
        let size = if size < 1024 { 1024 } else { cast(size) };
        let instructions_cnt = add(mul(size, 36), 15650000);
        // Convert to cost units
        instructions_cnt / CPU_INSTRUCTIONS_TO_COST_UNIT
    }

    #[inline]
    pub fn bls12381_v1_aggregate_verify_cost(&self, sizes: &[usize]) -> u32 {
        // Observed that aggregated verify might be broken down into:
        // - steps depending on message size
        //   - aggregation of pairings of each corresponding key and message pair
        //   - commit each above aggregation
        // - steps that do not depend on message size
        //   - read signature from bytes: 281125 instructions
        //   - signature validation: 583573 instructions
        //   - aggregated pairing of signature to verify and initialization point: 3027639 instructions
        //   - final verification: 4280077 instructions
        //
        // more details and data in https://docs.google.com/spreadsheets/d/1rV0KyB7UQrg2tOenbh2MQ1fo9MXwrwPFW4l0_CVO-6o/edit?usp=sharing

        // Pairing aggregate
        // Following linerar equation might be applied:
        //   (used: https://www.socscistatistics.com/tests/regression/default.aspx)
        //   instructions_cnt = 34.42199 * size + 2620295.64271
        //   Lets round:
        //    34.42199      -> 35
        //    2620295.64271 -> 2620296
        //
        // Also observed that additional 16850000 instructions are performed
        // every multiple of 8
        let mut instructions_cnt = 0u32;

        for s in sizes {
            instructions_cnt = add(add(instructions_cnt, mul(35, cast(*s))), 2620296);
        }
        let multiplier = cast(sizes.len() / 8);
        instructions_cnt = add(instructions_cnt, mul(multiplier, 16850000));

        // Pairing commit
        // Observed that number commit instructions repeats every multiple of 8
        instructions_cnt = add(
            instructions_cnt,
            match sizes.len() % 8 {
                0 => 0,
                1 => 3051556,
                2 => 5020768,
                3 => 6990111,
                4 => 8959454,
                5 => 10928798,
                6 => 12898141,
                7 => 14867484,
                _ => unreachable!(),
            },
        );

        // Instructions that do not depend on size
        instructions_cnt = add(instructions_cnt, 281125 + 583573 + 3027639 + 4280077);

        // Observed that threaded takes ~1.21 more instructions than no threaded
        instructions_cnt = mul(instructions_cnt / 100, 121);
        // Convert to cost units
        instructions_cnt / CPU_INSTRUCTIONS_TO_COST_UNIT
    }

    #[inline]
    pub fn bls12381_v1_fast_aggregate_verify_cost(&self, size: usize, keys_cnt: usize) -> u32 {
        // Based on  `test_crypto_scrypto_bls12381_v1_fast_aggregate_verify_costing`
        // - For sizes less than 1024, instruction count remains the same.
        // - For greater sizes following linear equation might be applied:
        //   instructions_cnt = 35.008 * size + 626055.4801 * keys_cnt + 15125588.5419
        //   (used: https://www.socscistatistics.com/tests/multipleregression/default.aspx)
        //   Lets round:
        //    35.008        -> 36
        //    626055.4801   -> 626056
        //    15125588.5419 -> 15200000  (increased slightly to get the positive difference between
        //             calculated and measured number of instructions)
        let size = if size < 1024 { 1024 } else { cast(size) };
        let instructions_cnt = add(add(mul(size, 36), mul(cast(keys_cnt), 626056)), 15200000);
        // Convert to cost units
        instructions_cnt / CPU_INSTRUCTIONS_TO_COST_UNIT
    }

    #[inline]
    pub fn bls12381_g2_signature_aggregate_cost(&self, signatures_cnt: usize) -> u32 {
        // Based on  `test_crypto_scrypto_bls12381_g2_signature_aggregate_costing`
        // Following linear equation might be applied:
        //   instructions_cnt = 879553.91557 * signatures_cnt - 567872.58948
        //   (used: https://www.socscistatistics.com/tests/regression/default.aspx)
        //   Lets round:
        //    879553.91557 -> 879554
        //    567872.5895  -> 500000 (decreased to get more accurate difference between calculated
        //           and measured instructions)
        let instructions_cnt = sub(mul(cast(signatures_cnt), 879554), 500000);
        // Convert to cost units
        instructions_cnt / CPU_INSTRUCTIONS_TO_COST_UNIT
    }

    #[inline]
    pub fn keccak256_hash_cost(&self, size: usize) -> u32 {
        // Based on  `test_crypto_scrypto_keccak256_costing`
        // - For sizes less than 100, instruction count remains the same.
        // - For greater sizes following linear equation might be applied:
        //   instructions_cnt = 46.41919 * size + 2641.66077
        //   (used: https://www.socscistatistics.com/tests/regression/default.aspx)
        //   Lets round:
        //     46.41919  -> 47
        //     2641.66077 -> 2642
        let size = if size < 100 { 100 } else { cast(size) };
        let instructions_cnt = add(mul(size, 47), 2642);
        // Convert to cost units
        instructions_cnt / CPU_INSTRUCTIONS_TO_COST_UNIT
    }

    #[inline]
    pub fn blake2b256_hash_cost(&self, size: usize) -> u32 {
        // Based on  `test_crypto_scrypto_blake2b_256_costing`
        // - For sizes less than 100, instruction count remains the same.
        // - For greater sizes following linear equation might be applied:
        //   instructions_cnt = 14.79642 * size + 1111.02264
        //   (used: https://www.socscistatistics.com/tests/regression/default.aspx)
        //   Lets round:
        //     14.79642  -> 15
        //     1111.02264 -> 1600 (increased to get more accurate difference between calculated
        //          and measured instruction)
        let size = if size < 100 { 100 } else { cast(size) };
        let instructions_cnt = add(mul(size, 15), 1600);
        // Convert to cost units
        instructions_cnt / CPU_INSTRUCTIONS_TO_COST_UNIT
    }

    #[inline]
    pub fn ed25519_verify_cost(&self, size: usize) -> u32 {
        // Based on  `test_crypto_scrypto_verify_ed25519_costing`
        //   instructions_cnt = 33.08798 * size + 444420.94242
        //   (used: https://www.socscistatistics.com/tests/regression/default.aspx)
        //   Lets round:
        //     33.08798 -> 34
        //     444420.94242 -> 500000 (increased slightly make sure we get the positive difference between
        //             calculated and measured number of instructions)
        let instructions_cnt = add(mul(cast(size), 34), 500000);
        // Convert to cost units
        instructions_cnt / CPU_INSTRUCTIONS_TO_COST_UNIT
    }

    #[inline]
    pub fn secp256k1_ecdsa_verify_cost(&self) -> u32 {
        // Based on  `test_crypto_scrypto_verify_secp256k1_ecdsa_costing`
        //   instructions_cnt = 464236 (input is always 32 bytes long)
        //   Lets round:
        //     464236 -> 500000
        let instructions_cnt = 500000;
        // Convert to cost units
        instructions_cnt / CPU_INSTRUCTIONS_TO_COST_UNIT
    }

    #[inline]
    pub fn secp256k1_ecdsa_verify_and_key_recover_cost(&self) -> u32 {
        // Based on  `test_crypto_scrypto_key_recover_secp256k1_ecdsa`
        //   instructions_cnt = 464236 (input is always 32 bytes long)
        //   Lets round:
        //     463506 -> 500000
        let instructions_cnt = 500000;
        // Convert to cost units
        instructions_cnt / CPU_INSTRUCTIONS_TO_COST_UNIT
    }

    //======================
    // Finalization costs
    // This is primarily to account for the additional work on the Node side
    //======================

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

    #[inline]
    pub fn commit_intent_status(&self, num_of_intent_statuses: usize) -> u32 {
        // Equivalent to a substate insertion
        mul(cast(num_of_intent_statuses), 100_000)
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
fn sub(a: u32, b: u32) -> u32 {
    a.checked_sub(b).unwrap_or(u32::MAX)
}

#[inline]
fn mul(a: u32, b: u32) -> u32 {
    a.checked_mul(b).unwrap_or(u32::MAX)
}
