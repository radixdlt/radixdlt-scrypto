use parity_wasm::elements::Instruction::{self, *};
use wasm_instrument::gas_metering::MemoryGrowCost;
use wasm_instrument::gas_metering::Rules;

use crate::types::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, ScryptoSbor)]
pub struct InstructionCostRules {
    pub inst_unreachable: u32,
    pub inst_nop: u32,
    pub inst_block: u32,
    pub inst_loop: u32,
    pub inst_if: u32,
    pub inst_else: u32,
    pub inst_end: u32,
    pub inst_br: u32,
    pub inst_brif: u32,
    pub inst_brtable: u32,
    pub inst_return: u32,

    pub inst_call: u32,
    pub inst_callindirect: u32,

    pub inst_drop: u32,
    pub inst_select: u32,

    pub inst_getlocal: u32,
    pub inst_setlocal: u32,
    pub inst_teelocal: u32,
    pub inst_getglobal: u32,
    pub inst_setglobal: u32,

    pub inst_i32load: u32,
    pub inst_i64load: u32,
    pub inst_f32load: u32,
    pub inst_f64load: u32,
    pub inst_i32load8s: u32,
    pub inst_i32load8u: u32,
    pub inst_i32load16s: u32,
    pub inst_i32load16u: u32,
    pub inst_i64load8s: u32,
    pub inst_i64load8u: u32,
    pub inst_i64load16s: u32,
    pub inst_i64load16u: u32,
    pub inst_i64load32s: u32,
    pub inst_i64load32u: u32,
    pub inst_i32store: u32,
    pub inst_i64store: u32,
    pub inst_f32store: u32,
    pub inst_f64store: u32,
    pub inst_i32store8: u32,
    pub inst_i32store16: u32,
    pub inst_i64store8: u32,
    pub inst_i64store16: u32,
    pub inst_i64store32: u32,

    pub inst_currentmemory: u32,
    pub inst_growmemory: u32,

    pub inst_i32const: u32,
    pub inst_i64const: u32,
    pub inst_f32const: u32,
    pub inst_f64const: u32,

    pub inst_i32eqz: u32,
    pub inst_i32eq: u32,
    pub inst_i32ne: u32,
    pub inst_i32lts: u32,
    pub inst_i32ltu: u32,
    pub inst_i32gts: u32,
    pub inst_i32gtu: u32,
    pub inst_i32les: u32,
    pub inst_i32leu: u32,
    pub inst_i32ges: u32,
    pub inst_i32geu: u32,

    pub inst_i64eqz: u32,
    pub inst_i64eq: u32,
    pub inst_i64ne: u32,
    pub inst_i64lts: u32,
    pub inst_i64ltu: u32,
    pub inst_i64gts: u32,
    pub inst_i64gtu: u32,
    pub inst_i64les: u32,
    pub inst_i64leu: u32,
    pub inst_i64ges: u32,
    pub inst_i64geu: u32,

    pub inst_f32eq: u32,
    pub inst_f32ne: u32,
    pub inst_f32lt: u32,
    pub inst_f32gt: u32,
    pub inst_f32le: u32,
    pub inst_f32ge: u32,

    pub inst_f64eq: u32,
    pub inst_f64ne: u32,
    pub inst_f64lt: u32,
    pub inst_f64gt: u32,
    pub inst_f64le: u32,
    pub inst_f64ge: u32,

    pub inst_i32clz: u32,
    pub inst_i32ctz: u32,
    pub inst_i32popcnt: u32,
    pub inst_i32add: u32,
    pub inst_i32sub: u32,
    pub inst_i32mul: u32,
    pub inst_i32divs: u32,
    pub inst_i32divu: u32,
    pub inst_i32rems: u32,
    pub inst_i32remu: u32,
    pub inst_i32and: u32,
    pub inst_i32or: u32,
    pub inst_i32xor: u32,
    pub inst_i32shl: u32,
    pub inst_i32shrs: u32,
    pub inst_i32shru: u32,
    pub inst_i32rotl: u32,
    pub inst_i32rotr: u32,

