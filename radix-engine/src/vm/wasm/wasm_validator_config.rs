extern crate radix_wasm_instrument as wasm_instrument;

use crate::internal_prelude::*;
use wasm_instrument::gas_metering::MemoryGrowCost;
use wasm_instrument::gas_metering::Rules;
use wasmparser::Operator::{self, *};

use super::InstructionWeights;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmValidatorConfigV1 {
    weights: InstructionWeights,
    max_stack_size: u32,
}

impl WasmValidatorConfigV1 {
    pub fn new() -> Self {
        Self {
            weights: InstructionWeights::default(),
            max_stack_size: 1024,
        }
    }

    pub fn version(&self) -> u8 {
        1
    }

    pub fn max_stack_size(&self) -> u32 {
        self.max_stack_size
    }
}

impl Rules for WasmValidatorConfigV1 {
    fn instruction_cost(&self, instruction: &Operator) -> Option<u32> {
        match instruction {
            // MVP
            Unreachable => Some(0),
            Nop => Some(self.weights.i64const),
            Block { .. } => Some(self.weights.i64const),
            Loop { .. } => Some(self.weights.i64const),
            If { .. } => Some(self.weights.r#if),
            Else { .. } => Some(0),
            End => Some(0),
            Br { .. } => Some(self.weights.br),
            BrIf { .. } => Some(self.weights.br_if),
            BrTable { .. } => Some(self.weights.br_table),
            Return => Some(0),
            Call { .. } => Some(self.weights.call),
            CallIndirect { .. } => Some(self.weights.call_indirect),
            Drop => Some(self.weights.i64const),
            Select => Some(self.weights.select),
            LocalGet { .. } => Some(self.weights.local_get),
            LocalSet { .. } => Some(self.weights.local_set),
            LocalTee { .. } => Some(self.weights.local_tee),
            GlobalGet { .. } => Some(self.weights.global_get),
            GlobalSet { .. } => Some(self.weights.global_set),
            I32Load { .. } => Some(self.weights.i64load),
            I64Load { .. } => Some(self.weights.i64load),
            F32Load { .. } => None,
            F64Load { .. } => None,
            I32Load8S { .. } => Some(self.weights.i64load),
            I32Load8U { .. } => Some(self.weights.i64load),
            I32Load16S { .. } => Some(self.weights.i64load),
            I32Load16U { .. } => Some(self.weights.i64load),
            I64Load8S { .. } => Some(self.weights.i64load),
            I64Load8U { .. } => Some(self.weights.i64load),
            I64Load16S { .. } => Some(self.weights.i64load),
            I64Load16U { .. } => Some(self.weights.i64load),
            I64Load32S { .. } => Some(self.weights.i64load),
            I64Load32U { .. } => Some(self.weights.i64load),
            I32Store { .. } => Some(self.weights.i64store),
            I64Store { .. } => Some(self.weights.i64store),
            F32Store { .. } => None,
            F64Store { .. } => None,
            I32Store8 { .. } => Some(self.weights.i64store),
            I32Store16 { .. } => Some(self.weights.i64store),
            I64Store8 { .. } => Some(self.weights.i64store),
            I64Store16 { .. } => Some(self.weights.i64store),
            I64Store32 { .. } => Some(self.weights.i64store),
            MemorySize { .. } => Some(self.weights.memory_size),
            MemoryGrow { .. } => Some(self.weights.memory_grow),
            I32Const { .. } => Some(self.weights.i64const),
            I64Const { .. } => Some(self.weights.i64const),
            F32Const { .. } => None,
            F64Const { .. } => None,
            I32Eqz => Some(self.weights.i64eqz),
            I32Eq => Some(self.weights.i64eq),
            I32Ne => Some(self.weights.i64ne),
            I32LtS => Some(self.weights.i64lts),
            I32LtU => Some(self.weights.i64ltu),
            I32GtS => Some(self.weights.i64gts),
            I32GtU => Some(self.weights.i64gtu),
            I32LeS => Some(self.weights.i64les),
            I32LeU => Some(self.weights.i64leu),
            I32GeS => Some(self.weights.i64ges),
            I32GeU => Some(self.weights.i64geu),
            I64Eqz => Some(self.weights.i64eqz),
            I64Eq => Some(self.weights.i64eq),
            I64Ne => Some(self.weights.i64ne),
            I64LtS => Some(self.weights.i64lts),
            I64LtU => Some(self.weights.i64ltu),
            I64GtS => Some(self.weights.i64gts),
            I64GtU => Some(self.weights.i64gtu),
            I64LeS => Some(self.weights.i64les),
            I64LeU => Some(self.weights.i64leu),
            I64GeS => Some(self.weights.i64ges),
            I64GeU => Some(self.weights.i64geu),
            F32Eq => None,
            F32Ne => None,
            F32Lt => None,
            F32Gt => None,
            F32Le => None,
            F32Ge => None,
            F64Eq => None,
            F64Ne => None,
            F64Lt => None,
            F64Gt => None,
            F64Le => None,
            F64Ge => None,
            I32Clz => Some(self.weights.i64clz),
            I32Ctz => Some(self.weights.i64ctz),
            I32Popcnt => Some(self.weights.i64popcnt),
            I32Add => Some(self.weights.i64add),
            I32Sub => Some(self.weights.i64sub),
            I32Mul => Some(self.weights.i64mul),
            I32DivS => Some(self.weights.i64divs),
            I32DivU => Some(self.weights.i64divu),
            I32RemS => Some(self.weights.i64rems),
            I32RemU => Some(self.weights.i64remu),
            I32And => Some(self.weights.i64and),
            I32Or => Some(self.weights.i64xor),
            I32Xor => Some(self.weights.i64xor),
            I32Shl => Some(self.weights.i64shl),
            I32ShrS => Some(self.weights.i64shrs),
            I32ShrU => Some(self.weights.i64shru),
            I32Rotl => Some(self.weights.i64rotl),
            I32Rotr => Some(self.weights.i64rotr),
            I64Clz => Some(self.weights.i64clz),
            I64Ctz => Some(self.weights.i64ctz),
            I64Popcnt => Some(self.weights.i64popcnt),
            I64Add => Some(self.weights.i64add),
            I64Sub => Some(self.weights.i64sub),
            I64Mul => Some(self.weights.i64mul),
            I64DivS => Some(self.weights.i64divs),
            I64DivU => Some(self.weights.i64divu),
            I64RemS => Some(self.weights.i64rems),
            I64RemU => Some(self.weights.i64remu),
            I64And => Some(self.weights.i64and),
            I64Or => Some(self.weights.i64or),
            I64Xor => Some(self.weights.i64xor),
            I64Shl => Some(self.weights.i64shl),
            I64ShrS => Some(self.weights.i64shrs),
            I64ShrU => Some(self.weights.i64shru),
            I64Rotl => Some(self.weights.i64rotl),
            I64Rotr => Some(self.weights.i64rotr),
            F32Abs => None,
            F32Neg => None,
            F32Ceil => None,
            F32Floor => None,
            F32Trunc => None,
            F32Nearest => None,
            F32Sqrt => None,
            F32Add => None,
            F32Sub => None,
            F32Mul => None,
            F32Div => None,
            F32Min => None,
            F32Max => None,
            F32Copysign => None,
            F64Abs => None,
            F64Neg => None,
            F64Ceil => None,
            F64Floor => None,
            F64Trunc => None,
            F64Nearest => None,
            F64Sqrt => None,
            F64Add => None,
            F64Sub => None,
            F64Mul => None,
            F64Div => None,
            F64Min => None,
            F64Max => None,
            F64Copysign => None,
            I32WrapI64 => Some(self.weights.i32wrapi64),
            I32TruncF32S => None,
            I32TruncF32U => None,
            I32TruncF64S => None,
            I32TruncF64U => None,
            I64ExtendI32S => Some(self.weights.i64extendsi32),
            I64ExtendI32U => Some(self.weights.i64extendui32),
            I64TruncF32S => None,
            I64TruncF32U => None,
            I64TruncF64S => None,
            I64TruncF64U => None,
            F32ConvertI32S => None,
            F32ConvertI32U => None,
            F32ConvertI64S => None,
            F32ConvertI64U => None,
            F32DemoteF64 => None,
            F64ConvertI32S => None,
            F64ConvertI32U => None,
            F64ConvertI64S => None,
            F64ConvertI64U => None,
            F64PromoteF32 => None,
            I32ReinterpretF32 => None,
            I64ReinterpretF64 => None,
            F32ReinterpretI32 => None,
            F64ReinterpretI64 => None,

            // sign-extension-ops proposal
            I32Extend8S => Some(self.weights.i64extendsi32),
            I32Extend16S => Some(self.weights.i64extendsi32),
            I64Extend8S => Some(self.weights.i64extendsi32),
            I64Extend16S => Some(self.weights.i64extendsi32),
            I64Extend32S => Some(self.weights.i64extendsi32),

            // Bulk memory proposal
            | MemoryInit { .. }
            | DataDrop { .. }
            | MemoryCopy { .. }
            | MemoryFill { .. }
            | TableInit { .. }
            | ElemDrop { .. }
            | TableCopy { .. }

            // Exception handling proposal
            | Try { .. }
            | Catch { .. }
            | Throw { .. }
            | Rethrow { .. }
            | Delegate { .. }
            | CatchAll { .. }

            // Typed function references proposal
            | CallRef { .. }
            | ReturnCallRef { .. }
            | RefAsNonNull { .. }
            | BrOnNull { .. }
            | BrOnNonNull { .. }

            // GC proposal
            | I31New { .. }
            | I31GetS { .. }
            | I31GetU { .. }

            // Memory control proposal
            | MemoryDiscard { .. }

            // Reference types proposal
            | TypedSelect { .. }
            | RefNull { .. }
            | RefIsNull { .. }
            | RefFunc { .. }
            | TableFill { .. }
            | TableGet { .. }
            | TableSet { .. }
            | TableGrow { .. }
            | TableSize { .. }

            // Relaxed SIMD proposal
            | I8x16RelaxedSwizzle { .. }
            | I32x4RelaxedTruncF32x4S { .. }
            | I32x4RelaxedTruncF32x4U { .. }
            | I32x4RelaxedTruncF64x2SZero { .. }
            | I32x4RelaxedTruncF64x2UZero { .. }
            | F32x4RelaxedMadd { .. }
            | F32x4RelaxedNmadd { .. }
            | F64x2RelaxedMadd { .. }
            | F64x2RelaxedNmadd { .. }
            | I8x16RelaxedLaneselect { .. }
            | I16x8RelaxedLaneselect { .. }
            | I32x4RelaxedLaneselect { .. }
            | I64x2RelaxedLaneselect { .. }
            | F32x4RelaxedMin { .. }
            | F32x4RelaxedMax { .. }
            | F64x2RelaxedMin { .. }
            | F64x2RelaxedMax { .. }
            | I16x8RelaxedQ15mulrS { .. }
            | I16x8RelaxedDotI8x16I7x16S { .. }
            | I32x4RelaxedDotI8x16I7x16AddS { .. }

            // nontrapping-float-to-int-conversions proposal
            | I32TruncSatF32S { .. }
            | I32TruncSatF32U { .. }
            | I32TruncSatF64S { .. }
            | I32TruncSatF64U { .. }
            | I64TruncSatF32S { .. }
            | I64TruncSatF32U { .. }
            | I64TruncSatF64S { .. }
            | I64TruncSatF64U { .. }

            // SIMD proposal
            | V128Load { .. }
            | V128Load8x8S { .. }
            | V128Load8x8U { .. }
            | V128Load16x4S { .. }
            | V128Load16x4U { .. }
            | V128Load32x2S { .. }
            | V128Load32x2U { .. }
            | V128Load8Splat { .. }
            | V128Load16Splat { .. }
            | V128Load32Splat { .. }
            | V128Load64Splat { .. }
            | V128Load32Zero { .. }
            | V128Load64Zero { .. }
            | V128Store { .. }
            | V128Load8Lane { .. }
            | V128Load16Lane { .. }
            | V128Load32Lane { .. }
            | V128Load64Lane { .. }
            | V128Store8Lane { .. }
            | V128Store16Lane { .. }
            | V128Store32Lane { .. }
            | V128Store64Lane { .. }
            | V128Const { .. }
            | I8x16Shuffle { .. }
            | I8x16ExtractLaneS { .. }
            | I8x16ExtractLaneU { .. }
            | I8x16ReplaceLane { .. }
            | I16x8ExtractLaneS { .. }
            | I16x8ExtractLaneU { .. }
            | I16x8ReplaceLane { .. }
            | I32x4ExtractLane { .. }
            | I32x4ReplaceLane { .. }
            | I64x2ExtractLane { .. }
            | I64x2ReplaceLane { .. }
            | F32x4ExtractLane { .. }
            | F32x4ReplaceLane { .. }
            | F64x2ExtractLane { .. }
            | F64x2ReplaceLane { .. }
            | I8x16Swizzle { .. }
            | I8x16Splat { .. }
            | I16x8Splat { .. }
            | I32x4Splat { .. }
            | I64x2Splat { .. }
            | F32x4Splat { .. }
            | F64x2Splat { .. }
            | I8x16Eq { .. }
            | I8x16Ne { .. }
            | I8x16LtS { .. }
            | I8x16LtU { .. }
            | I8x16GtS { .. }
            | I8x16GtU { .. }
            | I8x16LeS { .. }
            | I8x16LeU { .. }
            | I8x16GeS { .. }
            | I8x16GeU { .. }
            | I16x8Eq { .. }
            | I16x8Ne { .. }
            | I16x8LtS { .. }
            | I16x8LtU { .. }
            | I16x8GtS { .. }
            | I16x8GtU { .. }
            | I16x8LeS { .. }
            | I16x8LeU { .. }
            | I16x8GeS { .. }
            | I16x8GeU { .. }
            | I32x4Eq { .. }
            | I32x4Ne { .. }
            | I32x4LtS { .. }
            | I32x4LtU { .. }
            | I32x4GtS { .. }
            | I32x4GtU { .. }
            | I32x4LeS { .. }
            | I32x4LeU { .. }
            | I32x4GeS { .. }
            | I32x4GeU { .. }
            | I64x2Eq { .. }
            | I64x2Ne { .. }
            | I64x2LtS { .. }
            | I64x2GtS { .. }
            | I64x2LeS { .. }
            | I64x2GeS { .. }
            | F32x4Eq { .. }
            | F32x4Ne { .. }
            | F32x4Lt { .. }
            | F32x4Gt { .. }
            | F32x4Le { .. }
            | F32x4Ge { .. }
            | F64x2Eq { .. }
            | F64x2Ne { .. }
            | F64x2Lt { .. }
            | F64x2Gt { .. }
            | F64x2Le { .. }
            | F64x2Ge { .. }
            | V128Not { .. }
            | V128And { .. }
            | V128AndNot { .. }
            | V128Or { .. }
            | V128Xor { .. }
            | V128Bitselect { .. }
            | V128AnyTrue { .. }
            | I8x16Abs { .. }
            | I8x16Neg { .. }
            | I8x16Popcnt { .. }
            | I8x16AllTrue { .. }
            | I8x16Bitmask { .. }
            | I8x16NarrowI16x8S { .. }
            | I8x16NarrowI16x8U { .. }
            | I8x16Shl { .. }
            | I8x16ShrS { .. }
            | I8x16ShrU { .. }
            | I8x16Add { .. }
            | I8x16AddSatS { .. }
            | I8x16AddSatU { .. }
            | I8x16Sub { .. }
            | I8x16SubSatS { .. }
            | I8x16SubSatU { .. }
            | I8x16MinS { .. }
            | I8x16MinU { .. }
            | I8x16MaxS { .. }
            | I8x16MaxU { .. }
            | I8x16AvgrU { .. }
            | I16x8ExtAddPairwiseI8x16S { .. }
            | I16x8ExtAddPairwiseI8x16U { .. }
            | I16x8Abs { .. }
            | I16x8Neg { .. }
            | I16x8Q15MulrSatS { .. }
            | I16x8AllTrue { .. }
            | I16x8Bitmask { .. }
            | I16x8NarrowI32x4S { .. }
            | I16x8NarrowI32x4U { .. }
            | I16x8ExtendLowI8x16S { .. }
            | I16x8ExtendHighI8x16S { .. }
            | I16x8ExtendLowI8x16U { .. }
            | I16x8ExtendHighI8x16U { .. }
            | I16x8Shl { .. }
            | I16x8ShrS { .. }
            | I16x8ShrU { .. }
            | I16x8Add { .. }
            | I16x8AddSatS { .. }
            | I16x8AddSatU { .. }
            | I16x8Sub { .. }
            | I16x8SubSatS { .. }
            | I16x8SubSatU { .. }
            | I16x8Mul { .. }
            | I16x8MinS { .. }
            | I16x8MinU { .. }
            | I16x8MaxS { .. }
            | I16x8MaxU { .. }
            | I16x8AvgrU { .. }
            | I16x8ExtMulLowI8x16S { .. }
            | I16x8ExtMulHighI8x16S { .. }
            | I16x8ExtMulLowI8x16U { .. }
            | I16x8ExtMulHighI8x16U { .. }
            | I32x4ExtAddPairwiseI16x8S { .. }
            | I32x4ExtAddPairwiseI16x8U { .. }
            | I32x4Abs { .. }
            | I32x4Neg { .. }
            | I32x4AllTrue { .. }
            | I32x4Bitmask { .. }
            | I32x4ExtendLowI16x8S { .. }
            | I32x4ExtendHighI16x8S { .. }
            | I32x4ExtendLowI16x8U { .. }
            | I32x4ExtendHighI16x8U { .. }
            | I32x4Shl { .. }
            | I32x4ShrS { .. }
            | I32x4ShrU { .. }
            | I32x4Add { .. }
            | I32x4Sub { .. }
            | I32x4Mul { .. }
            | I32x4MinS { .. }
            | I32x4MinU { .. }
            | I32x4MaxS { .. }
            | I32x4MaxU { .. }
            | I32x4DotI16x8S { .. }
            | I32x4ExtMulLowI16x8S { .. }
            | I32x4ExtMulHighI16x8S { .. }
            | I32x4ExtMulLowI16x8U { .. }
            | I32x4ExtMulHighI16x8U { .. }
            | I64x2Abs { .. }
            | I64x2Neg { .. }
            | I64x2AllTrue { .. }
            | I64x2Bitmask { .. }
            | I64x2ExtendLowI32x4S { .. }
            | I64x2ExtendHighI32x4S { .. }
            | I64x2ExtendLowI32x4U { .. }
            | I64x2ExtendHighI32x4U { .. }
            | I64x2Shl { .. }
            | I64x2ShrS { .. }
            | I64x2ShrU { .. }
            | I64x2Add { .. }
            | I64x2Sub { .. }
            | I64x2Mul { .. }
            | I64x2ExtMulLowI32x4S { .. }
            | I64x2ExtMulHighI32x4S { .. }
            | I64x2ExtMulLowI32x4U { .. }
            | I64x2ExtMulHighI32x4U { .. }
            | F32x4Ceil { .. }
            | F32x4Floor { .. }
            | F32x4Trunc { .. }
            | F32x4Nearest { .. }
            | F32x4Abs { .. }
            | F32x4Neg { .. }
            | F32x4Sqrt { .. }
            | F32x4Add { .. }
            | F32x4Sub { .. }
            | F32x4Mul { .. }
            | F32x4Div { .. }
            | F32x4Min { .. }
            | F32x4Max { .. }
            | F32x4PMin { .. }
            | F32x4PMax { .. }
            | F64x2Ceil { .. }
            | F64x2Floor { .. }
            | F64x2Trunc { .. }
            | F64x2Nearest { .. }
            | F64x2Abs { .. }
            | F64x2Neg { .. }
            | F64x2Sqrt { .. }
            | F64x2Add { .. }
            | F64x2Sub { .. }
            | F64x2Mul { .. }
            | F64x2Div { .. }
            | F64x2Min { .. }
            | F64x2Max { .. }
            | F64x2PMin { .. }
            | F64x2PMax { .. }
            | I32x4TruncSatF32x4S { .. }
            | I32x4TruncSatF32x4U { .. }
            | F32x4ConvertI32x4S { .. }
            | F32x4ConvertI32x4U { .. }
            | I32x4TruncSatF64x2SZero { .. }
            | I32x4TruncSatF64x2UZero { .. }
            | F64x2ConvertLowI32x4S { .. }
            | F64x2ConvertLowI32x4U { .. }
            | F32x4DemoteF64x2Zero { .. }
            | F64x2PromoteLowF32x4 { .. }

            // tail-call proposal
            | ReturnCall { .. }
            | ReturnCallIndirect { .. }

            // Threads proposal
            | MemoryAtomicNotify { .. }
            | MemoryAtomicWait32 { .. }
            | MemoryAtomicWait64 { .. }
            | AtomicFence { .. }
            | I32AtomicLoad { .. }
            | I64AtomicLoad { .. }
            | I32AtomicLoad8U { .. }
            | I32AtomicLoad16U { .. }
            | I64AtomicLoad8U { .. }
            | I64AtomicLoad16U { .. }
            | I64AtomicLoad32U { .. }
            | I32AtomicStore { .. }
            | I64AtomicStore { .. }
            | I32AtomicStore8 { .. }
            | I32AtomicStore16 { .. }
            | I64AtomicStore8 { .. }
            | I64AtomicStore16 { .. }
            | I64AtomicStore32 { .. }
            | I32AtomicRmwAdd { .. }
            | I64AtomicRmwAdd { .. }
            | I32AtomicRmw8AddU { .. }
            | I32AtomicRmw16AddU { .. }
            | I64AtomicRmw8AddU { .. }
            | I64AtomicRmw16AddU { .. }
            | I64AtomicRmw32AddU { .. }
            | I32AtomicRmwSub { .. }
            | I64AtomicRmwSub { .. }
            | I32AtomicRmw8SubU { .. }
            | I32AtomicRmw16SubU { .. }
            | I64AtomicRmw8SubU { .. }
            | I64AtomicRmw16SubU { .. }
            | I64AtomicRmw32SubU { .. }
            | I32AtomicRmwAnd { .. }
            | I64AtomicRmwAnd { .. }
            | I32AtomicRmw8AndU { .. }
            | I32AtomicRmw16AndU { .. }
            | I64AtomicRmw8AndU { .. }
            | I64AtomicRmw16AndU { .. }
            | I64AtomicRmw32AndU { .. }
            | I32AtomicRmwOr { .. }
            | I64AtomicRmwOr { .. }
            | I32AtomicRmw8OrU { .. }
            | I32AtomicRmw16OrU { .. }
            | I64AtomicRmw8OrU { .. }
            | I64AtomicRmw16OrU { .. }
            | I64AtomicRmw32OrU { .. }
            | I32AtomicRmwXor { .. }
            | I64AtomicRmwXor { .. }
            | I32AtomicRmw8XorU { .. }
            | I32AtomicRmw16XorU { .. }
            | I64AtomicRmw8XorU { .. }
            | I64AtomicRmw16XorU { .. }
            | I64AtomicRmw32XorU { .. }
            | I32AtomicRmwXchg { .. }
            | I64AtomicRmwXchg { .. }
            | I32AtomicRmw8XchgU { .. }
            | I32AtomicRmw16XchgU { .. }
            | I64AtomicRmw8XchgU { .. }
            | I64AtomicRmw16XchgU { .. }
            | I64AtomicRmw32XchgU { .. }
            | I32AtomicRmwCmpxchg { .. }
            | I64AtomicRmwCmpxchg { .. }
            | I32AtomicRmw8CmpxchgU { .. }
            | I32AtomicRmw16CmpxchgU { .. }
            | I64AtomicRmw8CmpxchgU { .. }
            | I64AtomicRmw16CmpxchgU { .. }
            | I64AtomicRmw32CmpxchgU { .. } =>
                todo!("instruction {:?} not covered", instruction),
        }
    }

    fn memory_grow_cost(&self) -> MemoryGrowCost {
        // Per Substrate documentation, the `memory.grow` instruction cost is from benchmarks using MAX page size.
        // Similarly, Radix Engine enforces `DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME`.
        // Thus, no additional costing is applied.
        MemoryGrowCost::Free
    }

    fn call_per_local_cost(&self) -> u32 {
        self.weights.call_per_local
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_params() {
        assert_eq!(format!("{:?}", WasmValidatorConfigV1::new()), "WasmValidatorConfigV1 { weights: InstructionWeights { version: 4, fallback: 0, i64const: 1372, i64load: 3597, i64store: 3905, select: 3434, if: 8054, br: 3529, br_if: 4706, br_table: 8198, br_table_per_entry: 29, call: 14340, call_indirect: 19936, call_per_local: 1651, local_get: 2816, local_set: 2822, local_tee: 2087, global_get: 7002, global_set: 7806, memory_size: 2555, memory_grow: 14764221, i64clz: 1509, i64ctz: 2035, i64popcnt: 1499, i64eqz: 1889, i64extendsi32: 1478, i64extendui32: 1939, i32wrapi64: 1505, i64eq: 2149, i64ne: 1628, i64lts: 1654, i64ltu: 2088, i64gts: 2205, i64gtu: 1661, i64les: 1648, i64leu: 2135, i64ges: 2226, i64geu: 1661, i64add: 1623, i64sub: 2212, i64mul: 1640, i64divs: 2678, i64divu: 1751, i64rems: 2659, i64remu: 1681, i64and: 2045, i64or: 1641, i64xor: 2196, i64shl: 1662, i64shrs: 2124, i64shru: 1646, i64rotl: 1658, i64rotr: 2062 }, max_stack_size: 1024 }")
    }
}
