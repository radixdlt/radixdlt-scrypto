use parity_wasm::elements::{
    External, FunctionType,
    Instruction::{self, *},
    Internal, Module, Type, ValueType,
};
use sbor::rust::format;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use scrypto::abi::BlueprintAbi;
use scrypto::prelude::HashMap;
use wasm_instrument::{gas_metering, inject_stack_limiter};
use wasmi_validation::{validate_module, PlainValidator};

use crate::wasm::{constants::*, errors::*, PrepareError};

use super::WasmiEnvModule;

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

        // Function input and output types
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
                if entry.module() == MODULE_ENV_NAME && entry.field() == RADIX_ENGINE_FUNCTION_NAME
                {
                    if let External::Function(type_index) = entry.external() {
                        if Self::function_type_matches(
                            &self.module,
                            *type_index as usize,
                            vec![ValueType::I32],
                            vec![ValueType::I32],
                        ) {
                            continue;
                        }
                    }
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
        // Must have exactly 1 internal memory definition
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

        // TODO: do we need to enforce limit on the number of locals and parameter?

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

    pub fn enforce_export_constraints(
        self,
        blueprints: &HashMap<String, BlueprintAbi>,
    ) -> Result<Self, PrepareError> {
        let exports = self
            .module
            .export_section()
            .ok_or(PrepareError::NoExportSection)?;
        for (_, blueprint_abi) in blueprints {
            for func in &blueprint_abi.fns {
                let func_name = &func.export_name;
                if !exports.entries().iter().any(|x| {
                    x.field().eq(func_name) && {
                        if let Internal::Function(func_index) = x.internal() {
                            Self::function_matches(
                                &self.module,
                                *func_index as usize,
                                vec![ValueType::I32],
                                vec![ValueType::I32],
                            )
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

    pub fn inject_instruction_metering(
        mut self,
        instruction_cost: u32,
        grow_memory_cost: u32,
    ) -> Result<Self, PrepareError> {
        self.module = gas_metering::inject(
            self.module,
            &gas_metering::ConstantCostRules::new(instruction_cost, grow_memory_cost),
            MODULE_ENV_NAME,
        )
        .map_err(|_| PrepareError::RejectedByInstructionMetering)?;

        Ok(self)
    }

    pub fn inject_stack_metering(mut self, wasm_max_stack_size: u32) -> Result<Self, PrepareError> {
        self.module = inject_stack_limiter(self.module, wasm_max_stack_size)
            .map_err(|_| PrepareError::RejectedByStackMetering)?;
        Ok(self)
    }

    pub fn ensure_instantiatable(self) -> Result<Self, PrepareError> {
        // During instantiation time, the following steps are applied

        // 1. Resolve imports with external values
        // This should always succeed as we only allow `env::radix_engine` function import

        // 2. Allocate externals, functions, tables, memory and globals
        // This should always succeed as we enforce an upper bound for each type

        // 3. Update table with elements
        // It may fail if the offset is out of bound

        // 4. Update memory with data segments
        // It may fail if the offset is out of bound

        // Because the offset can be an `InitExpr` that requires evaluation against an WASM instance,
        // we're using the `wasmi` logic as a short cut.

        wasmi::ModuleInstance::new(
            &wasmi::Module::from_parity_wasm_module(self.module.clone())
                .expect("Due to the `init` step module should be valid"),
            &wasmi::ImportsBuilder::new().with_resolver(MODULE_ENV_NAME, &WasmiEnvModule {}),
        )
        .map_err(|e| PrepareError::NotInstantiatable(format!("{:?}", e)))?;

        Ok(self)
    }

    pub fn ensure_compilable(self) -> Result<Self, PrepareError> {
        // TODO: Understand WASM JIT compilability
        //
        // Can we make the assumption that all "prepared" modules are compilable, if machine resource is "sufficient"?
        //
        // Another option is to attempt to compile, although it would make RE protocol coupled with the specific implementation

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
        let func = module
            .function_section()
            .map(|s| s.entries())
            .unwrap_or(&[])
            .get(func_index)
            .expect("Due to validation function should exist");
        Self::function_type_matches(module, func.type_ref() as usize, params, results)
    }

    fn function_type_matches(
        module: &Module,
        type_index: usize,
        params: Vec<ValueType>,
        results: Vec<ValueType>,
    ) -> bool {
        let ty = module
            .type_section()
            .map(|s| s.types())
            .unwrap_or(&[])
            .get(type_index)
            .expect("Due to validation type should exist");

        ty == &Type::Function(FunctionType::new(params, results))
    }
}
