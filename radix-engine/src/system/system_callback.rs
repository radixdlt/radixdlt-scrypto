use crate::errors::{KernelError, RuntimeError, SystemUpstreamError};
use crate::kernel::actor::{Actor, MethodActor};
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::{KernelApi, KernelInvocation};
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::module::SystemModule;
use crate::system::module_mixer::SystemModuleMixer;
use crate::system::system::SystemService;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::virtualization::VirtualizationModule;
use crate::track::interface::StoreAccessInfo;
use crate::types::*;
use crate::vm::{NativeVm, VmInvoke};
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::ClientBlueprintApi;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::{
    Proof, ProofDropInput, FUNGIBLE_PROOF_BLUEPRINT, NON_FUNGIBLE_PROOF_BLUEPRINT, PROOF_DROP_IDENT,
};
use radix_engine_interface::schema::IndexedBlueprintSchema;

fn validate_input<'a, Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
    service: &mut SystemService<'a, Y, V>,
    blueprint_schema: &IndexedBlueprintSchema,
    fn_ident: &str,
    with_receiver: bool,
    input: &IndexedScryptoValue,
) -> Result<String, RuntimeError> {
    let function_schema =
        blueprint_schema
            .functions
            .get(fn_ident)
            .ok_or(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::FunctionNotFound(fn_ident.to_string()),
            ))?;

    if function_schema.receiver.is_some() != with_receiver {
        return Err(RuntimeError::SystemUpstreamError(
            SystemUpstreamError::ReceiverNotMatch(fn_ident.to_string()),
        ));
    }

    service
        .validate_payload(
            input.as_slice(),
            &blueprint_schema.schema,
            function_schema.input,
        )
        .map_err(|err| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputSchemaNotMatch(
                fn_ident.to_string(),
                err.error_message(&blueprint_schema.schema),
            ))
        })?;

    Ok(function_schema.export_name.clone())
}

fn validate_output<'a, Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
    service: &mut SystemService<'a, Y, V>,
    blueprint_schema: &IndexedBlueprintSchema,
    fn_ident: &str,
    output: &IndexedScryptoValue,
) -> Result<(), RuntimeError> {
    let function_schema = blueprint_schema
        .functions
        .get(fn_ident)
        .expect("Checked by `validate_input`");

    service
        .validate_payload(
            output.as_slice(),
            &blueprint_schema.schema,
            function_schema.output,
        )
        .map_err(|err| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::OutputSchemaNotMatch(
                fn_ident.to_string(),
                err.error_message(&blueprint_schema.schema),
            ))
        })?;

    Ok(())
}

#[derive(Clone)]
pub enum SystemLockData {
    KeyValueEntry(KeyValueEntryLockData),
    Field(FieldLockData),
    Default,
}

impl Default for SystemLockData {
    fn default() -> Self {
        SystemLockData::Default
    }
}

#[derive(Clone)]
pub enum KeyValueEntryLockData {
    Read,
    Write {
        schema: ScryptoSchema,
        index: LocalTypeIndex,
        can_own: bool,
    },
}

#[derive(Clone)]
pub enum FieldLockData {
    Read,
    Write {
        schema: ScryptoSchema,
        index: LocalTypeIndex,
    },
}

impl SystemLockData {
    pub fn is_kv_entry(&self) -> bool {
        matches!(self, SystemLockData::KeyValueEntry(..))
    }
}

pub struct SystemConfig<C: SystemCallbackObject> {
    pub callback_obj: C,
    // TODO: We should be able to make this a more generic cache for
    // TODO: immutable substates
    pub blueprint_schema_cache: NonIterMap<Blueprint, IndexedBlueprintSchema>,
    pub modules: SystemModuleMixer,
}

