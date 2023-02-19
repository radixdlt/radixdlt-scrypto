use parity_wasm::elements::Instruction::{self, *};
use wasm_instrument::gas_metering::MemoryGrowCost;
use wasm_instrument::gas_metering::Rules;

use crate::types::*;

#[derive(Debug, Clone, Sbor)]
pub struct InstructionCostRules {
    tier_1_cost: u32,
    tier_2_cost: u32,
    tier_3_cost: u32,
    grow_memory_cost: u32,
}

impl InstructionCostRules {
    pub fn constant(instruction_cost: u32, grow_memory_cost: u32) -> Self {
        Self {
            tier_1_cost: instruction_cost,
            tier_2_cost: instruction_cost,
            tier_3_cost: instruction_cost,
            grow_memory_cost,
        }
    }

    pub fn tiered(
        tier_1_cost: u32,
        tier_2_cost: u32,
        tier_3_cost: u32,
        grow_memory_cost: u32,
    ) -> Self {
        Self {
            tier_1_cost,
            tier_2_cost,
            tier_3_cost,
            grow_memory_cost,
        }
    }
}

impl Rules for InstructionCostRules {
    fn instruction_cost(&self, instruction: &Instruction) -> Option<u32> {
        // TO BE FINE-TUNED
        Some(match instruction {
            Unreachable => self.tier_1_cost,
            Nop => self.tier_1_cost,
            Block(_) => self.tier_1_cost,
            Loop(_) => self.tier_1_cost,
            If(_) => self.tier_1_cost,
            Else => self.tier_1_cost,
            End => self.tier_1_cost,
            Br(_) => self.tier_1_cost,
            BrIf(_) => self.tier_1_cost,
            BrTable(_) => self.tier_1_cost,
            Return => self.tier_1_cost,

            Call(_) => self.tier_3_cost,
            CallIndirect(_, _) => self.tier_3_cost,

            Drop => self.tier_1_cost,
            Select => self.tier_1_cost,

            GetLocal(_) => self.tier_1_cost,
            SetLocal(_) => self.tier_1_cost,
            TeeLocal(_) => self.tier_1_cost,
            GetGlobal(_) => self.tier_2_cost,
            SetGlobal(_) => self.tier_2_cost,

            I32Load(_, _) => self.tier_1_cost,
            I64Load(_, _) => self.tier_1_cost,
            F32Load(_, _) => self.tier_1_cost,
            F64Load(_, _) => self.tier_1_cost,
            I32Load8S(_, _) => self.tier_1_cost,
            I32Load8U(_, _) => self.tier_1_cost,
            I32Load16S(_, _) => self.tier_1_cost,
            I32Load16U(_, _) => self.tier_1_cost,
            I64Load8S(_, _) => self.tier_1_cost,
            I64Load8U(_, _) => self.tier_1_cost,
            I64Load16S(_, _) => self.tier_1_cost,
            I64Load16U(_, _) => self.tier_1_cost,
            I64Load32S(_, _) => self.tier_1_cost,
            I64Load32U(_, _) => self.tier_1_cost,
            I32Store(_, _) => self.tier_1_cost,
            I64Store(_, _) => self.tier_1_cost,
            F32Store(_, _) => self.tier_1_cost,
            F64Store(_, _) => self.tier_1_cost,
            I32Store8(_, _) => self.tier_1_cost,
            I32Store16(_, _) => self.tier_1_cost,
            I64Store8(_, _) => self.tier_1_cost,
            I64Store16(_, _) => self.tier_1_cost,
            I64Store32(_, _) => self.tier_1_cost,

            CurrentMemory(_) => self.tier_1_cost,
            GrowMemory(_) => self.tier_2_cost,

            I32Const(_) => self.tier_1_cost,
            I64Const(_) => self.tier_1_cost,
            F32Const(_) => self.tier_1_cost,
            F64Const(_) => self.tier_1_cost,

            I32Eqz => self.tier_1_cost,
            I32Eq => self.tier_1_cost,
            I32Ne => self.tier_1_cost,
            I32LtS => self.tier_1_cost,
            I32LtU => self.tier_1_cost,
            I32GtS => self.tier_1_cost,
            I32GtU => self.tier_1_cost,
            I32LeS => self.tier_1_cost,
            I32LeU => self.tier_1_cost,
            I32GeS => self.tier_1_cost,
            I32GeU => self.tier_1_cost,

            I64Eqz => self.tier_1_cost,
            I64Eq => self.tier_1_cost,
            I64Ne => self.tier_1_cost,
            I64LtS => self.tier_1_cost,
            I64LtU => self.tier_1_cost,
            I64GtS => self.tier_1_cost,
            I64GtU => self.tier_1_cost,
            I64LeS => self.tier_1_cost,
            I64LeU => self.tier_1_cost,
            I64GeS => self.tier_1_cost,
            I64GeU => self.tier_1_cost,

            F32Eq => self.tier_1_cost,
            F32Ne => self.tier_1_cost,
            F32Lt => self.tier_1_cost,
            F32Gt => self.tier_1_cost,
            F32Le => self.tier_1_cost,
            F32Ge => self.tier_1_cost,

            F64Eq => self.tier_1_cost,
            F64Ne => self.tier_1_cost,
            F64Lt => self.tier_1_cost,
            F64Gt => self.tier_1_cost,
            F64Le => self.tier_1_cost,
            F64Ge => self.tier_1_cost,

            I32Clz => self.tier_1_cost,
            I32Ctz => self.tier_1_cost,
            I32Popcnt => self.tier_1_cost,
            I32Add => self.tier_1_cost,
            I32Sub => self.tier_1_cost,
            I32Mul => self.tier_1_cost,
            I32DivS => self.tier_1_cost,
            I32DivU => self.tier_1_cost,
            I32RemS => self.tier_1_cost,
            I32RemU => self.tier_1_cost,
            I32And => self.tier_1_cost,
            I32Or => self.tier_1_cost,
            I32Xor => self.tier_1_cost,
            I32Shl => self.tier_1_cost,
            I32ShrS => self.tier_1_cost,
            I32ShrU => self.tier_1_cost,
            I32Rotl => self.tier_1_cost,
            I32Rotr => self.tier_1_cost,

            I64Clz => self.tier_1_cost,
            I64Ctz => self.tier_1_cost,
            I64Popcnt => self.tier_1_cost,
            I64Add => self.tier_1_cost,
            I64Sub => self.tier_1_cost,
            I64Mul => self.tier_1_cost,
            I64DivS => self.tier_1_cost,
            I64DivU => self.tier_1_cost,
            I64RemS => self.tier_1_cost,
            I64RemU => self.tier_1_cost,
            I64And => self.tier_1_cost,
            I64Or => self.tier_1_cost,
            I64Xor => self.tier_1_cost,
            I64Shl => self.tier_1_cost,
            I64ShrS => self.tier_1_cost,
            I64ShrU => self.tier_1_cost,
            I64Rotl => self.tier_1_cost,
            I64Rotr => self.tier_1_cost,
            F32Abs => self.tier_1_cost,
            F32Neg => self.tier_1_cost,
            F32Ceil => self.tier_1_cost,
            F32Floor => self.tier_1_cost,
            F32Trunc => self.tier_1_cost,
            F32Nearest => self.tier_1_cost,
            F32Sqrt => self.tier_1_cost,
            F32Add => self.tier_1_cost,
            F32Sub => self.tier_1_cost,
            F32Mul => self.tier_1_cost,
            F32Div => self.tier_1_cost,
            F32Min => self.tier_1_cost,
            F32Max => self.tier_1_cost,
            F32Copysign => self.tier_1_cost,
            F64Abs => self.tier_1_cost,
            F64Neg => self.tier_1_cost,
            F64Ceil => self.tier_1_cost,
            F64Floor => self.tier_1_cost,
            F64Trunc => self.tier_1_cost,
            F64Nearest => self.tier_1_cost,
            F64Sqrt => self.tier_1_cost,
            F64Add => self.tier_1_cost,
            F64Sub => self.tier_1_cost,
            F64Mul => self.tier_1_cost,
            F64Div => self.tier_1_cost,
            F64Min => self.tier_1_cost,
            F64Max => self.tier_1_cost,
            F64Copysign => self.tier_1_cost,

            I32WrapI64 => self.tier_1_cost,
            I32TruncSF32 => self.tier_1_cost,
            I32TruncUF32 => self.tier_1_cost,
            I32TruncSF64 => self.tier_1_cost,
            I32TruncUF64 => self.tier_1_cost,
            I64ExtendSI32 => self.tier_1_cost,
            I64ExtendUI32 => self.tier_1_cost,
            I64TruncSF32 => self.tier_1_cost,
            I64TruncUF32 => self.tier_1_cost,
            I64TruncSF64 => self.tier_1_cost,
            I64TruncUF64 => self.tier_1_cost,
            F32ConvertSI32 => self.tier_1_cost,
            F32ConvertUI32 => self.tier_1_cost,
            F32ConvertSI64 => self.tier_1_cost,
            F32ConvertUI64 => self.tier_1_cost,
            F32DemoteF64 => self.tier_1_cost,
            F64ConvertSI32 => self.tier_1_cost,
            F64ConvertUI32 => self.tier_1_cost,
            F64ConvertSI64 => self.tier_1_cost,
            F64ConvertUI64 => self.tier_1_cost,
            F64PromoteF32 => self.tier_1_cost,

            I32ReinterpretF32 => self.tier_1_cost,
            I64ReinterpretF64 => self.tier_1_cost,
            F32ReinterpretI32 => self.tier_1_cost,
            F64ReinterpretI64 => self.tier_1_cost,
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
    use crate::wasm::WasmModule;
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
        let rules = InstructionCostRules::tiered(1, 5, 10, 55);
        let transformed = WasmModule::init(&code)
            .unwrap()
            .inject_instruction_metering(&rules)
            .unwrap()
            .to_bytes()
            .unwrap()
            .0;

        // Costs:
        // 12 = 10 (local.get) + 1 (i32.const) + 1 (i32.mul)
        // 1010 = 10 (local.get) + 1000 (call)
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
                  i32.const 11
                  call 0
                  local.get 0
                  call 1))
            "#,
        )
        .unwrap();

        assert_eq!(transformed, expected);
    }
}
