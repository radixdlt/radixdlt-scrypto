/* Copyright 2021 Radix Publishing Ltd incorporated in Jersey (Channel Islands).
 *
 * Licensed under the Radix License, Version 1.0 (the "License"); you may not use this
 * file except in compliance with the License. You may obtain a copy of the License at:
 *
 * radixfoundation.org/licenses/LICENSE-v1
 *
 * The Licensor hereby grants permission for the Canonical version of the Work to be
 * published, distributed and used under or by reference to the Licensor's trademark
 * Radix ® and use of any unregistered trade names, logos or get-up.
 *
 * The Licensor provides the Work (and each Contributor provides its Contributions) on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied,
 * including, without limitation, any warranties or conditions of TITLE, NON-INFRINGEMENT,
 * MERCHANTABILITY, or FITNESS FOR A PARTICULAR PURPOSE.
 *
 * Whilst the Work is capable of being deployed, used and adopted (instantiated) to create
 * a distributed ledger it is your responsibility to test and validate the code, together
 * with all logic and performance of that code under all foreseeable scenarios.
 *
 * The Licensor does not make or purport to make and hereby excludes liability for all
 * and any representation, warranty or undertaking in any form whatsoever, whether express
 * or implied, to any entity or person, including any representation, warranty or
 * undertaking, as to the functionality security use, value or other characteristics of
 * any distributed ledger nor in respect the functioning or value of any tokens which may
 * be created stored or transferred using the Work. The Licensor does not warrant that the
 * Work or any use of the Work complies with any law or regulation in any territory where
 * it may be implemented or used or that it will be appropriate for any specific purpose.
 *
 * Neither the licensor nor any current or former employees, officers, directors, partners,
 * trustees, representatives, agents, advisors, contractors, or volunteers of the Licensor
 * shall be liable for any direct or indirect, special, incidental, consequential or other
 * losses of any kind, in tort, contract or otherwise (including but not limited to loss
 * of revenue, income or profits, or loss of use or data, or loss of reputation, or loss
 * of any economic or other opportunity of whatsoever nature or howsoever arising), arising
 * out of or in connection with (without limitation of any use, misuse, of any ledger system
 * or use made or its functionality or any performance or operation of any code or protocol
 * caused by bugs or programming or logic errors or otherwise);
 *
 * A. any offer, purchase, holding, use, sale, exchange or transmission of any
 * cryptographic keys, tokens or assets created, exchanged, stored or arising from any
 * interaction with the Work;
 *
 * B. any failure in a transmission or loss of any token or assets keys or other digital
 * artefacts due to errors in transmission;
 *
 * C. bugs, hacks, logic errors or faults in the Work or any communication;
 *
 * D. system software or apparatus including but not limited to losses caused by errors
 * in holding or transmitting tokens by any third-party;
 *
 * E. breaches or failure of security including hacker attacks, loss or disclosure of
 * password, loss of private key, unauthorised use or misuse of such passwords or keys;
 *
 * F. any losses including loss of anticipated savings or other benefits resulting from
 * use of the Work or any changes to the Work (however implemented).
 *
 * You are solely responsible for; testing, validating and evaluation of all operation
 * logic, functionality, security and appropriateness of using the Work for any commercial
 * or non-commercial purpose and for any reproduction or redistribution by You of the
 * Work. You assume all risks associated with Your use of the Work and the exercise of
 * permissions under this License.
 */

// This file contains code sourced from https://github.com/paritytech/substrate/tree/monthly-2023-06
// This original source is licensed under https://github.com/paritytech/substrate/blob/monthly-2023-06/LICENSE-APACHE2
//
// The code in this file has been implemented by Radix® pursuant to an Apache 2 licence and has
// been modified by Radix® and is now licensed pursuant to the Radix® Open-Source Licence.
//
// Each sourced code fragment includes an inline attribution to the original source file in a
// comment starting "SOURCE: ..."
//
// Modifications from the original source are captured in two places:
// * Initial changes to get the code functional/integrated are marked by inline "INITIAL-MODIFICATION: ..." comments
// * Subsequent changes to the code are captured in the git commit history
//
// The following notice is retained from the original source
// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// SOURCE: https://github.com/paritytech/substrate/blob/monthly-2023-06/primitives/weights/src/weight_v2.rs#L29
pub struct Weight {
    /// The weight of computational time used based on some reference hardware.
    ref_time: u64,
    /// The weight of storage space used by proof of validity.
    proof_size: u64,
}