    pub inst_i64clz: u32,
    pub inst_i64ctz: u32,
    pub inst_i64popcnt: u32,
    pub inst_i64add: u32,
    pub inst_i64sub: u32,
    pub inst_i64mul: u32,
    pub inst_i64divs: u32,
    pub inst_i64divu: u32,
    pub inst_i64rems: u32,
    pub inst_i64remu: u32,
    pub inst_i64and: u32,
    pub inst_i64or: u32,
    pub inst_i64xor: u32,
    pub inst_i64shl: u32,
    pub inst_i64shrs: u32,
    pub inst_i64shru: u32,
    pub inst_i64rotl: u32,
    pub inst_i64rotr: u32,
    pub inst_f32abs: u32,
    pub inst_f32neg: u32,
    pub inst_f32ceil: u32,
    pub inst_f32floor: u32,
    pub inst_f32trunc: u32,
    pub inst_f32nearest: u32,
    pub inst_f32sqrt: u32,
    pub inst_f32add: u32,
    pub inst_f32sub: u32,
    pub inst_f32mul: u32,
    pub inst_f32div: u32,
    pub inst_f32min: u32,
    pub inst_f32max: u32,
    pub inst_f32copysign: u32,
    pub inst_f64abs: u32,
    pub inst_f64neg: u32,
    pub inst_f64ceil: u32,
    pub inst_f64floor: u32,
    pub inst_f64trunc: u32,
    pub inst_f64nearest: u32,
    pub inst_f64sqrt: u32,
    pub inst_f64add: u32,
    pub inst_f64sub: u32,
    pub inst_f64mul: u32,
    pub inst_f64div: u32,
    pub inst_f64min: u32,
    pub inst_f64max: u32,
    pub inst_f64copysign: u32,

    pub inst_i32wrapi64: u32,
    pub inst_i32truncsf32: u32,
    pub inst_i32truncuf32: u32,
    pub inst_i32truncsf64: u32,
    pub inst_i32truncuf64: u32,
    pub inst_i64extendsi32: u32,
    pub inst_i64extendui32: u32,
    pub inst_i64truncsf32: u32,
    pub inst_i64truncuf32: u32,
    pub inst_i64truncsf64: u32,
    pub inst_i64truncuf64: u32,
    pub inst_f32convertsi32: u32,
    pub inst_f32convertui32: u32,
    pub inst_f32convertsi64: u32,
    pub inst_f32convertui64: u32,
    pub inst_f32demotef64: u32,
    pub inst_f64convertsi32: u32,
    pub inst_f64convertui32: u32,
    pub inst_f64convertsi64: u32,
    pub inst_f64convertui64: u32,
    pub inst_f64promotef32: u32,

    pub inst_i32reinterpretf32: u32,
    pub inst_i64reinterpretf64: u32,
    pub inst_f32reinterpreti32: u32,
    pub inst_f64reinterpreti64: u32,

    pub inst_signext: u32,

    pub grow_memory_cost: u32,
}

