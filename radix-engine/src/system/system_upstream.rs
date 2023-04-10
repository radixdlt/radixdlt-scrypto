use crate::blueprints::package::PackageCodeTypeSubstate;
use crate::errors::{KernelError, RuntimeError, SystemInvokeError};
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::{KernelApi, KernelInvocation, KernelUpstream};
use crate::system::module::SystemModule;
use crate::system::module_mixer::SystemModuleMixer;
use crate::system::system_downstream::SystemDownstream;
use crate::system::system_modules::virtualization::VirtualizationModule;
use crate::types::*;
use crate::vm::wasm::{WasmEngine, WasmInstance, WasmRuntime};
use crate::vm::{NativeVm, ScryptoRuntime, ScryptoVm};
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientBlueprintApi;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::api::ClientTransactionLimitsApi;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::{
    Proof, ProofDropInput, PROOF_BLUEPRINT, PROOF_DROP_IDENT,
};
use radix_engine_interface::schema::BlueprintSchema;

fn validate_input(
    blueprint_schema: &BlueprintSchema,
    fn_ident: &str,
    with_receiver: bool,
    input: &IndexedScryptoValue,
) -> Result<String, RuntimeError> {
    let function_schema =
        blueprint_schema
            .functions
            .get(fn_ident)
            .ok_or(RuntimeError::SystemInvokeError(
                SystemInvokeError::FunctionNotFound(fn_ident.to_string()),
            ))?;

    if function_schema.receiver.is_some() != with_receiver {
        return Err(RuntimeError::SystemInvokeError(
            SystemInvokeError::ReceiverNotMatch(fn_ident.to_string()),
        ));
    }

    validate_payload_against_schema(
        input.as_slice(),
        &blueprint_schema.schema,
        function_schema.input,
    )
    .map_err(|err| {
        RuntimeError::SystemInvokeError(SystemInvokeError::InputSchemaNotMatch(
            fn_ident.to_string(),
            err.error_message(&blueprint_schema.schema),
        ))
    })?;

    Ok(function_schema.export_name.clone())
}

fn validate_output(
    blueprint_schema: &BlueprintSchema,
    fn_ident: &str,
    output: Vec<u8>,
) -> Result<IndexedScryptoValue, RuntimeError> {
    let value = IndexedScryptoValue::from_vec(output)
        .map_err(|e| RuntimeError::SystemInvokeError(SystemInvokeError::OutputDecodeError(e)))?;

    let function_schema = blueprint_schema
        .functions
        .get(fn_ident)
        .expect("Checked by `validate_input`");

    validate_payload_against_schema(
        value.as_slice(),
        &blueprint_schema.schema,
        function_schema.output,
    )
    .map_err(|err| {
        RuntimeError::SystemInvokeError(SystemInvokeError::OutputSchemaNotMatch(
            fn_ident.to_string(),
            err.error_message(&blueprint_schema.schema),
        ))
    })?;

    Ok(value)
}

#[derive(Debug)]
pub struct SystemInvocation {
    pub blueprint: Blueprint,
    pub ident: FnIdent,
    pub receiver: Option<MethodIdentifier>,
}

pub struct SystemUpstream<'g, W: WasmEngine> {
    pub scrypto_vm: &'g ScryptoVm<W>,
    pub modules: SystemModuleMixer,
}

