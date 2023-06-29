use crate::types::*;
use parity_wasm::elements::Instruction::{self, *};
use wasm_instrument::gas_metering::MemoryGrowCost;
use wasm_instrument::gas_metering::Rules;

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
    fn instruction_cost(&self, instruction: &Instruction) -> Option<u32> {
        match instruction {
            Unreachable => Some(0),
            Nop => Some(self.weights.i64const),
            Block(_) => Some(self.weights.i64const),
            Loop(_) => Some(self.weights.i64const),
            If(_) => Some(self.weights.r#if),
            Else => Some(0),
            End => Some(0),
            Br(_) => Some(self.weights.br),
            BrIf(_) => Some(self.weights.br_if),
            BrTable(_) => Some(self.weights.br_table),
            Return => Some(0),
            Call(_) => Some(self.weights.call),
            CallIndirect(_, _) => Some(self.weights.call_indirect),
            Drop => Some(self.weights.i64const),
            Select => Some(self.weights.select),
            GetLocal(_) => Some(self.weights.local_get),
            SetLocal(_) => Some(self.weights.local_set),
            TeeLocal(_) => Some(self.weights.local_tee),
            GetGlobal(_) => Some(self.weights.global_get),
            SetGlobal(_) => Some(self.weights.global_set),
            I32Load(_, _) => Some(self.weights.i64load),
            I64Load(_, _) => Some(self.weights.i64load),
            F32Load(_, _) => None,
            F64Load(_, _) => None,
            I32Load8S(_, _) => Some(self.weights.i64load),
            I32Load8U(_, _) => Some(self.weights.i64load),
            I32Load16S(_, _) => Some(self.weights.i64load),
            I32Load16U(_, _) => Some(self.weights.i64load),
            I64Load8S(_, _) => Some(self.weights.i64load),
            I64Load8U(_, _) => Some(self.weights.i64load),
            I64Load16S(_, _) => Some(self.weights.i64load),
            I64Load16U(_, _) => Some(self.weights.i64load),
            I64Load32S(_, _) => Some(self.weights.i64load),
            I64Load32U(_, _) => Some(self.weights.i64load),
            I32Store(_, _) => Some(self.weights.i64store),
            I64Store(_, _) => Some(self.weights.i64store),
            F32Store(_, _) => None,
            F64Store(_, _) => None,
            I32Store8(_, _) => Some(self.weights.i64store),
            I32Store16(_, _) => Some(self.weights.i64store),
            I64Store8(_, _) => Some(self.weights.i64store),
            I64Store16(_, _) => Some(self.weights.i64store),
            I64Store32(_, _) => Some(self.weights.i64store),
            CurrentMemory(_) => Some(self.weights.memory_current),
            GrowMemory(_) => Some(self.weights.memory_grow),
            I32Const(_) => Some(self.weights.i64const),
            I64Const(_) => Some(self.weights.i64const),
            F32Const(_) => None,
            F64Const(_) => None,
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
            I32TruncSF32 => None,
            I32TruncUF32 => None,
            I32TruncSF64 => None,
            I32TruncUF64 => None,
            I64ExtendSI32 => Some(self.weights.i64extendsi32),
            I64ExtendUI32 => Some(self.weights.i64extendui32),
            I64TruncSF32 => None,
            I64TruncUF32 => None,
            I64TruncSF64 => None,
            I64TruncUF64 => None,
            F32ConvertSI32 => None,
            F32ConvertUI32 => None,
            F32ConvertSI64 => None,
            F32ConvertUI64 => None,
            F32DemoteF64 => None,
            F64ConvertSI32 => None,
            F64ConvertUI32 => None,
            F64ConvertSI64 => None,
            F64ConvertUI64 => None,
            F64PromoteF32 => None,
            I32ReinterpretF32 => None,
            I64ReinterpretF64 => None,
            F32ReinterpretI32 => None,
            F64ReinterpretI64 => None,
            SignExt(_) => Some(self.weights.i64extendsi32),
        }
    }

    fn memory_grow_cost(&self) -> MemoryGrowCost {
        // Per Substrate documentation, the `memory.grow` instruction cost is from benchmarks using MAX page size.
        // Similarly, Radix Engine enforces `DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME`.
        // Thus, no additional costing is applied.
        MemoryGrowCost::Free
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_params() {
        assert_eq!(format!("{:?}", WasmValidatorConfigV1::new()), "WasmValidatorConfigV1 { weights: InstructionWeights { version: 4, fallback: 0, i64const: 1372, i64load: 3597, i64store: 3905, select: 3434, if: 8054, br: 3529, br_if: 4706, br_table: 8198, br_table_per_entry: 29, call: 14340, call_indirect: 19936, call_per_local: 1651, local_get: 2816, local_set: 2822, local_tee: 2087, global_get: 7002, global_set: 7806, memory_current: 2555, memory_grow: 14764221, i64clz: 1509, i64ctz: 2035, i64popcnt: 1499, i64eqz: 1889, i64extendsi32: 1478, i64extendui32: 1939, i32wrapi64: 1505, i64eq: 2149, i64ne: 1628, i64lts: 1654, i64ltu: 2088, i64gts: 2205, i64gtu: 1661, i64les: 1648, i64leu: 2135, i64ges: 2226, i64geu: 1661, i64add: 1623, i64sub: 2212, i64mul: 1640, i64divs: 2678, i64divu: 1751, i64rems: 2659, i64remu: 1681, i64and: 2045, i64or: 1641, i64xor: 2196, i64shl: 1662, i64shrs: 2124, i64shru: 1646, i64rotl: 1658, i64rotr: 2062 }, max_stack_size: 1024 }")
    }
}