// SOURCE: https://github.com/paritytech/substrate/blob/monthly-2023-06/primitives/weights/src/weight_v2.rs#L38
impl Weight {
    /// Construct [`Weight`] from weight parts, namely reference time and proof size weights.
    pub const fn from_parts(ref_time: u64, proof_size: u64) -> Self {
        Self {
            ref_time,
            proof_size,
        }
    }

    /// Return the reference time part of the weight.
    pub const fn ref_time(&self) -> u64 {
        self.ref_time
    }

    /// Return the storage size part of the weight.
    pub const fn proof_size(&self) -> u64 {
        self.proof_size
    }

    /// Saturating [`Weight`] addition. Computes `self + rhs`, saturating at the numeric bounds of
    /// all fields instead of overflowing.
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self {
            ref_time: self.ref_time.saturating_add(rhs.ref_time),
            proof_size: self.proof_size.saturating_add(rhs.proof_size),
        }
    }

    /// Saturating [`Weight`] subtraction. Computes `self - rhs`, saturating at the numeric bounds
    /// of all fields instead of overflowing.
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self {
            ref_time: self.ref_time.saturating_sub(rhs.ref_time),
            proof_size: self.proof_size.saturating_sub(rhs.proof_size),
        }
    }

    /// Saturating [`Weight`] scalar multiplication. Computes `self.field * scalar` for all fields,
    /// saturating at the numeric bounds of all fields instead of overflowing.
    pub const fn saturating_mul(self, scalar: u64) -> Self {
        Self {
            ref_time: self.ref_time.saturating_mul(scalar),
            proof_size: self.proof_size.saturating_mul(scalar),
        }
    }
}

// SOURCE: https://github.com/paritytech/substrate/blob/monthly-2023-06/frame/contracts/src/schedule.rs#L143
/// Describes the weight for all categories of supported wasm instructions.
///
/// There there is one field for each wasm instruction that describes the weight to
/// execute one instruction of that name. There are a few exceptions:
///
/// 1. If there is a i64 and a i32 variant of an instruction we use the weight
///    of the former for both.
/// 2. The following instructions are free of charge because they merely structure the
///    wasm module and cannot be spammed without making the module invalid (and rejected):
///    End, Unreachable, Return, Else
/// 3. The following instructions cannot be benchmarked because they are removed by any
///    real world execution engine as a preprocessing step and therefore don't yield a
///    meaningful benchmark result. However, in contrast to the instructions mentioned
///    in 2. they can be spammed. We price them with the same weight as the "default"
///    instruction (i64.const): Block, Loop, Nop
/// 4. We price both i64.const and drop as InstructionWeights.i64const / 2. The reason
///    for that is that we cannot benchmark either of them on its own but we need their
///    individual values to derive (by subtraction) the weight of all other instructions
///    that use them as supporting instructions. Supporting means mainly pushing arguments
///    and dropping return values in order to maintain a valid module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionWeights {
    /// Version of the instruction weights.
    ///
    /// # Note
    ///
    /// Should be incremented whenever any instruction weight is changed. The
    /// reason is that changes to instruction weights require a re-instrumentation
    /// in order to apply the changes to an already deployed code. The re-instrumentation
    /// is triggered by comparing the version of the current schedule with the version the code was
    /// instrumented with. Changes usually happen when pallet_contracts is re-benchmarked.
    ///
    /// Changes to other parts of the schedule should not increment the version in
    /// order to avoid unnecessary re-instrumentations.
    pub version: u32,
    /// Weight to be used for instructions which don't have benchmarks assigned.
    ///
    /// This weight is used whenever a code is uploaded with [`Determinism::Relaxed`]
    /// and an instruction (usually a float instruction) is encountered. This weight is **not**
    /// used if a contract is uploaded with [`Determinism::Enforced`]. If this field is set to
    /// `0` (the default) only deterministic codes are allowed to be uploaded.
    pub fallback: u32,
    pub i64const: u32,
    pub i64load: u32,
    pub i64store: u32,
    pub select: u32,
    pub r#if: u32,
    pub br: u32,
    pub br_if: u32,
    pub br_table: u32,
    pub br_table_per_entry: u32,
    pub call: u32,
    pub call_indirect: u32,
    pub call_per_local: u32,
    pub local_get: u32,
    pub local_set: u32,
    pub local_tee: u32,
    pub global_get: u32,
    pub global_set: u32,
    pub memory_size: u32,
    pub memory_grow: u32,
    pub i64clz: u32,
    pub i64ctz: u32,
    pub i64popcnt: u32,
    pub i64eqz: u32,
    pub i64extendsi32: u32,
    pub i64extendui32: u32,
    pub i32wrapi64: u32,
    pub i64eq: u32,
    pub i64ne: u32,
    pub i64lts: u32,
    pub i64ltu: u32,
    pub i64gts: u32,
    pub i64gtu: u32,
    pub i64les: u32,
    pub i64leu: u32,
    pub i64ges: u32,
    pub i64geu: u32,
    pub i64add: u32,
    pub i64sub: u32,
    pub i64mul: u32,
    pub i64divs: u32,
    pub i64divu: u32,
    pub i64rems: u32,
    pub i64remu: u32,
    pub i64and: u32,
    pub i64or: u32,
    pub i64xor: u32,
    pub i64shl: u32,
    pub i64shrs: u32,
    pub i64shru: u32,
    pub i64rotl: u32,
    pub i64rotr: u32,
}

