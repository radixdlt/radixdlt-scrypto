use crate::types::*;
use wasm_instrument::gas_metering::MemoryGrowCost;
use wasm_instrument::gas_metering::Rules;
use wasmparser::Operator::{self, *};

use super::InstructionWeights;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmValidatorConfigV1 {
    weights: InstructionWeights,
    max_stack_size: u32,
    call_per_local_cost: u32,
}

impl WasmValidatorConfigV1 {
    pub fn new() -> Self {
        Self {
            weights: InstructionWeights::default(),
            max_stack_size: 1024,
            call_per_local_cost: 1,
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
            //            CurrentMemory { .. } => Some(self.weights.memory_current),
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
            // sign-extension instructions
            I32Extend8S => Some(self.weights.i64extendsi32),
            I32Extend16S => Some(self.weights.i64extendsi32),
            I64Extend8S => Some(self.weights.i64extendsi32),
            I64Extend16S => Some(self.weights.i64extendsi32),
            I64Extend32S => Some(self.weights.i64extendsi32),
            _ => todo!("instruction {:?} not covered", instruction),
        }
    }

    fn memory_grow_cost(&self) -> MemoryGrowCost {
        // Per Substrate documentation, the `memory.grow` instruction cost is from benchmarks using MAX page size.
        // Similarly, Radix Engine enforces `DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME`.
        // Thus, no additional costing is applied.
        MemoryGrowCost::Free
    }

    fn call_per_local_cost(&self) -> u32 {
        self.call_per_local_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_params() {
        assert_eq!(format!("{:?}", WasmValidatorConfigV1::new()), "WasmValidatorConfigV1 { weights: InstructionWeights { version: 4, fallback: 0, i64const: 1372, i64load: 3597, i64store: 3905, select: 3434, if: 8054, br: 3529, br_if: 4706, br_table: 8198, br_table_per_entry: 29, call: 14340, call_indirect: 19936, call_per_local: 1651, local_get: 2816, local_set: 2822, local_tee: 2087, global_get: 7002, global_set: 7806, memory_current: 2555, memory_grow: 14764221, i64clz: 1509, i64ctz: 2035, i64popcnt: 1499, i64eqz: 1889, i64extendsi32: 1478, i64extendui32: 1939, i32wrapi64: 1505, i64eq: 2149, i64ne: 1628, i64lts: 1654, i64ltu: 2088, i64gts: 2205, i64gtu: 1661, i64les: 1648, i64leu: 2135, i64ges: 2226, i64geu: 1661, i64add: 1623, i64sub: 2212, i64mul: 1640, i64divs: 2678, i64divu: 1751, i64rems: 2659, i64remu: 1681, i64and: 2045, i64or: 1641, i64xor: 2196, i64shl: 1662, i64shrs: 2124, i64shru: 1646, i64rotl: 1658, i64rotr: 2062 }, max_stack_size: 1024, call_per_local_cost: 1 }")
    }
}