impl<C: SystemCallbackObject> KernelCallbackObject for SystemConfig<C> {
    type LockData = SystemLockData;
    type CallFrameData = Actor;

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
        node_module_init: &BTreeMap<PartitionNumber, BTreeMap<SubstateKey, IndexedScryptoValue>>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_create_node(api, node_id, node_module_init)
    }

    fn before_lock_substate<Y>(
        node_id: &NodeId,
        partition_num: &PartitionNumber,
        substate_key: &SubstateKey,
        flags: &LockFlags,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_lock_substate(api, node_id, partition_num, substate_key, flags)
    }

    fn after_lock_substate<Y>(
        handle: LockHandle,
        size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_lock_substate(api, handle, store_access, size)
    }

    fn on_drop_lock<Y>(
        lock_handle: LockHandle,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_drop_lock(api, lock_handle, store_access)
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

    fn on_scan_substates<Y>(store_access: &StoreAccessInfo, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_scan_substate(api, store_access)
    }

    fn on_set_substate<Y>(store_access: &StoreAccessInfo, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_set_substate(api, store_access)
    }

    fn on_take_substates<Y>(store_access: &StoreAccessInfo, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_take_substates(api, store_access)
    }

    fn after_create_node<Y>(
        node_id: &NodeId,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_create_node(api, node_id, store_access)
    }

    fn before_invoke<Y>(
        identifier: &KernelInvocation<Actor>,
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
        match callee {
            Actor::Method(MethodActor {
                global_address,
                object_info,
                ..
            }) => {
                if let Some(address) = global_address {
                    update
                        .node_refs_to_copy
                        .insert(address.as_node_id().clone());
                }
                if let Some(blueprint_parent) = object_info.outer_object {
                    update
                        .node_refs_to_copy
                        .insert(blueprint_parent.as_node_id().clone());
                }
            }
            _ => {}
        }
        SystemModuleMixer::before_push_frame(api, callee, update, args)
    }

    fn on_execution_start<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_execution_start(api)
    }

    fn invoke_upstream<Y>(
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelApi<SystemConfig<C>>,
    {
        let mut system = SystemService::new(api);
        let receiver = system.actor_get_receiver_node_id();
        let FnIdentifier { blueprint, ident } = system.actor_get_fn_identifier()?;

        let output = if blueprint.package_address.eq(&PACKAGE_PACKAGE) {
            // TODO: Clean this up
            // Do we need to check against the abi? Probably not since we should be able to verify this
            // in the native package itself.
            let export_name = match ident {
                FnIdent::Application(ident) => ident,
                FnIdent::System(..) => {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::InvalidSystemCall,
                    ))
                }
            };

            // TODO: Load dependent resources/components

            let mut vm_instance =
                { NativeVm::create_instance(&blueprint.package_address, &[PACKAGE_CODE_ID])? };
            let output =
                { vm_instance.invoke(receiver.as_ref(), &export_name, args, &mut system)? };

            output
        } else if blueprint.package_address.eq(&TRANSACTION_PROCESSOR_PACKAGE) {
            // TODO: the above special rule can be removed if we move schema validation
            // into a kernel model, and turn it off for genesis.

            let export_name = match ident {
                FnIdent::Application(ident) => ident,
                FnIdent::System(..) => {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::InvalidSystemCall,
                    ))
                }
            };

            // TODO: Load dependent resources/components

            let mut vm_instance = {
                NativeVm::create_instance(
                    &blueprint.package_address,
                    &[TRANSACTION_PROCESSOR_CODE_ID],
                )?
            };
            let output =
                { vm_instance.invoke(receiver.as_ref(), &export_name, args, &mut system)? };

            output
        } else {
            let schema = system.get_blueprint_schema(&blueprint)?;

            // Make dependent resources/components visible

            let handle = system.kernel_lock_substate(
                blueprint.package_address.as_node_id(),
                OBJECT_BASE_PARTITION,
                &PackageField::Info.into(),
                LockFlags::read_only(),
                SystemLockData::default(),
            )?;
            system.kernel_drop_lock(handle)?;

            //  Validate input
            let export_name = match &ident {
                FnIdent::Application(ident) => {
                    let export_name =
                        validate_input(&mut system, &schema, &ident, receiver.is_some(), &args)?;
                    export_name
                }
                FnIdent::System(system_func_id) => {
                    if let Some(sys_func) = schema.virtual_lazy_load_functions.get(&system_func_id)
                    {
                        sys_func.export_name.to_string()
                    } else {
                        return Err(RuntimeError::SystemUpstreamError(
                            SystemUpstreamError::InvalidSystemCall,
                        ));
                    }
                }
            };

            // Execute
            let output = {
                C::invoke(
                    &blueprint.package_address,
                    receiver.as_ref(),
                    &export_name,
                    args,
                    &mut system,
                )?
            };

            // Validate output
            match ident {
                FnIdent::Application(ident) => {
                    validate_output(&mut system, &schema, &ident, &output)?
                }
                FnIdent::System(..) => {
                    // TODO: Validate against virtual schema
                }
            }

            output
        };

        Ok(output)
    }

    fn on_execution_finish<Y>(update: &CallFrameUpdate, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_execution_finish(api, update)
    }

    fn auto_drop<Y>(nodes: Vec<NodeId>, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        let mut system = SystemService::new(api);
        for node_id in nodes {
            if let Ok(blueprint) = system.get_object_info(&node_id).map(|x| x.blueprint) {
                match (blueprint.package_address, blueprint.blueprint_name.as_str()) {
                    (RESOURCE_PACKAGE, FUNGIBLE_PROOF_BLUEPRINT) => {
                        system.call_function(
                            RESOURCE_PACKAGE,
                            FUNGIBLE_PROOF_BLUEPRINT,
                            PROOF_DROP_IDENT,
                            scrypto_encode(&ProofDropInput {
                                proof: Proof(Own(node_id)),
                            })
                            .unwrap(),
                        )?;
                    }
                    (RESOURCE_PACKAGE, NON_FUNGIBLE_PROOF_BLUEPRINT) => {
                        system.call_function(
                            RESOURCE_PACKAGE,
                            NON_FUNGIBLE_PROOF_BLUEPRINT,
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
        partition_num: PartitionNumber,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        VirtualizationModule::on_substate_lock_fault(node_id, partition_num, offset, api)
    }

    fn on_allocate_node_id<Y>(
        entity_type: Option<EntityType>,
        virtual_node: bool,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_allocate_node_id(api, entity_type, virtual_node)
    }
}