macro_rules! replace_token {
    ($_in:tt $replacement:tt) => {
        $replacement
    };
}

macro_rules! call_zero {
	($name:ident, $( $arg:expr ),*) => {
		InstructionWeights::$name($( replace_token!($arg 0) ),*)
	};
}

macro_rules! cost_args {
	($name:ident, $( $arg: expr ),+) => {
		(InstructionWeights::$name($( $arg ),+).saturating_sub(call_zero!($name, $( $arg ),+)))
	}
}

macro_rules! cost_instr_no_params {
    ($name:ident) => {
        cost_args!($name, 1).ref_time() as u32
    };
}

macro_rules! cost_instr {
    ($name:ident, $num_params:expr) => {
        cost_instr_no_params!($name)
            .saturating_sub((cost_instr_no_params!(instr_i64const) / 2).saturating_mul($num_params))
    };
}

// SOURCE: https://github.com/paritytech/substrate/blob/monthly-2023-06/frame/contracts/src/schedule.rs#L494
impl Default for InstructionWeights {
    fn default() -> Self {
        Self {
            version: 4,
            fallback: 0,
            i64const: cost_instr!(instr_i64const, 1),
            i64load: cost_instr!(instr_i64load, 2),
            i64store: cost_instr!(instr_i64store, 2),
            select: cost_instr!(instr_select, 4),
            r#if: cost_instr!(instr_if, 3),
            br: cost_instr!(instr_br, 2),
            br_if: cost_instr!(instr_br_if, 3),
            br_table: cost_instr!(instr_br_table, 3),
            br_table_per_entry: cost_instr!(instr_br_table_per_entry, 0),
            call: cost_instr!(instr_call, 2),
            call_indirect: cost_instr!(instr_call_indirect, 3),
            call_per_local: cost_instr!(instr_call_per_local, 0),
            local_get: cost_instr!(instr_local_get, 1),
            local_set: cost_instr!(instr_local_set, 1),
            local_tee: cost_instr!(instr_local_tee, 2),
            global_get: cost_instr!(instr_global_get, 1),
            global_set: cost_instr!(instr_global_set, 1),
            memory_size: cost_instr!(instr_memory_size, 1),
            memory_grow: cost_instr!(instr_memory_grow, 1),
            i64clz: cost_instr!(instr_i64clz, 2),
            i64ctz: cost_instr!(instr_i64ctz, 2),
            i64popcnt: cost_instr!(instr_i64popcnt, 2),
            i64eqz: cost_instr!(instr_i64eqz, 2),
            i64extendsi32: cost_instr!(instr_i64extendsi32, 2),
            i64extendui32: cost_instr!(instr_i64extendui32, 2),
            i32wrapi64: cost_instr!(instr_i32wrapi64, 2),
            i64eq: cost_instr!(instr_i64eq, 3),
            i64ne: cost_instr!(instr_i64ne, 3),
            i64lts: cost_instr!(instr_i64lts, 3),
            i64ltu: cost_instr!(instr_i64ltu, 3),
            i64gts: cost_instr!(instr_i64gts, 3),
            i64gtu: cost_instr!(instr_i64gtu, 3),
            i64les: cost_instr!(instr_i64les, 3),
            i64leu: cost_instr!(instr_i64leu, 3),
            i64ges: cost_instr!(instr_i64ges, 3),
            i64geu: cost_instr!(instr_i64geu, 3),
            i64add: cost_instr!(instr_i64add, 3),
            i64sub: cost_instr!(instr_i64sub, 3),
            i64mul: cost_instr!(instr_i64mul, 3),
            i64divs: cost_instr!(instr_i64divs, 3),
            i64divu: cost_instr!(instr_i64divu, 3),
            i64rems: cost_instr!(instr_i64rems, 3),
            i64remu: cost_instr!(instr_i64remu, 3),
            i64and: cost_instr!(instr_i64and, 3),
            i64or: cost_instr!(instr_i64or, 3),
            i64xor: cost_instr!(instr_i64xor, 3),
            i64shl: cost_instr!(instr_i64shl, 3),
            i64shrs: cost_instr!(instr_i64shrs, 3),
            i64shru: cost_instr!(instr_i64shru, 3),
            i64rotl: cost_instr!(instr_i64rotl, 3),
            i64rotr: cost_instr!(instr_i64rotr, 3),
        }
    }
}