impl<'g, W: WasmEngine + 'g> KernelUpstream for SystemUpstream<'g, W> {
    type Invocation = SystemInvocation;

    fn on_init<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_init(api)
    }

    fn on_teardown<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_teardown(api)
    }

    fn before_drop_node<Y>(node_id: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_drop_node(api, node_id)
    }

    fn after_drop_node<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_drop_node(api)
    }

    fn before_create_node<Y>(
        node_id: &NodeId,
        node_module_init: &BTreeMap<SysModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_create_node(api, node_id, node_module_init)
    }

    fn before_lock_substate<Y>(
        node_id: &NodeId,
        module_id: &SysModuleId,
        substate_key: &SubstateKey,
        flags: &LockFlags,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_lock_substate(api, node_id, module_id, substate_key, flags)
    }

    fn after_lock_substate<Y>(
        handle: LockHandle,
        size: usize,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_lock_substate(api, handle, size)
    }

    fn on_drop_lock<Y>(lock_handle: LockHandle, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_drop_lock(api, lock_handle)
    }

    fn on_read_substate<Y>(
        lock_handle: LockHandle,
        size: usize,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_read_substate(api, lock_handle, size)
    }

    fn on_write_substate<Y>(
        lock_handle: LockHandle,
        size: usize,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_write_substate(api, lock_handle, size)
    }

    fn after_create_node<Y>(node_id: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_create_node(api, node_id)
    }

    fn before_invoke<Y>(
        identifier: &KernelInvocation<SystemInvocation>,
        input_size: usize,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_invoke(api, identifier, input_size)
    }

    fn after_invoke<Y>(output_size: usize, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_invoke(api, output_size)
    }

    fn before_push_frame<Y>(
        callee: &Actor,
        update: &mut CallFrameUpdate,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_push_frame(api, callee, update, args)
    }

    fn on_execution_start<Y>(caller: &Option<Actor>, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_execution_start(api, &caller)
    }

    fn invoke_upstream<Y>(
        invocation: SystemInvocation,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelApi<SystemUpstream<'g, W>>,
    {
        let output = if invocation.blueprint.package_address.eq(&PACKAGE_PACKAGE) {
            // TODO: Clean this up
            api.kernel_load_package_package_dependencies();

            // TODO: Clean this up
            // Do we need to check against the abi? Probably not since we should be able to verify this
            // in the native package itself.
            let export_name = match invocation.ident {
                FnIdent::Application(ident) => ident,
                FnIdent::System(..) => {
                    return Err(RuntimeError::SystemInvokeError(
                        SystemInvokeError::InvalidSystemCall,
                    ))
                }
            };
            // Make dependent resources/components visible
            let handle = api.kernel_lock_substate(
                invocation.blueprint.package_address.as_node_id(),
                SysModuleId::ObjectTuple,
                &PackageOffset::Info.into(),
                LockFlags::read_only(),
            );

            if let Ok(handle) = handle {
                api.kernel_drop_lock(handle)?;
            }

            let mut system = SystemDownstream::new(api);

            NativeVm::invoke_native_package(
                PACKAGE_CODE_ID,
                &invocation.receiver,
                &export_name,
                args,
                &mut system,
            )?
        } else if invocation
            .blueprint
            .package_address
            .eq(&TRANSACTION_PROCESSOR_PACKAGE)
        {
            // TODO: the above special rule can be removed if we move schema validation
            // into a kernel model, and turn it off for genesis.
            let export_name = match invocation.ident {
                FnIdent::Application(ident) => ident,
                FnIdent::System(..) => {
                    return Err(RuntimeError::SystemInvokeError(
                        SystemInvokeError::InvalidSystemCall,
                    ))
                }
            };

            let mut system = SystemDownstream::new(api);
            NativeVm::invoke_native_package(
                TRANSACTION_PROCESSOR_CODE_ID,
                &invocation.receiver,
                &export_name,
                args,
                &mut system,
            )?
        } else {
            // Make dependent resources/components visible
            let handle = api.kernel_lock_substate(
                invocation.blueprint.package_address.as_node_id(),
                SysModuleId::ObjectTuple,
                &PackageOffset::Info.into(),
                LockFlags::read_only(),
            )?;
            api.kernel_drop_lock(handle)?;

            // TODO: Remove this weirdness or move to a kernel module if we still want to support this
            // Make common resources/components visible
            api.kernel_load_common();

            // Load schema
            let schema = {
                let handle = api.kernel_lock_substate(
                    invocation.blueprint.package_address.as_node_id(),
                    SysModuleId::ObjectTuple,
                    &PackageOffset::Info.into(),
                    LockFlags::read_only(),
                )?;
                let package_info = api.kernel_read_substate(handle)?;
                let package_info: PackageInfoSubstate = package_info.as_typed().unwrap();
                let schema = package_info
                    .schema
                    .blueprints
                    .get(&invocation.blueprint.blueprint_name)
                    .ok_or(RuntimeError::SystemInvokeError(
                        SystemInvokeError::BlueprintNotFound(invocation.blueprint.clone()),
                    ))?
                    .clone();
                api.kernel_drop_lock(handle)?;
                Box::new(schema)
            };

            //  Validate input
            let export_name = match &invocation.ident {
                FnIdent::Application(ident) => {
                    let export_name =
                        validate_input(&schema, &ident, invocation.receiver.is_some(), &args)?;
                    export_name
                }
                FnIdent::System(system_func_id) => {
                    if let Some(sys_func) = schema.virtual_lazy_load_functions.get(&system_func_id)
                    {
                        sys_func.export_name.to_string()
                    } else {
                        return Err(RuntimeError::SystemInvokeError(
                            SystemInvokeError::InvalidSystemCall,
                        ));
                    }
                }
            };

            // Interpret
            let code_type = {
                let handle = api.kernel_lock_substate(
                    invocation.blueprint.package_address.as_node_id(),
                    SysModuleId::ObjectTuple,
                    &PackageOffset::CodeType.into(),
                    LockFlags::read_only(),
                )?;
                let code_type = api.kernel_read_substate(handle)?;
                let code_type: PackageCodeTypeSubstate = code_type.as_typed().unwrap();
                api.kernel_drop_lock(handle)?;
                code_type
            };
            let output = match code_type {
                PackageCodeTypeSubstate::Native => {
                    let handle = api.kernel_lock_substate(
                        invocation.blueprint.package_address.as_node_id(),
                        SysModuleId::ObjectTuple,
                        &PackageOffset::Code.into(),
                        LockFlags::read_only(),
                    )?;
                    let code = api.kernel_read_substate(handle)?;
                    let code: PackageCodeSubstate = code.as_typed().unwrap();
                    let native_package_code_id = code.code[0];
                    api.kernel_drop_lock(handle)?;

                    let mut system = SystemDownstream::new(api);

                    NativeVm::invoke_native_package(
                        native_package_code_id,
                        &invocation.receiver,
                        &export_name,
                        args,
                        &mut system,
                    )?
                    .into()
                }
                PackageCodeTypeSubstate::Wasm => {
                    let mut wasm_instance = {
                        let handle = api.kernel_lock_substate(
                            invocation.blueprint.package_address.as_node_id(),
                            SysModuleId::ObjectTuple,
                            &PackageOffset::Code.into(),
                            LockFlags::read_only(),
                        )?;
                        // TODO: check if save to unwrap
                        let package_code: PackageCodeSubstate =
                            api.kernel_read_substate(handle)?.as_typed().unwrap();
                        api.kernel_drop_lock(handle)?;

                        let system = api.kernel_get_system();
                        let wasm_instance = system.scrypto_vm.create_instance(
                            invocation.blueprint.package_address,
                            &package_code.code,
                        );
                        wasm_instance
                    };

                    let output = {
                        let mut system = SystemDownstream::new(api);
                        let mut runtime: Box<dyn WasmRuntime> =
                            Box::new(ScryptoRuntime::new(&mut system));

                        let mut input = Vec::new();
                        if let Some(MethodIdentifier(node_id, ..)) = invocation.receiver {
                            input.push(
                                runtime
                                    .allocate_buffer(
                                        scrypto_encode(&node_id)
                                            .expect("Failed to encode component id"),
                                    )
                                    .expect("Failed to allocate buffer"),
                            );
                        }
                        input.push(
                            runtime
                                .allocate_buffer(args.as_slice().to_vec())
                                .expect("Failed to allocate buffer"),
                        );

                        wasm_instance.invoke_export(&export_name, input, &mut runtime)?
                    };

                    let mut system = SystemDownstream::new(api);
                    system.update_wasm_memory_usage(wasm_instance.consumed_memory()?)?;

                    output
                }
            };

            // Validate output
            let output = match invocation.ident {
                FnIdent::Application(ident) => validate_output(&schema, &ident, output)?,
                FnIdent::System(..) => {
                    // TODO: Validate against virtual schema
                    let value = IndexedScryptoValue::from_vec(output).map_err(|e| {
                        RuntimeError::SystemInvokeError(SystemInvokeError::OutputDecodeError(e))
                    })?;
                    value
                }
            };

            output
        };

        Ok(output)
    }

    fn on_execution_finish<Y>(
        caller: &Option<Actor>,
        update: &CallFrameUpdate,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_execution_finish(api, caller, update)
    }

    fn auto_drop<Y>(nodes: Vec<NodeId>, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        let mut system = SystemDownstream::new(api);
        for node_id in nodes {
            if let Ok(blueprint) = system.get_object_info(&node_id).map(|x| x.blueprint) {
                match (blueprint.package_address, blueprint.blueprint_name.as_str()) {
                    (RESOURCE_MANAGER_PACKAGE, PROOF_BLUEPRINT) => {
                        system.call_function(
                            RESOURCE_MANAGER_PACKAGE,
                            PROOF_BLUEPRINT,
                            PROOF_DROP_IDENT,
                            scrypto_encode(&ProofDropInput {
                                proof: Proof(Own(node_id)),
                            })
                            .unwrap(),
                        )?;
                    }
                    _ => {
                        return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                            node_id,
                        )))
                    }
                }
            } else {
                return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                    node_id,
                )));
            }
        }

        Ok(())
    }

    fn after_pop_frame<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_pop_frame(api)
    }

    fn on_substate_lock_fault<Y>(
        node_id: NodeId,
        module_id: SysModuleId,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        VirtualizationModule::on_substate_lock_fault(node_id, module_id, offset, api)
    }
}