impl Rules for InstructionCostRules {
    fn instruction_cost(&self, instruction: &Instruction) -> Option<u32> {
        Some(match instruction {
            Unreachable => self.inst_unreachable,
            Nop => self.inst_nop,
            Block(_) => self.inst_block,
            Loop(_) => self.inst_loop,
            If(_) => self.inst_if,
            Else => self.inst_else,
            End => self.inst_end,
            Br(_) => self.inst_br,
            BrIf(_) => self.inst_brif,
            BrTable(_) => self.inst_brtable,
            Return => self.inst_return,

            Call(_) => self.inst_call,
            CallIndirect(_, _) => self.inst_callindirect,

            Drop => self.inst_drop,
            Select => self.inst_select,

            GetLocal(_) => self.inst_getlocal,
            SetLocal(_) => self.inst_setlocal,
            TeeLocal(_) => self.inst_teelocal,
            GetGlobal(_) => self.inst_getglobal,
            SetGlobal(_) => self.inst_setglobal,

            I32Load(_, _) => self.inst_i32load,
            I64Load(_, _) => self.inst_i64load,
            F32Load(_, _) => self.inst_f32load,
            F64Load(_, _) => self.inst_f64load,
            I32Load8S(_, _) => self.inst_i32load8s,
            I32Load8U(_, _) => self.inst_i32load8u,
            I32Load16S(_, _) => self.inst_i32load16s,
            I32Load16U(_, _) => self.inst_i32load16u,
            I64Load8S(_, _) => self.inst_i64load8s,
            I64Load8U(_, _) => self.inst_i64load8u,
            I64Load16S(_, _) => self.inst_i64load16s,
            I64Load16U(_, _) => self.inst_i64load16u,
            I64Load32S(_, _) => self.inst_i64load32s,
            I64Load32U(_, _) => self.inst_i64load32u,
            I32Store(_, _) => self.inst_i32store,
            I64Store(_, _) => self.inst_i64store,
            F32Store(_, _) => self.inst_f32store,
            F64Store(_, _) => self.inst_f64store,
            I32Store8(_, _) => self.inst_i32store8,
            I32Store16(_, _) => self.inst_i32store16,
            I64Store8(_, _) => self.inst_i64store8,
            I64Store16(_, _) => self.inst_i64store16,
            I64Store32(_, _) => self.inst_i64store32,

            CurrentMemory(_) => self.inst_currentmemory,
            GrowMemory(_) => self.inst_growmemory,

            I32Const(_) => self.inst_i32const,
            I64Const(_) => self.inst_i64const,
            F32Const(_) => self.inst_f32const,
            F64Const(_) => self.inst_f64const,

            I32Eqz => self.inst_i32eqz,
            I32Eq => self.inst_i32eq,
            I32Ne => self.inst_i32ne,
            I32LtS => self.inst_i32lts,
            I32LtU => self.inst_i32ltu,
            I32GtS => self.inst_i32gts,
            I32GtU => self.inst_i32gtu,
            I32LeS => self.inst_i32les,
            I32LeU => self.inst_i32leu,
            I32GeS => self.inst_i32ges,
            I32GeU => self.inst_i32geu,

            I64Eqz => self.inst_i64eqz,
            I64Eq => self.inst_i64eq,
            I64Ne => self.inst_i64ne,
            I64LtS => self.inst_i64lts,
            I64LtU => self.inst_i64ltu,
            I64GtS => self.inst_i64gts,
            I64GtU => self.inst_i64gtu,
            I64LeS => self.inst_i64les,
            I64LeU => self.inst_i64leu,
            I64GeS => self.inst_i64ges,
            I64GeU => self.inst_i64geu,

            F32Eq => self.inst_f32eq,
            F32Ne => self.inst_f32ne,
            F32Lt => self.inst_f32lt,
            F32Gt => self.inst_f32gt,
            F32Le => self.inst_f32le,
            F32Ge => self.inst_f32ge,

            F64Eq => self.inst_f64eq,
            F64Ne => self.inst_f64ne,
            F64Lt => self.inst_f64lt,
            F64Gt => self.inst_f64gt,
            F64Le => self.inst_f64le,
            F64Ge => self.inst_f64ge,

            I32Clz => self.inst_i32clz,
            I32Ctz => self.inst_i32ctz,
            I32Popcnt => self.inst_i32popcnt,
            I32Add => self.inst_i32add,
            I32Sub => self.inst_i32sub,
            I32Mul => self.inst_i32mul,
            I32DivS => self.inst_i32divs,
            I32DivU => self.inst_i32divu,
            I32RemS => self.inst_i32rems,
            I32RemU => self.inst_i32remu,
            I32And => self.inst_i32and,
            I32Or => self.inst_i32or,
            I32Xor => self.inst_i32xor,
            I32Shl => self.inst_i32shl,
            I32ShrS => self.inst_i32shrs,
            I32ShrU => self.inst_i32shru,
            I32Rotl => self.inst_i32rotl,
            I32Rotr => self.inst_i32rotr,

            I64Clz => self.inst_i64clz,
            I64Ctz => self.inst_i64ctz,
            I64Popcnt => self.inst_i64popcnt,
            I64Add => self.inst_i64add,
            I64Sub => self.inst_i64sub,
            I64Mul => self.inst_i64mul,
            I64DivS => self.inst_i64divs,
            I64DivU => self.inst_i64divu,
            I64RemS => self.inst_i64rems,
            I64RemU => self.inst_i64remu,
            I64And => self.inst_i64and,
            I64Or => self.inst_i64or,
            I64Xor => self.inst_i64xor,
            I64Shl => self.inst_i64shl,
            I64ShrS => self.inst_i64shrs,
            I64ShrU => self.inst_i64shru,
            I64Rotl => self.inst_i64rotl,
            I64Rotr => self.inst_i64rotr,
            F32Abs => self.inst_f32abs,
            F32Neg => self.inst_f32neg,
            F32Ceil => self.inst_f32ceil,
            F32Floor => self.inst_f32floor,
            F32Trunc => self.inst_f32trunc,
            F32Nearest => self.inst_f32nearest,
            F32Sqrt => self.inst_f32sqrt,
            F32Add => self.inst_f32add,
            F32Sub => self.inst_f32sub,
            F32Mul => self.inst_f32mul,
            F32Div => self.inst_f32div,
            F32Min => self.inst_f32min,
            F32Max => self.inst_f32max,
            F32Copysign => self.inst_f32copysign,
            F64Abs => self.inst_f64abs,
            F64Neg => self.inst_f64neg,
            F64Ceil => self.inst_f64ceil,
            F64Floor => self.inst_f64floor,
            F64Trunc => self.inst_f64trunc,
            F64Nearest => self.inst_f64nearest,
            F64Sqrt => self.inst_f64sqrt,
            F64Add => self.inst_f64add,
            F64Sub => self.inst_f64sub,
            F64Mul => self.inst_f64mul,
            F64Div => self.inst_f64div,
            F64Min => self.inst_f64min,
            F64Max => self.inst_f64max,
            F64Copysign => self.inst_f64copysign,

            I32WrapI64 => self.inst_i32wrapi64,
            I32TruncSF32 => self.inst_i32truncsf32,
            I32TruncUF32 => self.inst_i32truncuf32,
            I32TruncSF64 => self.inst_i32truncsf64,
            I32TruncUF64 => self.inst_i32truncuf64,
            I64ExtendSI32 => self.inst_i64extendsi32,
            I64ExtendUI32 => self.inst_i64extendui32,
            I64TruncSF32 => self.inst_i64truncsf32,
            I64TruncUF32 => self.inst_i64truncuf32,
            I64TruncSF64 => self.inst_i64truncsf64,
            I64TruncUF64 => self.inst_i64truncuf64,
            F32ConvertSI32 => self.inst_f32convertsi32,
            F32ConvertUI32 => self.inst_f32convertui32,
            F32ConvertSI64 => self.inst_f32convertsi64,
            F32ConvertUI64 => self.inst_f32convertui64,
            F32DemoteF64 => self.inst_f32demotef64,
            F64ConvertSI32 => self.inst_f64convertsi32,
            F64ConvertUI32 => self.inst_f64convertui32,
            F64ConvertSI64 => self.inst_f64convertsi64,
            F64ConvertUI64 => self.inst_f64convertui64,
            F64PromoteF32 => self.inst_f64promotef32,

            I32ReinterpretF32 => self.inst_i32reinterpretf32,
            I64ReinterpretF64 => self.inst_i64reinterpretf64,
            F32ReinterpretI32 => self.inst_f32reinterpreti32,
            F64ReinterpretI64 => self.inst_f64reinterpreti64,

            SignExt(_) => self.inst_signext,
        })
    }

