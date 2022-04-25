use sbor::*;
use scrypto::abi::{Function, Method};
use scrypto::buffer::scrypto_decode;
use scrypto::prelude::PackageFunction;
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::rust::format;
use scrypto::values::ScryptoValue;
use wasmi::{Externals, ExternVal, ImportsBuilder, MemoryRef, Module, ModuleInstance, ModuleRef, NopExternals, RuntimeValue};

use crate::engine::{EnvModuleResolver, SystemApi};
use crate::errors::{RuntimeError, WasmValidationError};

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Package {
    code: Vec<u8>,
    blueprints: HashMap<String, Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageError {
    InvalidRequestData(DecodeError),
    BlueprintNotFound,
    WasmValidationError(WasmValidationError),
    MethodNotFound(String),
}

impl Package {
    /// Validates and creates a package
    pub fn new(code: Vec<u8>) -> Result<Self, WasmValidationError> {
        // Parse
        let parsed = Self::parse_module(&code)?;

        // check floating point
        parsed
            .deny_floating_point()
            .map_err(|_| WasmValidationError::FloatingPointNotAllowed)?;

        // Instantiate
        let instance = ModuleInstance::new(
            &parsed,
            &ImportsBuilder::new().with_resolver("env", &EnvModuleResolver),
        )
        .map_err(|_| WasmValidationError::InvalidModule)?;

        // Check start function
        if instance.has_start() {
            return Err(WasmValidationError::StartFunctionNotAllowed);
        }
        let module = instance.assert_no_start();

        // Check memory export
        let memory = match module.export_by_name("memory") {
            Some(ExternVal::Memory(mem)) => mem,
            _ => return Err(WasmValidationError::NoValidMemoryExport),
        };

        // TODO: Currently a hack so that we don't require a package_init function.
        // TODO: Fix this by implement package metadata along with the code during compilation.
        let exports = module.exports();
        let blueprint_abi_methods: Vec<String> = exports
            .iter()
            .filter(|(name, val)| {
                name.ends_with("_abi") && name.len() > 4 && matches!(val, ExternVal::Func(_))
            })
            .map(|(name, _)| name.to_string())
            .collect();

        let mut blueprints = HashMap::new();

        for method_name in blueprint_abi_methods {
            let rtn = module
                .invoke_export(&method_name, &[], &mut NopExternals)
                .map_err(|e| WasmValidationError::NoPackageInitExport(e.into()))?
                .ok_or(WasmValidationError::InvalidPackageInit)?;

            let blueprint_type: Type = match rtn {
                RuntimeValue::I32(ptr) => {
                    let len: u32 = memory
                        .get_value(ptr as u32)
                        .map_err(|_| WasmValidationError::InvalidPackageInit)?;

                    // SECURITY: meter before allocating memory
                    let mut data = vec![0u8; len as usize];
                    memory
                        .get_into((ptr + 4) as u32, &mut data)
                        .map_err(|_| WasmValidationError::InvalidPackageInit)?;

                    let result: (Type, Vec<Function>, Vec<Method>) = scrypto_decode(&data)
                        .map_err(|_| WasmValidationError::InvalidPackageInit)?;
                    Ok(result.0)
                }
                _ => Err(WasmValidationError::InvalidPackageInit),
            }?;

            if let Type::Struct { name, fields: _ } = &blueprint_type {
                blueprints.insert(name.clone(), blueprint_type);
            } else {
                return Err(WasmValidationError::InvalidPackageInit);
            }
        }

        Ok(Self { blueprints, code })
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn contains_blueprint(&self, blueprint_name: &str) -> bool {
        self.blueprints.contains_key(blueprint_name)
    }

    pub fn load_blueprint_schema(&self, blueprint_name: &str) -> Result<&Type, PackageError> {
        self.blueprints
            .get(blueprint_name)
            .ok_or(PackageError::BlueprintNotFound)
    }

    pub fn load_module(&self) -> Result<(ModuleRef, MemoryRef), PackageError> {
        let module = Self::parse_module(&self.code).unwrap();
        let inst = Self::instantiate_module(&module).unwrap();
        Ok(inst)
    }

    fn parse_module(code: &[u8]) -> Result<Module, WasmValidationError> {
        Module::from_buffer(code).map_err(|_| WasmValidationError::InvalidModule)
    }

    fn instantiate_module(module: &Module) -> Result<(ModuleRef, MemoryRef), WasmValidationError> {
        // Instantiate
        let instance = ModuleInstance::new(
            module,
            &ImportsBuilder::new().with_resolver("env", &EnvModuleResolver),
        )
        .map_err(|_| WasmValidationError::InvalidModule)?
        .assert_no_start();

        // Find memory export
        if let Some(ExternVal::Memory(memory)) = instance.export_by_name("memory") {
            Ok((instance, memory))
        } else {
            Err(WasmValidationError::NoValidMemoryExport)
        }
    }

    pub fn static_main<S: SystemApi>(
        arg: ScryptoValue,
        system_api: &mut S,
    ) -> Result<ScryptoValue, PackageError> {
        let function: PackageFunction = scrypto_decode(&arg.raw).map_err(|e| PackageError::InvalidRequestData(e))?;
        match function {
            PackageFunction::Publish(bytes) => {
                let package = Package::new(bytes).map_err(PackageError::WasmValidationError)?;
                let package_address = system_api.create_package(package);
                Ok(ScryptoValue::from_value(&package_address))
            }
        }
    }

    /// Calls the ABI generator of a blueprint.
    // TODO: Remove
    pub fn call_abi(
        &self,
        blueprint_name: &str,
    ) -> Result<ScryptoValue, RuntimeError> {
        let (module, memory) = self.load_module().unwrap();
        let export_name = format!("{}_abi", blueprint_name);
        let result = module.invoke_export(&export_name, &[], &mut NopExternals);
        let rtn = result
            .map_err(|e| {
                match e.into_host_error() {
                    // Pass-through runtime errors
                    Some(host_error) => *host_error.downcast::<RuntimeError>().unwrap(),
                    None => RuntimeError::InvokeError,
                }
            })?
            .ok_or(RuntimeError::NoReturnData)?;
        match rtn {
            RuntimeValue::I32(ptr) => Self::read_return_value(memory, ptr as u32),
            _ => Err(RuntimeError::InvalidReturnType),
        }
    }

    fn read_return_value(memory: MemoryRef, ptr: u32) -> Result<ScryptoValue, RuntimeError> {
        // read length
        let len: u32 = memory
            .get_value(ptr)
            .map_err(|_| RuntimeError::MemoryAccessError)?;

        let start = ptr.checked_add(4).ok_or(RuntimeError::MemoryAccessError)?;
        let end = start
            .checked_add(len)
            .ok_or(RuntimeError::MemoryAccessError)?;
        let range = start as usize..end as usize;
        let direct = memory.direct_access();
        let buffer = direct.as_ref();

        if end > buffer.len().try_into().unwrap() {
            return Err(RuntimeError::MemoryAccessError);
        }

        ScryptoValue::from_slice(&buffer[range]).map_err(RuntimeError::ParseScryptoValueError)
    }

    pub fn run<E: Externals>(
        func_name: &str,
        module: ModuleRef,
        memory: MemoryRef,
        externals: &mut E,
    ) -> Result<ScryptoValue, RuntimeError> {
        let result = module.invoke_export(func_name, &[], externals);

        // Return value
        let rtn = result
            .map_err(|e| {
                match e.into_host_error() {
                    // Pass-through runtime errors
                    Some(host_error) => *host_error.downcast::<RuntimeError>().unwrap(),
                    None => RuntimeError::InvokeError,
                }
            })?
            .ok_or(RuntimeError::NoReturnData)?;
        match rtn {
            RuntimeValue::I32(ptr) => Self::read_return_value(memory, ptr as u32),
            _ => Err(RuntimeError::InvalidReturnType),
        }
    }
}
