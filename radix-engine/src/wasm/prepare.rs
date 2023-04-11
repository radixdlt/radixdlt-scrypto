use parity_wasm::elements::{
    External, FunctionType,
    Instruction::{self, *},
    Internal, Module, Type, ValueType,
};
use radix_engine_interface::schema::PackageSchema;
use wasm_instrument::{
    gas_metering::{self, Rules},
    inject_stack_limiter,
};
use wasmi_validation::{validate_module, PlainValidator};

use crate::types::*;
use crate::wasm::{constants::*, errors::*, PrepareError};

use super::WasmiModule;
#[derive(Debug, PartialEq)]
pub struct WasmModule {
    module: Module,
}

impl WasmModule {
    pub fn init(code: &[u8]) -> Result<Self, PrepareError> {
        // deserialize
        let module = parity_wasm::deserialize_buffer(code)
            .map_err(|_| PrepareError::DeserializationError)?;

        // validate
        validate_module::<PlainValidator>(&module).map_err(|_| PrepareError::ValidationError)?;

        Ok(Self { module })
    }

    pub fn enforce_no_floating_point(self) -> Result<Self, PrepareError> {
        // Global value types
        if let Some(globals) = self.module.global_section() {
            for global in globals.entries() {
                match global.global_type().content_type() {
                    ValueType::F32 | ValueType::F64 => {
                        return Err(PrepareError::FloatingPointNotAllowed)
                    }
                    _ => {}
                }
            }
        }

        // Function local value types and floating-point related instructions
        if let Some(code) = self.module.code_section() {
            for func_body in code.bodies() {
                for local in func_body.locals() {
                    match local.value_type() {
                        ValueType::F32 | ValueType::F64 => {
                            return Err(PrepareError::FloatingPointNotAllowed)
                        }
                        _ => {}
                    }
                }

                for op in func_body.code().elements() {
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
                            return Err(PrepareError::FloatingPointNotAllowed);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Function argument and result types
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
                                return Err(PrepareError::FloatingPointNotAllowed);
                            }
                        }
                    }
                }
            }
        }

        Ok(self)
    }

    pub fn enforce_no_start_function(self) -> Result<Self, PrepareError> {
        if self.module.start_section().is_some() {
            Err(PrepareError::StartFunctionNotAllowed)
        } else {
            Ok(self)
        }
    }

    pub fn enforce_import_limit(self) -> Result<Self, PrepareError> {
        // Only allow `env::radix_engine` import
        if let Some(sec) = self.module.import_section() {
            for entry in sec.entries() {
                if entry.module() == MODULE_ENV_NAME {
                    match entry.field() {
                        CONSUME_BUFFER_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![ValueType::I32, ValueType::I32],
                                    vec![],
                                ) {
                                    continue;
                                }

                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        CONSUME_BUFFER_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        CALL_METHOD_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                    ],
                                    vec![ValueType::I64],
                                ) {
                                    continue;
                                }

                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        CALL_METHOD_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        CALL_FUNCTION_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                    ],
                                    vec![ValueType::I64],
                                ) {
                                    continue;
                                }

                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        CALL_METHOD_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        DROP_OBJECT_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![ValueType::I32, ValueType::I32],
                                    vec![],
                                ) {
                                    continue;
                                }

                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        DROP_OBJECT_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        LOCK_SUBSTATE_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                    ],
                                    vec![ValueType::I32],
                                ) {
                                    continue;
                                }

                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        LOCK_SUBSTATE_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        READ_SUBSTATE_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![ValueType::I32],
                                    vec![ValueType::I64],
                                ) {
                                    continue;
                                }

                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        READ_SUBSTATE_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        WRITE_SUBSTATE_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![ValueType::I32, ValueType::I32, ValueType::I32],
                                    vec![],
                                ) {
                                    continue;
                                }

                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        WRITE_SUBSTATE_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        DROP_LOCK_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![ValueType::I32],
                                    vec![],
                                ) {
                                    continue;
                                }

                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        DROP_LOCK_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        GET_GLOBAL_ADDRESS_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![],
                                    vec![ValueType::I64],
                                ) {
                                    continue;
                                }

                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        GET_GLOBAL_ADDRESS_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        GET_BLUEPRINT_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![],
                                    vec![ValueType::I64],
                                ) {
                                    continue;
                                }

                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        GET_BLUEPRINT_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        GET_AUTH_ZONE_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![],
                                    vec![ValueType::I64],
                                ) {
                                    continue;
                                }

                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        GET_AUTH_ZONE_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        ASSERT_ACCESS_RULE_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![ValueType::I32, ValueType::I32],
                                    vec![],
                                ) {
                                    continue;
                                }

                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        ASSERT_ACCESS_RULE_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        NEW_OBJECT_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                    ],
                                    vec![ValueType::I64],
                                ) {
                                    continue;
                                }

                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        NEW_OBJECT_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        GLOBALIZE_OBJECT_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                    ],
                                    vec![ValueType::I64],
                                ) {
                                    continue;
                                }
                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        GLOBALIZE_OBJECT_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        NEW_KEY_VALUE_STORE_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![ValueType::I32, ValueType::I32],
                                    vec![ValueType::I64],
                                ) {
                                    continue;
                                }
                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        NEW_KEY_VALUE_STORE_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        GET_OBJECT_INFO_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![ValueType::I32, ValueType::I32],
                                    vec![ValueType::I64],
                                ) {
                                    continue;
                                }
                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        GET_OBJECT_INFO_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        GET_KEY_VALUE_STORE_INFO_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![ValueType::I32, ValueType::I32],
                                    vec![ValueType::I64],
                                ) {
                                    continue;
                                }
                                return Err(PrepareError::InvalidImport(
                                    InvalidImport::InvalidFunctionType(
                                        GET_KEY_VALUE_STORE_INFO_FUNCTION_NAME.to_string(),
                                    ),
                                ));
                            }
                        }
                        EMIT_EVENT_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                    ],
                                    vec![],
                                ) {
                                    continue;
                                }
                            }
                        }
                        LOG_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                        ValueType::I32,
                                    ],
                                    vec![],
                                ) {
                                    continue;
                                }
                            }
                        }
                        GET_TRANSACTION_HASH_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![],
                                    vec![ValueType::I64],
                                ) {
                                    continue;
                                }
                            }
                        }
                        GENERATE_UUID_FUNCTION_NAME => {
                            if let External::Function(type_index) = entry.external() {
                                if Self::function_type_matches(
                                    &self.module,
                                    *type_index as usize,
                                    vec![],
                                    vec![ValueType::I64],
                                ) {
                                    continue;
                                }
                            }
                        }
                        _ => {}
                    };
                }

                return Err(PrepareError::InvalidImport(InvalidImport::ImportNotAllowed));
            }
        }

        Ok(self)
    }

    pub fn enforce_memory_limit(
        self,
        max_initial_memory_size_pages: u32,
    ) -> Result<Self, PrepareError> {
        // Must have exactly 1 internal, exported memory definition
        // TODO: consider if we can benefit from shared external memory.
        let memory_section = self
            .module
            .memory_section()
            .ok_or(PrepareError::InvalidMemory(InvalidMemory::NoMemorySection))?;

        let memory = match memory_section.entries().len() {
            0 => Err(PrepareError::InvalidMemory(
                InvalidMemory::EmptyMemorySection,
            )),
            1 => Ok(memory_section.entries()[0]),
            _ => Err(PrepareError::InvalidMemory(InvalidMemory::TooManyMemories)),
        }?;
        if memory.limits().initial() > max_initial_memory_size_pages {
            return Err(PrepareError::InvalidMemory(
                InvalidMemory::InitialMemorySizeLimitExceeded,
            ));
        }

        self.module
            .export_section()
            .and_then(|section| {
                section
                    .entries()
                    .iter()
                    .filter(|e| e.field() == EXPORT_MEMORY && e.internal() == &Internal::Memory(0))
                    .next()
            })
            .ok_or(PrepareError::InvalidMemory(
                InvalidMemory::MemoryNotExported,
            ))?;

        Ok(self)
    }

    pub fn enforce_table_limit(self, max_initial_table_size: u32) -> Result<Self, PrepareError> {
        if let Some(section) = self.module.table_section() {
            if section.entries().len() > 1 {
                // Sanity check MVP rule
                return Err(PrepareError::InvalidTable(InvalidTable::MoreThanOneTable));
            }

            if let Some(table) = section.entries().get(0) {
                if table.limits().initial() > max_initial_table_size {
                    return Err(PrepareError::InvalidTable(
                        InvalidTable::InitialTableSizeLimitExceeded,
                    ));
                }
            }
        }

        Ok(self)
    }

    pub fn enforce_br_table_limit(
        self,
        max_number_of_br_table_targets: u32,
    ) -> Result<Self, PrepareError> {
        if let Some(section) = self.module.code_section() {
            for inst in section
                .bodies()
                .iter()
                .flat_map(|body| body.code().elements())
            {
                if let Instruction::BrTable(table_data) = inst {
                    if table_data.table.len() > max_number_of_br_table_targets as usize {
                        return Err(PrepareError::TooManyTargetsInBrTable);
                    }
                }
            }
        }
        Ok(self)
    }

    pub fn enforce_function_limit(
        self,
        max_number_of_functions: u32,
    ) -> Result<Self, PrepareError> {
        if let Some(section) = self.module.function_section() {
            if section.entries().len() > max_number_of_functions as usize {
                return Err(PrepareError::TooManyGlobals);
            }
        }

        // TODO: do we need to enforce limit on the number of locals and parameters?

        Ok(self)
    }

    pub fn enforce_global_limit(self, max_number_of_globals: u32) -> Result<Self, PrepareError> {
        if let Some(section) = self.module.global_section() {
            if section.entries().len() > max_number_of_globals as usize {
                return Err(PrepareError::TooManyGlobals);
            }
        }
        Ok(self)
    }

    pub fn enforce_export_constraints(self, schema: &PackageSchema) -> Result<Self, PrepareError> {
        let exports = self
            .module
            .export_section()
            .ok_or(PrepareError::NoExportSection)?;
        for (_, blueprint_schema) in &schema.blueprints {
            for func in blueprint_schema.functions.values() {
                let func_name = &func.export_name;
                if !exports.entries().iter().any(|x| {
                    x.field().eq(func_name) && {
                        if let Internal::Function(func_index) = x.internal() {
                            if func.receiver.is_some() {
                                Self::function_matches(
                                    &self.module,
                                    *func_index as usize,
                                    vec![ValueType::I64, ValueType::I64],
                                    vec![ValueType::I64],
                                )
                            } else {
                                Self::function_matches(
                                    &self.module,
                                    *func_index as usize,
                                    vec![ValueType::I64],
                                    vec![ValueType::I64],
                                )
                            }
                        } else {
                            false
                        }
                    }
                }) {
                    return Err(PrepareError::MissingExport {
                        export_name: func_name.to_string(),
                    });
                }
            }
        }

        Ok(self)
    }

    pub fn inject_instruction_metering<R: Rules>(
        mut self,
        rules: &R,
    ) -> Result<Self, PrepareError> {
        self.module = gas_metering::inject(self.module, rules, MODULE_ENV_NAME)
            .map_err(|_| PrepareError::RejectedByInstructionMetering)?;

        Ok(self)
    }

    pub fn inject_stack_metering(mut self, wasm_max_stack_size: u32) -> Result<Self, PrepareError> {
        self.module = inject_stack_limiter(self.module, wasm_max_stack_size)
            .map_err(|_| PrepareError::RejectedByStackMetering)?;
        Ok(self)
    }

    pub fn ensure_instantiatable(self) -> Result<Self, PrepareError> {
        // During instantiation time, the following procedures are applied:

        // 1. Resolve imports with external values
        // This should always succeed as we only allow `env::radix_engine` function import

        // 2. Allocate externals, functions, tables, memory and globals
        // This should always succeed as we enforce an upper bound for each type

        // 3. Update table with elements
        // It may fail if the offset is out of bound

        // 4. Update memory with data segments
        // It may fail if the offset is out of bound

        // Because the offset can be an `InitExpr` that requires evaluation against an WASM instance,
        // we're using the `wasmi` logic as a shortcut.
        let code = parity_wasm::serialize(self.module.clone())
            .map_err(|_| PrepareError::SerializationError)?;

        WasmiModule::new(&code[..]).map(|_| self)
    }

    pub fn ensure_compilable(self) -> Result<Self, PrepareError> {
        // TODO: Understand WASM JIT compilability
        //
        // Can we make the assumption that all "prepared" modules are compilable,
        // if machine resource is "sufficient"?
        //
        // Another option is to attempt to compile, although it may make RE protocol
        // coupled with a specific implementation.

        Ok(self)
    }

    pub fn to_bytes(self) -> Result<(Vec<u8>, Vec<String>), PrepareError> {
        let function_exports = self
            .module
            .export_section()
            .map(|sec| {
                sec.entries()
                    .iter()
                    .filter(|e| matches!(e.internal(), Internal::Function(_)))
                    .map(|e| e.field().to_string())
                    .collect()
            })
            .unwrap_or(Vec::new());

        let code =
            parity_wasm::serialize(self.module).map_err(|_| PrepareError::SerializationError)?;

        Ok((code, function_exports))
    }

    fn function_matches(
        module: &Module,
        func_index: usize,
        params: Vec<ValueType>,
        results: Vec<ValueType>,
    ) -> bool {
        let func_import_count = module
            .import_section()
            .map(|s| s.entries())
            .unwrap_or(&[])
            .iter()
            .filter(|e| matches!(e.external(), External::Function(_)))
            .count();

        module
            .function_section()
            .map(|s| s.entries())
            .unwrap_or(&[])
            .get(func_index - func_import_count)
            .map(|func| {
                Self::function_type_matches(module, func.type_ref() as usize, params, results)
            })
            .unwrap_or(false)
    }

    fn function_type_matches(
        module: &Module,
        type_index: usize,
        params: Vec<ValueType>,
        results: Vec<ValueType>,
    ) -> bool {
        module
            .type_section()
            .map(|s| s.types())
            .unwrap_or(&[])
            .get(type_index)
            .map(|ty| ty == &Type::Function(FunctionType::new(params, results)))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use radix_engine_interface::schema::{BlueprintSchema, FunctionSchema};
    use sbor::basic_well_known_types::{ANY_ID, UNIT_ID};
    use wabt::wat2wasm;

    macro_rules! assert_invalid_wasm {
        ($wat: expr, $err: expr, $func: expr) => {
            let code = wat2wasm($wat).unwrap();
            assert_eq!(Err($err), WasmModule::init(&code).map($func).unwrap());
        };
    }

    #[test]
    fn test_floating_point() {
        // return
        assert_invalid_wasm!(
            r#"
            (module
                (func (result f64)
                    f64.const 123
                )
            )
            "#,
            PrepareError::FloatingPointNotAllowed,
            WasmModule::enforce_no_floating_point
        );
        // input
        assert_invalid_wasm!(
            r#"
            (module
                (func (param f64)
                )
            )
            "#,
            PrepareError::FloatingPointNotAllowed,
            WasmModule::enforce_no_floating_point
        );
        // instruction
        assert_invalid_wasm!(
            r#"
            (module
                (func
                    f64.const 1
                    f64.const 2
                    f64.add
                    drop
                )
            )
            "#,
            PrepareError::FloatingPointNotAllowed,
            WasmModule::enforce_no_floating_point
        );
        // global
        assert_invalid_wasm!(
            r#"
            (module
                (global $fp f32 (f32.const 10))
            )
            "#,
            PrepareError::FloatingPointNotAllowed,
            WasmModule::enforce_no_floating_point
        );
    }

    #[test]
    fn test_start_function() {
        assert_invalid_wasm!(
            r#"
            (module
                (func $main)
                (start $main)
            )
            "#,
            PrepareError::StartFunctionNotAllowed,
            WasmModule::enforce_no_start_function
        );
    }

    #[test]
    fn test_memory() {
        assert_invalid_wasm!(
            r#"
            (module
            )
            "#,
            PrepareError::InvalidMemory(InvalidMemory::NoMemorySection),
            |x| WasmModule::enforce_memory_limit(x, 5)
        );
        // NOTE: Disabled as MVP only allow 1 memory definition
        // assert_invalid_wasm!(
        //     r#"
        //     (module
        //         (memory 2)
        //         (memory 2)
        //     )
        //     "#,
        //     PrepareError::InvalidMemory(InvalidMemory::TooManyMemories),
        //     |x| WasmModule::enforce_memory_limit(x, 5)
        // );
        assert_invalid_wasm!(
            r#"
            (module
                (memory 6)
            )
            "#,
            PrepareError::InvalidMemory(InvalidMemory::InitialMemorySizeLimitExceeded),
            |x| WasmModule::enforce_memory_limit(x, 5)
        );
        assert_invalid_wasm!(
            r#"
            (module
                (memory 2)
            )
            "#,
            PrepareError::InvalidMemory(InvalidMemory::MemoryNotExported),
            |x| WasmModule::enforce_memory_limit(x, 5)
        );
    }

    #[test]
    fn test_table() {
        assert_invalid_wasm!(
            r#"
            (module
                (table 6 funcref)
            )
            "#,
            PrepareError::InvalidTable(InvalidTable::InitialTableSizeLimitExceeded),
            |x| WasmModule::enforce_table_limit(x, 5)
        );
    }

    #[test]
    fn test_br_table() {
        assert_invalid_wasm!(
            r#"
            (module
                (func (param i32) (result i32)
                    (block
                        (block
                            (br_table 1 0 1 0 1 0 1 (local.get 0))
                            (return (i32.const 21))
                        )
                        (return (i32.const 20))
                    )
                    (i32.const 22)
                )
            )
            "#,
            PrepareError::TooManyTargetsInBrTable,
            |x| WasmModule::enforce_br_table_limit(x, 5)
        );
    }

    #[test]
    fn test_blueprint_constraints() {
        let mut package_schema = PackageSchema::default();
        package_schema.blueprints.insert(
            "Test".to_string(),
            BlueprintSchema {
                parent: None,
                schema: ScryptoSchema {
                    type_kinds: vec![],
                    type_metadata: vec![],
                    type_validations: vec![],
                },
                substates: vec![LocalTypeIndex::WellKnown(UNIT_ID)],
                functions: btreemap!(
                    "f".to_string() => FunctionSchema {
                        receiver: Option::None,
                        input: LocalTypeIndex::WellKnown(ANY_ID),
                        output: LocalTypeIndex::WellKnown(UNIT_ID),
                        export_name: "Test_f".to_string(),
                    }
                ),
                virtual_lazy_load_functions: btreemap!(),
                event_schema: [].into(),
            },
        );

        assert_invalid_wasm!(
            r#"
            (module
            )
            "#,
            PrepareError::NoExportSection,
            |x| WasmModule::enforce_export_constraints(x, &package_schema)
        );
        // symbol not found
        assert_invalid_wasm!(
            r#"
            (module
                (func (export "foo") (result i32)
                    (i32.const 0)
                )
            )
            "#,
            PrepareError::MissingExport {
                export_name: "Test_f".to_string()
            },
            |x| WasmModule::enforce_export_constraints(x, &package_schema)
        );
        // signature does not match
        assert_invalid_wasm!(
            r#"
            (module
                (func (export "Test_f") (result i32)
                    (i32.const 0)
                )
            )
            "#,
            PrepareError::MissingExport {
                export_name: "Test_f".to_string()
            },
            |x| WasmModule::enforce_export_constraints(x, &package_schema)
        );
    }
}