// SOURCE: https://github.com/paritytech/substrate/blob/monthly-2023-06/frame/contracts/src/weights.rs#L52
/// Weight functions needed for pallet_contracts.
pub trait WeightInfo {
    fn instr_i64const(r: u32) -> Weight;
    fn instr_i64load(r: u32) -> Weight;
    fn instr_i64store(r: u32) -> Weight;
    fn instr_select(r: u32) -> Weight;
    fn instr_if(r: u32) -> Weight;
    fn instr_br(r: u32) -> Weight;
    fn instr_br_if(r: u32) -> Weight;
    fn instr_br_table(r: u32) -> Weight;
    fn instr_br_table_per_entry(e: u32) -> Weight;
    fn instr_call(r: u32) -> Weight;
    fn instr_call_indirect(r: u32) -> Weight;
    fn instr_call_per_local(l: u32) -> Weight;
    fn instr_local_get(r: u32) -> Weight;
    fn instr_local_set(r: u32) -> Weight;
    fn instr_local_tee(r: u32) -> Weight;
    fn instr_global_get(r: u32) -> Weight;
    fn instr_global_set(r: u32) -> Weight;
    fn instr_memory_size(r: u32) -> Weight;
    fn instr_memory_grow(r: u32) -> Weight;
    fn instr_i64clz(r: u32) -> Weight;
    fn instr_i64ctz(r: u32) -> Weight;
    fn instr_i64popcnt(r: u32) -> Weight;
    fn instr_i64eqz(r: u32) -> Weight;
    fn instr_i64extendsi32(r: u32) -> Weight;
    fn instr_i64extendui32(r: u32) -> Weight;
    fn instr_i32wrapi64(r: u32) -> Weight;
    fn instr_i64eq(r: u32) -> Weight;
    fn instr_i64ne(r: u32) -> Weight;
    fn instr_i64lts(r: u32) -> Weight;
    fn instr_i64ltu(r: u32) -> Weight;
    fn instr_i64gts(r: u32) -> Weight;
    fn instr_i64gtu(r: u32) -> Weight;
    fn instr_i64les(r: u32) -> Weight;
    fn instr_i64leu(r: u32) -> Weight;
    fn instr_i64ges(r: u32) -> Weight;
    fn instr_i64geu(r: u32) -> Weight;
    fn instr_i64add(r: u32) -> Weight;
    fn instr_i64sub(r: u32) -> Weight;
    fn instr_i64mul(r: u32) -> Weight;
    fn instr_i64divs(r: u32) -> Weight;
    fn instr_i64divu(r: u32) -> Weight;
    fn instr_i64rems(r: u32) -> Weight;
    fn instr_i64remu(r: u32) -> Weight;
    fn instr_i64and(r: u32) -> Weight;
    fn instr_i64or(r: u32) -> Weight;
    fn instr_i64xor(r: u32) -> Weight;
    fn instr_i64shl(r: u32) -> Weight;
    fn instr_i64shrs(r: u32) -> Weight;
    fn instr_i64shru(r: u32) -> Weight;
    fn instr_i64rotl(r: u32) -> Weight;
    fn instr_i64rotr(r: u32) -> Weight;
}

