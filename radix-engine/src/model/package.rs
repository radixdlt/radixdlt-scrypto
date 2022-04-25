use sbor::*;
use scrypto::abi::{Function, Method};
use scrypto::buffer::{scrypto_decode, scrypto_encode};
use scrypto::core::{ScryptoActorInfo};
use scrypto::prelude::{PackageFunction};
use scrypto::rust::collections::HashMap;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::fmt;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::rust::format;
use scrypto::values::ScryptoValue;
use scrypto::engine::api::*;
use wasmi::{Externals, ExternVal, ImportsBuilder, MemoryRef, Module, ModuleInstance, ModuleRef, NopExternals, RuntimeArgs, RuntimeValue, Trap};
use crate::engine::*;
use crate::engine::{EnvModuleResolver, SystemApi};
use crate::errors::{RuntimeError, WasmValidationError};
use crate::model::Component;

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

    pub fn run<'a, E: SystemApi>(
        module: ModuleRef,
        memory: MemoryRef,
        actor_info: ScryptoActorInfo,
        message: ScryptoValue,
        externals: &'a mut E,
    ) -> Result<ScryptoValue, RuntimeError> {
        let func_name = actor_info.export_name().to_string();
        let mut wasm_process = WasmProcess::new(
            actor_info,
            message,
            module.clone(),
            memory.clone(),
            externals,
        );

        let result = module.invoke_export(&func_name, &[], &mut wasm_process);

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

struct WasmProcess<'a, E: SystemApi> {
    actor_info: ScryptoActorInfo,
    message: ScryptoValue,
    externals: &'a mut E,
    module: ModuleRef,
    memory: MemoryRef,
}

impl<'a, E: SystemApi> WasmProcess<'a, E> {
    pub fn new(
        actor_info: ScryptoActorInfo,
        message: ScryptoValue,
        module: ModuleRef,
        memory: MemoryRef,
        externals: &'a mut E
    ) -> Self {
        WasmProcess {
            actor_info,
            message,
            module,
            memory,
            externals,
        }
    }

    /// Handles a system call.
    fn handle<I: Decode + fmt::Debug, O: Encode + fmt::Debug>(
        &mut self,
        args: RuntimeArgs,
        handler: fn(&mut Self, input: I) -> Result<O, RuntimeError>,
    ) -> Result<Option<RuntimeValue>, Trap> {
        let input_ptr: u32 = args.nth_checked(1)?;
        let input_len: u32 = args.nth_checked(2)?;
        // SECURITY: bill before allocating memory
        let mut input_bytes = vec![0u8; input_len as usize];
        self.memory
            .get_into(input_ptr, &mut input_bytes)
            .map_err(|_| Trap::from(RuntimeError::MemoryAccessError))?;
        let input: I = scrypto_decode(&input_bytes)
            .map_err(|e| Trap::from(RuntimeError::InvalidRequestData(e)))?;

        let output: O = handler(self, input).map_err(Trap::from)?;
        let output_bytes = scrypto_encode(&output);
        let output_ptr = self.send_bytes(&output_bytes).map_err(Trap::from)?;

        Ok(Some(RuntimeValue::I32(output_ptr)))
    }

    fn handle_get_call_data(
        &mut self,
        _input: GetCallDataInput,
    ) -> Result<GetCallDataOutput, RuntimeError> {
        Ok(GetCallDataOutput {
            component: self.actor_info.component_address(),
            arg: self.message.raw.clone(),
        })
    }

    fn handle_create_component(
        &mut self,
        input: CreateComponentInput,
    ) -> Result<CreateComponentOutput, RuntimeError> {
        let component = Component::new(
            self.actor_info.package_address().clone(),
            input.blueprint_name,
            input.access_rules_list,
            input.state,
        );
        let component_address = self.externals.create_component(component)?;
        Ok(CreateComponentOutput { component_address })
    }

    fn handle_get_component_state(
        &mut self,
        input: GetComponentStateInput,
    ) -> Result<GetComponentStateOutput, RuntimeError> {
        let state = self.externals.read_component_state(input.component_address)?;
        Ok(GetComponentStateOutput { state })
    }

    fn handle_put_component_state(
        &mut self,
        input: PutComponentStateInput,
    ) -> Result<PutComponentStateOutput, RuntimeError> {
        self.externals.write_component_state(input.component_address, input.state)?;
        Ok(PutComponentStateOutput {})
    }

