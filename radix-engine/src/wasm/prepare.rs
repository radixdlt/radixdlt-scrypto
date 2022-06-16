use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use wasm_instrument::parity_wasm::{
    self,
    elements::{External, Instruction::*, Internal, Module, Type, ValueType},
};
use wasm_instrument::{gas_metering, inject_stack_limiter};
use wasmi_validation::{validate_module, PlainValidator};

use crate::wasm::constants::*;
use crate::wasm::errors::*;
use crate::wasm::WasmiEnvModule;

pub struct WasmModule {
    module: Module,
}

impl WasmModule {
    /// The maximum initial memory size: `64 Pages * 64 KiB per Page = 4 MiB`
    pub const MAX_INITIAL_MEMORY_SIZE_PAGES: u32 = 64;

    pub fn init(code: &[u8]) -> Result<Self, PrepareError> {
        // deserialize
        let module = parity_wasm::deserialize_buffer(code)
            .map_err(|_| PrepareError::DeserializationError)?;

        // validate
        validate_module::<PlainValidator>(&module).map_err(|_| PrepareError::ValidationError)?;

        Ok(Self { module })
    }

    pub fn reject_floating_point(self) -> Result<Self, PrepareError> {
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
                        return Err(PrepareError::FloatingPointNotAllowed);
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
                                return Err(PrepareError::FloatingPointNotAllowed);
                            }
                        }
                    }
                }
            }
        }

        Ok(self)
    }

    pub fn reject_start_function(self) -> Result<Self, PrepareError> {
        if self.module.start_section().is_some() {
            Err(PrepareError::StartFunctionNotAllowed)
        } else {
            Ok(self)
        }
    }

    pub fn check_imports(self) -> Result<Self, PrepareError> {
        // only allow `env::radix_engine` import

        if let Some(sec) = self.module.import_section() {
            if sec.entries().len() > 1 {
                return Err(PrepareError::InvalidImports);
            }

            if let Some(entry) = sec.entries().get(0) {
                if entry.module() != MODULE_ENV_NAME
                    || entry.field() != RADIX_ENGINE_FUNCTION_NAME
                    || !matches!(entry.external(), External::Function(_))
                {
                    return Err(PrepareError::InvalidImports);
                }
            }
        }

        Ok(self)
    }

    pub fn check_memory(self) -> Result<Self, PrepareError> {
        // Must have exactly 1 memory definition
        // TODO: consider if we can benefit from shared external memory instead of internal ones.
        let memory_section = self.module.memory_section().ok_or(PrepareError::NoMemory)?;

        let memory = match memory_section.entries().len() {
            0 => Err(PrepareError::NoMemory),
            1 => Ok(memory_section.entries()[0]),
            _ => Err(PrepareError::TooManyMemories),
        }?;
        if memory.limits().initial() > Self::MAX_INITIAL_MEMORY_SIZE_PAGES {
            return Err(PrepareError::InitialMemorySizeLimitExceeded);
        }

        if !self
            .module
            .export_section()
            .ok_or(PrepareError::NoMemoryExport)?
            .entries()
            .iter()
            .any(|e| e.field() == EXPORT_MEMORY && e.internal() == &Internal::Memory(0))
        {
            return Err(PrepareError::NoMemoryExport);
        }

        Ok(self)
    }

    pub fn enforce_initial_memory_limit(self) -> Result<Self, PrepareError> {
        // TODO
        Ok(self)
    }

    pub fn enforce_functions_limit(self) -> Result<Self, PrepareError> {
        // TODO
        Ok(self)
    }

    pub fn enforce_locals_limit(self) -> Result<Self, PrepareError> {
        // TODO
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

    fn ensure_instantiatable(code: &[u8]) -> Result<(), PrepareError> {
        // There are a few things that are done during wasm instantiation time
        //
        // 1. Resolve all the external functions, tables, memories and globals (which should always succeed if `check_imports` step is applied)
        // 2. Allocate globals (TODO: study the behavior and design costing strategy)
        // 3. Allocate tables (TODO: study the behavior and design costing strategy)
        // 4. Allocate memories (which should always succeed if `check_memory` step is applied)
        // 5. Update table with elements (TODO: study the behavior and design costing strategy)
        // 6. Update memory with data sections (which may fail if the initial memory is less than required)
        //
        // Before we fully understand the specs and reference implementation, we attempt to instantiate the module and reject it if failure.

        wasmi::ModuleInstance::new(
            &wasmi::Module::from_buffer(&code).expect("The input code should be a valid module"),
            &wasmi::ImportsBuilder::new().with_resolver(MODULE_ENV_NAME, &WasmiEnvModule {}),
        )
        .map_err(|_| PrepareError::NotInstantiatable)?;

        Ok(())
    }

    fn ensure_compilable(_code: &[u8]) -> Result<(), PrepareError> {
        // TODO: can we make the assumption that all "validated" modules are compilable, if the machine resource is "sufficient"?

        // Another option is to attempt to compile, although it would make RE protocol coupled with the specific implementation

        Ok(())
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

        // serialize
        let code =
            parity_wasm::serialize(self.module).map_err(|_| PrepareError::SerializationError)?;

        // make sure the module is instantiable and compilable
        Self::ensure_instantiatable(&code)?;
        Self::ensure_compilable(&code)?;

        Ok((code, function_exports))
    }
}
