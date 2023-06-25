use parity_wasm::elements::Instruction::{self, *};
use wasm_instrument::gas_metering::MemoryGrowCost;
use wasm_instrument::gas_metering::Rules;

use crate::types::*;

#[derive(Debug, Clone, Sbor)]
pub struct InstructionCostRules {
    instruction_cost: u32,
    grow_memory_cost: u32,
}

impl InstructionCostRules {
    pub fn new() -> Self {
        // FIXME: below are wrong numbers!!!!
        Self {
            instruction_cost: 1,
            grow_memory_cost: 10,
        }
    }
}

impl Rules for InstructionCostRules {
    fn instruction_cost(&self, instruction: &Instruction) -> Option<u32> {
        Some(match instruction {
            Unreachable => self.instruction_cost,
            Nop => self.instruction_cost,
            Block(_) => self.instruction_cost,
            Loop(_) => self.instruction_cost,
            If(_) => self.instruction_cost,
            Else => self.instruction_cost,
            End => self.instruction_cost,
            Br(_) => self.instruction_cost,
            BrIf(_) => self.instruction_cost,
            BrTable(_) => self.instruction_cost,
            Return => self.instruction_cost,

            Call(_) => self.instruction_cost,
            CallIndirect(_, _) => self.instruction_cost,

            Drop => self.instruction_cost,
            Select => self.instruction_cost,

            GetLocal(_) => self.instruction_cost,
            SetLocal(_) => self.instruction_cost,
            TeeLocal(_) => self.instruction_cost,
            GetGlobal(_) => self.instruction_cost,
            SetGlobal(_) => self.instruction_cost,

            I32Load(_, _) => self.instruction_cost,
            I64Load(_, _) => self.instruction_cost,
            F32Load(_, _) => self.instruction_cost,
            F64Load(_, _) => self.instruction_cost,
            I32Load8S(_, _) => self.instruction_cost,
            I32Load8U(_, _) => self.instruction_cost,
            I32Load16S(_, _) => self.instruction_cost,
            I32Load16U(_, _) => self.instruction_cost,
            I64Load8S(_, _) => self.instruction_cost,
            I64Load8U(_, _) => self.instruction_cost,
            I64Load16S(_, _) => self.instruction_cost,
            I64Load16U(_, _) => self.instruction_cost,
            I64Load32S(_, _) => self.instruction_cost,
            I64Load32U(_, _) => self.instruction_cost,
            I32Store(_, _) => self.instruction_cost,
            I64Store(_, _) => self.instruction_cost,
            F32Store(_, _) => self.instruction_cost,
            F64Store(_, _) => self.instruction_cost,
            I32Store8(_, _) => self.instruction_cost,
            I32Store16(_, _) => self.instruction_cost,
            I64Store8(_, _) => self.instruction_cost,
            I64Store16(_, _) => self.instruction_cost,
            I64Store32(_, _) => self.instruction_cost,

            CurrentMemory(_) => self.instruction_cost,
            GrowMemory(_) => self.instruction_cost,

            I32Const(_) => self.instruction_cost,
            I64Const(_) => self.instruction_cost,
            F32Const(_) => self.instruction_cost,
            F64Const(_) => self.instruction_cost,

            I32Eqz => self.instruction_cost,
            I32Eq => self.instruction_cost,
            I32Ne => self.instruction_cost,
            I32LtS => self.instruction_cost,
            I32LtU => self.instruction_cost,
            I32GtS => self.instruction_cost,
            I32GtU => self.instruction_cost,
            I32LeS => self.instruction_cost,
            I32LeU => self.instruction_cost,
            I32GeS => self.instruction_cost,
            I32GeU => self.instruction_cost,

            I64Eqz => self.instruction_cost,
            I64Eq => self.instruction_cost,
            I64Ne => self.instruction_cost,
            I64LtS => self.instruction_cost,
            I64LtU => self.instruction_cost,
            I64GtS => self.instruction_cost,
            I64GtU => self.instruction_cost,
            I64LeS => self.instruction_cost,
            I64LeU => self.instruction_cost,
            I64GeS => self.instruction_cost,
            I64GeU => self.instruction_cost,

            F32Eq => self.instruction_cost,
            F32Ne => self.instruction_cost,
            F32Lt => self.instruction_cost,
            F32Gt => self.instruction_cost,
            F32Le => self.instruction_cost,
            F32Ge => self.instruction_cost,

            F64Eq => self.instruction_cost,
            F64Ne => self.instruction_cost,
            F64Lt => self.instruction_cost,
            F64Gt => self.instruction_cost,
            F64Le => self.instruction_cost,
            F64Ge => self.instruction_cost,

            I32Clz => self.instruction_cost,
            I32Ctz => self.instruction_cost,
            I32Popcnt => self.instruction_cost,
            I32Add => self.instruction_cost,
            I32Sub => self.instruction_cost,
            I32Mul => self.instruction_cost,
            I32DivS => self.instruction_cost,
            I32DivU => self.instruction_cost,
            I32RemS => self.instruction_cost,
            I32RemU => self.instruction_cost,
            I32And => self.instruction_cost,
            I32Or => self.instruction_cost,
            I32Xor => self.instruction_cost,
            I32Shl => self.instruction_cost,
            I32ShrS => self.instruction_cost,
            I32ShrU => self.instruction_cost,
            I32Rotl => self.instruction_cost,
            I32Rotr => self.instruction_cost,

            I64Clz => self.instruction_cost,
            I64Ctz => self.instruction_cost,
            I64Popcnt => self.instruction_cost,
            I64Add => self.instruction_cost,
            I64Sub => self.instruction_cost,
            I64Mul => self.instruction_cost,
            I64DivS => self.instruction_cost,
            I64DivU => self.instruction_cost,
            I64RemS => self.instruction_cost,
            I64RemU => self.instruction_cost,
            I64And => self.instruction_cost,
            I64Or => self.instruction_cost,
            I64Xor => self.instruction_cost,
            I64Shl => self.instruction_cost,
            I64ShrS => self.instruction_cost,
            I64ShrU => self.instruction_cost,
            I64Rotl => self.instruction_cost,
            I64Rotr => self.instruction_cost,
            F32Abs => self.instruction_cost,
            F32Neg => self.instruction_cost,
            F32Ceil => self.instruction_cost,
            F32Floor => self.instruction_cost,
            F32Trunc => self.instruction_cost,
            F32Nearest => self.instruction_cost,
            F32Sqrt => self.instruction_cost,
            F32Add => self.instruction_cost,
            F32Sub => self.instruction_cost,
            F32Mul => self.instruction_cost,
            F32Div => self.instruction_cost,
            F32Min => self.instruction_cost,
            F32Max => self.instruction_cost,
            F32Copysign => self.instruction_cost,
            F64Abs => self.instruction_cost,
            F64Neg => self.instruction_cost,
            F64Ceil => self.instruction_cost,
            F64Floor => self.instruction_cost,
            F64Trunc => self.instruction_cost,
            F64Nearest => self.instruction_cost,
            F64Sqrt => self.instruction_cost,
            F64Add => self.instruction_cost,
            F64Sub => self.instruction_cost,
            F64Mul => self.instruction_cost,
            F64Div => self.instruction_cost,
            F64Min => self.instruction_cost,
            F64Max => self.instruction_cost,
            F64Copysign => self.instruction_cost,

            I32WrapI64 => self.instruction_cost,
            I32TruncSF32 => self.instruction_cost,
            I32TruncUF32 => self.instruction_cost,
            I32TruncSF64 => self.instruction_cost,
            I32TruncUF64 => self.instruction_cost,
            I64ExtendSI32 => self.instruction_cost,
            I64ExtendUI32 => self.instruction_cost,
            I64TruncSF32 => self.instruction_cost,
            I64TruncUF32 => self.instruction_cost,
            I64TruncSF64 => self.instruction_cost,
            I64TruncUF64 => self.instruction_cost,
            F32ConvertSI32 => self.instruction_cost,
            F32ConvertUI32 => self.instruction_cost,
            F32ConvertSI64 => self.instruction_cost,
            F32ConvertUI64 => self.instruction_cost,
            F32DemoteF64 => self.instruction_cost,
            F64ConvertSI32 => self.instruction_cost,
            F64ConvertUI32 => self.instruction_cost,
            F64ConvertSI64 => self.instruction_cost,
            F64ConvertUI64 => self.instruction_cost,
            F64PromoteF32 => self.instruction_cost,

            I32ReinterpretF32 => self.instruction_cost,
            I64ReinterpretF64 => self.instruction_cost,
            F32ReinterpretI32 => self.instruction_cost,
            F64ReinterpretI64 => self.instruction_cost,

            SignExt(_) => self.instruction_cost,
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
    use super::*;
    use crate::vm::wasm::WasmModule;
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
        let rules = InstructionCostRules::new();
        let transformed = WasmModule::init(&code)
            .unwrap()
            .inject_instruction_metering(&rules)
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