    fn handle_get_component_info(
        &mut self,
        input: GetComponentInfoInput,
    ) -> Result<GetComponentInfoOutput, RuntimeError> {
        let (package_address, blueprint_name) = self.externals.get_component_info(input.component_address)?;
        Ok(GetComponentInfoOutput { package_address, blueprint_name })
    }

    fn handle_create_lazy_map(
        &mut self,
        _input: CreateLazyMapInput,
    ) -> Result<CreateLazyMapOutput, RuntimeError> {
        let lazy_map_id = self.externals.create_lazy_map();
        Ok(CreateLazyMapOutput { lazy_map_id })
    }

    fn handle_get_lazy_map_entry(
        &mut self,
        input: GetLazyMapEntryInput,
    ) -> Result<GetLazyMapEntryOutput, RuntimeError> {
        let value = self.externals.read_lazy_map_entry(input.lazy_map_id, input.key)?;
        Ok(GetLazyMapEntryOutput { value })
    }

    fn handle_put_lazy_map_entry(
        &mut self,
        input: PutLazyMapEntryInput,
    ) -> Result<PutLazyMapEntryOutput, RuntimeError> {
        self.externals.write_lazy_map_entry(input.lazy_map_id, input.key, input.value)?;
        Ok(PutLazyMapEntryOutput {})
    }

    fn handle_get_actor(&mut self, _input: GetActorInput) -> Result<GetActorOutput, RuntimeError> {
        return Ok(GetActorOutput {
            actor: self.actor_info.clone(),
        });
    }

    fn handle_invoke_snode(
        &mut self,
        input: InvokeSNodeInput,
    ) -> Result<InvokeSNodeOutput, RuntimeError> {
        let arg = ScryptoValue::from_slice(&input.arg)
            .map_err(RuntimeError::ParseScryptoValueError)?;
        let result = self.externals.invoke_snode(input.snode_ref, arg)?;
        Ok(InvokeSNodeOutput { rtn: result.raw })
    }

    fn handle_generate_uuid(
        &mut self,
        _input: GenerateUuidInput,
    ) -> Result<GenerateUuidOutput, RuntimeError> {
        let uuid = self.externals.generate_uuid();
        Ok(GenerateUuidOutput { uuid })
    }

    fn handle_emit_log(&mut self, input: EmitLogInput) -> Result<EmitLogOutput, RuntimeError> {
        self.externals.emit_log(input.level, input.message);
        Ok(EmitLogOutput {})
    }

    /// Send a byte array to wasm instance.
    fn send_bytes(&mut self, bytes: &[u8]) -> Result<i32, RuntimeError> {
        let result = self.module.invoke_export(
            "scrypto_alloc",
            &[RuntimeValue::I32((bytes.len()) as i32)],
            &mut NopExternals,
        );

        if let Ok(Some(RuntimeValue::I32(ptr))) = result {
            if self.memory.set((ptr + 4) as u32, bytes).is_ok() {
                return Ok(ptr);
            }
        }

        Err(RuntimeError::MemoryAllocError)
    }
}

impl<'a, E:SystemApi> Externals for WasmProcess<'a, E> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            ENGINE_FUNCTION_INDEX => {
                let operation: u32 = args.nth_checked(0)?;
                match operation {
                    INVOKE_SNODE => self.handle(args, Self::handle_invoke_snode),
                    GET_CALL_DATA => self.handle(args, Self::handle_get_call_data),
                    CREATE_COMPONENT => self.handle(args, Self::handle_create_component),
                    GET_COMPONENT_INFO => self.handle(args, Self::handle_get_component_info),
                    GET_COMPONENT_STATE => self.handle(args, Self::handle_get_component_state),
                    PUT_COMPONENT_STATE => self.handle(args, Self::handle_put_component_state),
                    CREATE_LAZY_MAP => self.handle(args, Self::handle_create_lazy_map),
                    GET_LAZY_MAP_ENTRY => self.handle(args, Self::handle_get_lazy_map_entry),
                    PUT_LAZY_MAP_ENTRY => self.handle(args, Self::handle_put_lazy_map_entry),
                    GET_ACTOR => self.handle(args, Self::handle_get_actor),
                    GENERATE_UUID => self.handle(args, Self::handle_generate_uuid),
                    EMIT_LOG => self.handle(args, Self::handle_emit_log),
                    _ => Err(RuntimeError::InvalidRequestCode(operation).into()),
                }
            }
            _ => Err(RuntimeError::HostFunctionNotFound(index).into()),
        }
    }
}