    fn memory_grow_cost(&self) -> MemoryGrowCost {
        MemoryGrowCost::Linear(
            NonZeroU32::new(self.grow_memory_cost).expect("GROW_MEMORY_COST value is zero"),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::vm::wasm::{WasmInstrumenterConfig, WasmModule};
    use wabt::wat2wasm;

    #[test]
    fn test_cost_rules() {
        let code = wat2wasm(
            r#"
            (module
                (func (param $p0 i32) (result i32)
                    local.get $p0
                    i32.const 5
                    i32.mul
                )
                (func (param $p0 i32) (result i32)
                    local.get $p0
                    call 0
                )
            )
            "#,
        )
        .unwrap();
        let config = WasmInstrumenterConfig::v1();
        let transformed = WasmModule::init(&code)
            .unwrap()
            .inject_instruction_metering(config.instruction_cost_rules())
            .unwrap()
            .to_bytes()
            .unwrap()
            .0;

        // Costs:
        // 3 = 1 (local.get) + 1 (i32.const) + 1 (i32.mul)
        // 2 = 1 (local.get) + 1 (call)
        let expected = wat2wasm(
            r#"
            (module
                (type (;0;) (func (param i32) (result i32)))
                (type (;1;) (func (param i32)))
                (import "env" "gas" (func (;0;) (type 1)))
                (func (;1;) (type 0) (param i32) (result i32)
                  i32.const 3
                  call 0
                  local.get 0
                  i32.const 5
                  i32.mul)
                (func (;2;) (type 0) (param i32) (result i32)
                  i32.const 2
                  call 0
                  local.get 0
                  call 1))
            "#,
        )
        .unwrap();

        assert_eq!(transformed, expected);
    }
}