// SOURCE: https://github.com/paritytech/substrate/blob/monthly-2023-06/frame/contracts/src/weights.rs#L184
impl WeightInfo for InstructionWeights {
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64const(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_405_000 picoseconds.
        Weight::from_parts(1_583_300, 0)
            // Standard Error: 1
            .saturating_add(Weight::from_parts(2_743, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64load(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_796_000 picoseconds.
        Weight::from_parts(2_279_812, 0)
            // Standard Error: 7
            .saturating_add(Weight::from_parts(6_339, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64store(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_768_000 picoseconds.
        Weight::from_parts(2_274_070, 0)
            // Standard Error: 4
            .saturating_add(Weight::from_parts(6_647, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_select(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_396_000 picoseconds.
        Weight::from_parts(1_730_388, 0)
            // Standard Error: 5
            .saturating_add(Weight::from_parts(8_918, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_if(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_383_000 picoseconds.
        Weight::from_parts(1_473_000, 0)
            // Standard Error: 22
            .saturating_add(Weight::from_parts(12_167, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_br(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_418_000 picoseconds.
        Weight::from_parts(1_490_208, 0)
            // Standard Error: 18
            .saturating_add(Weight::from_parts(6_271, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_br_if(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_396_000 picoseconds.
        Weight::from_parts(1_584_684, 0)
            // Standard Error: 61
            .saturating_add(Weight::from_parts(8_819, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_br_table(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_384_000 picoseconds.
        Weight::from_parts(1_501_244, 0)
            // Standard Error: 17
            .saturating_add(Weight::from_parts(12_311, 0).saturating_mul(r.into()))
    }
    /// The range of component `e` is `[1, 256]`.
    fn instr_br_table_per_entry(e: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_433_000 picoseconds.
        Weight::from_parts(1_594_462, 0)
            // Standard Error: 19
            .saturating_add(Weight::from_parts(29, 0).saturating_mul(e.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_call(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_420_000 picoseconds.
        Weight::from_parts(1_602_036, 0)
            // Standard Error: 16
            .saturating_add(Weight::from_parts(17_082, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_call_indirect(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_619_000 picoseconds.
        Weight::from_parts(2_069_590, 0)
            // Standard Error: 20
            .saturating_add(Weight::from_parts(24_049, 0).saturating_mul(r.into()))
    }
    /// The range of component `l` is `[0, 1024]`.
    fn instr_call_per_local(l: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_478_000 picoseconds.
        Weight::from_parts(1_699_579, 0)
            // Standard Error: 13
            .saturating_add(Weight::from_parts(1_651, 0).saturating_mul(l.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_local_get(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 3_123_000 picoseconds.
        Weight::from_parts(3_200_824, 0)
            // Standard Error: 12
            .saturating_add(Weight::from_parts(4_187, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_local_set(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 3_121_000 picoseconds.
        Weight::from_parts(3_302_628, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(4_193, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_local_tee(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 3_155_000 picoseconds.
        Weight::from_parts(3_359_832, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(4_829, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_global_get(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_547_000 picoseconds.
        Weight::from_parts(1_899_252, 0)
            // Standard Error: 13
            .saturating_add(Weight::from_parts(8_373, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_global_set(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_513_000 picoseconds.
        Weight::from_parts(1_892_537, 0)
            // Standard Error: 15
            .saturating_add(Weight::from_parts(9_177, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_memory_size(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_904_000 picoseconds.
        Weight::from_parts(2_140_940, 0)
            // Standard Error: 5
            .saturating_add(Weight::from_parts(3_926, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 16]`.
    fn instr_memory_grow(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_437_000 picoseconds.
        Weight::from_parts(4_481, 0)
            // Standard Error: 131_975
            .saturating_add(Weight::from_parts(14_765_592, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64clz(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_443_000 picoseconds.
        Weight::from_parts(1_596_467, 0)
            // Standard Error: 1
            .saturating_add(Weight::from_parts(4_251, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64ctz(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_372_000 picoseconds.
        Weight::from_parts(1_569_760, 0)
            // Standard Error: 7
            .saturating_add(Weight::from_parts(4_777, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64popcnt(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_411_000 picoseconds.
        Weight::from_parts(1_642_163, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(4_241, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64eqz(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_395_000 picoseconds.
        Weight::from_parts(1_726_615, 0)
            // Standard Error: 10
            .saturating_add(Weight::from_parts(4_631, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64extendsi32(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_373_000 picoseconds.
        Weight::from_parts(1_620_217, 0)
            // Standard Error: 1
            .saturating_add(Weight::from_parts(4_220, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64extendui32(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_423_000 picoseconds.
        Weight::from_parts(1_611_025, 0)
            // Standard Error: 11
            .saturating_add(Weight::from_parts(4_681, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i32wrapi64(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_402_000 picoseconds.
        Weight::from_parts(1_616_506, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(4_247, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64eq(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_464_000 picoseconds.
        Weight::from_parts(1_641_492, 0)
            // Standard Error: 8
            .saturating_add(Weight::from_parts(6_262, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64ne(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_401_000 picoseconds.
        Weight::from_parts(1_673_299, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(5_741, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64lts(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_414_000 picoseconds.
        Weight::from_parts(1_615_167, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(5_767, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64ltu(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_445_000 picoseconds.
        Weight::from_parts(1_687_595, 0)
            // Standard Error: 10
            .saturating_add(Weight::from_parts(6_201, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64gts(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_415_000 picoseconds.
        Weight::from_parts(1_629_044, 0)
            // Standard Error: 3
            .saturating_add(Weight::from_parts(6_318, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64gtu(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_377_000 picoseconds.
        Weight::from_parts(1_660_178, 0)
            // Standard Error: 3
            .saturating_add(Weight::from_parts(5_774, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64les(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_467_000 picoseconds.
        Weight::from_parts(1_619_688, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(5_761, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64leu(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_485_000 picoseconds.
        Weight::from_parts(1_619_756, 0)
            // Standard Error: 10
            .saturating_add(Weight::from_parts(6_248, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64ges(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_391_000 picoseconds.
        Weight::from_parts(1_629_993, 0)
            // Standard Error: 3
            .saturating_add(Weight::from_parts(6_339, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64geu(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_413_000 picoseconds.
        Weight::from_parts(1_605_123, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(5_774, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64add(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_470_000 picoseconds.
        Weight::from_parts(1_699_382, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(5_736, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64sub(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_394_000 picoseconds.
        Weight::from_parts(1_599_038, 0)
            // Standard Error: 5
            .saturating_add(Weight::from_parts(6_325, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64mul(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_422_000 picoseconds.
        Weight::from_parts(1_655_350, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(5_753, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64divs(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_407_000 picoseconds.
        Weight::from_parts(1_710_195, 0)
            // Standard Error: 8
            .saturating_add(Weight::from_parts(6_791, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64divu(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_406_000 picoseconds.
        Weight::from_parts(2_022_275, 0)
            // Standard Error: 13
            .saturating_add(Weight::from_parts(5_864, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64rems(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_424_000 picoseconds.
        Weight::from_parts(1_735_622, 0)
            // Standard Error: 8
            .saturating_add(Weight::from_parts(6_772, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64remu(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_457_000 picoseconds.
        Weight::from_parts(1_636_788, 0)
            // Standard Error: 4
            .saturating_add(Weight::from_parts(5_794, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64and(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_423_000 picoseconds.
        Weight::from_parts(1_703_832, 0)
            // Standard Error: 11
            .saturating_add(Weight::from_parts(6_158, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64or(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_401_000 picoseconds.
        Weight::from_parts(1_653_216, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(5_754, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64xor(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_419_000 picoseconds.
        Weight::from_parts(1_685_121, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(6_309, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64shl(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_395_000 picoseconds.
        Weight::from_parts(1_580_918, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(5_775, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64shrs(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_408_000 picoseconds.
        Weight::from_parts(1_646_493, 0)
            // Standard Error: 9
            .saturating_add(Weight::from_parts(6_237, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64shru(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_446_000 picoseconds.
        Weight::from_parts(1_633_531, 0)
            // Standard Error: 7
            .saturating_add(Weight::from_parts(5_759, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64rotl(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_478_000 picoseconds.
        Weight::from_parts(1_634_023, 0)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(5_771, 0).saturating_mul(r.into()))
    }
    /// The range of component `r` is `[0, 5000]`.
    fn instr_i64rotr(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_389_000 picoseconds.
        Weight::from_parts(1_627_867, 0)
            // Standard Error: 10
            .saturating_add(Weight::from_parts(6_175, 0).saturating_mul(r.into()))
    }
}
