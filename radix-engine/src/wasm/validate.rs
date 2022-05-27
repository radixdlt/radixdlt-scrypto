use sbor::rust::vec::Vec;
use wasm_instrument::parity_wasm::{
    self,
    elements::{Instruction::*, Module, Type, ValueType},
};
use wasm_instrument::{gas_metering, inject_stack_limiter};

use crate::wasm::constants::*;
use crate::wasm::errors::*;

pub struct ScryptoValidator {
    module: Module,
}

impl ScryptoValidator {
    pub fn init(code: &[u8]) -> Result<Self, ValidateError> {
        let module = parity_wasm::deserialize_buffer(code)
            .map_err(|_| ValidateError::DeserializationError)?;
        Ok(Self { module })
    }

    pub fn reject_floating_point(self) -> Result<Self, ValidateError> {
        if let Some(code) = self.module.code_section() {
            for op in code.bodies().iter().flat_map(|body| body.code().elements()) {
                match op {
                    F32Load(_, _)
                    | F64Load(_, _)
                    | F32Store(_, _)
                    | F64Store(_, _)
                    | F32Const(_)
                    | F64Const(_)
                    | F32Eq
                    | F32Ne
                    | F32Lt
                    | F32Gt
                    | F32Le
                    | F32Ge
                    | F64Eq
                    | F64Ne
                    | F64Lt
                    | F64Gt
                    | F64Le
                    | F64Ge
                    | F32Abs
                    | F32Neg
                    | F32Ceil
                    | F32Floor
                    | F32Trunc
                    | F32Nearest
                    | F32Sqrt
                    | F32Add
                    | F32Sub
                    | F32Mul
                    | F32Div
                    | F32Min
                    | F32Max
                    | F32Copysign
                    | F64Abs
                    | F64Neg
                    | F64Ceil
                    | F64Floor
                    | F64Trunc
                    | F64Nearest
                    | F64Sqrt
                    | F64Add
                    | F64Sub
                    | F64Mul
                    | F64Div
                    | F64Min
                    | F64Max
                    | F64Copysign
                    | F32ConvertSI32
                    | F32ConvertUI32
                    | F32ConvertSI64
                    | F32ConvertUI64
                    | F32DemoteF64
                    | F64ConvertSI32
                    | F64ConvertUI32
                    | F64ConvertSI64
                    | F64ConvertUI64
                    | F64PromoteF32
                    | F32ReinterpretI32
                    | F64ReinterpretI64
                    | I32TruncSF32
                    | I32TruncUF32
                    | I32TruncSF64
                    | I32TruncUF64
                    | I64TruncSF32
                    | I64TruncUF32
                    | I64TruncSF64
                    | I64TruncUF64
                    | I32ReinterpretF32
                    | I64ReinterpretF64 => {
                        return Err(ValidateError::FloatingPointNotAllowed);
                    }
                    _ => {}
                }
            }
        }

        if let (Some(functions), Some(types)) =
            (self.module.function_section(), self.module.type_section())
        {
            let types = types.types();

            for sig in functions.entries() {
                if let Some(typ) = types.get(sig.type_ref() as usize) {
                    match *typ {
                        Type::Function(ref func) => {
                            if func
                                .params()
                                .iter()
                                .chain(func.results())
                                .any(|&typ| typ == ValueType::F32 || typ == ValueType::F64)
                            {
                                return Err(ValidateError::FloatingPointNotAllowed);
                            }
                        }
                    }
                }
            }
        }

        Ok(self)
    }

    pub fn reject_start_function(self) -> Result<Self, ValidateError> {
        if self.module.start_section().is_some() {
            Err(ValidateError::StartFunctionNotAllowed)
        } else {
            Ok(self)
        }
    }

    pub fn check_imports(self) -> Result<Self, ValidateError> {
        // TODO
        Ok(self)
    }

    pub fn check_exports(self) -> Result<Self, ValidateError> {
        // TODO
        Ok(self)
    }

    pub fn check_memory(self) -> Result<Self, ValidateError> {
        // TODO
        Ok(self)
    }

    pub fn enforce_initial_memory_limit(self) -> Result<Self, ValidateError> {
        // TODO
        Ok(self)
    }

    pub fn enforce_functions_limit(self) -> Result<Self, ValidateError> {
        // TODO
        Ok(self)
    }

    pub fn enforce_locals_limit(self) -> Result<Self, ValidateError> {
        // TODO
        Ok(self)
    }

    pub fn inject_instruction_metering(mut self) -> Result<Self, ValidateError> {
        self.module = gas_metering::inject(
            self.module,
            &gas_metering::ConstantCostRules::new(INSTRUCTION_COST, MEMORY_GROW_COST),
            MODULE_ENV_NAME,
        )
        .map_err(|_| ValidateError::FailedToInjectInstructionMetering)?;

        Ok(self)
    }

    pub fn inject_stack_metering(mut self) -> Result<Self, ValidateError> {
        self.module = inject_stack_limiter(self.module, MAX_STACK_DEPTH)
            .map_err(|_| ValidateError::FailedToInjectStackMetering)?;
        Ok(self)
    }

    pub fn to_bytes(self) -> Result<Vec<u8>, ValidateError> {
        parity_wasm::serialize(self.module).map_err(|_| ValidateError::SerializationError)
    }
}
