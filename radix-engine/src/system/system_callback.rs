use super::node_modules::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use super::payload_validation::SchemaOrigin;
use crate::blueprints::resource::AuthZone;
use crate::errors::{RuntimeError, SystemUpstreamError};
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::Message;
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
use radix_engine_interface::api::{ClientBlueprintApi, ClientObjectApi};
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::{
    Proof, ProofDropInput, FUNGIBLE_PROOF_BLUEPRINT, NON_FUNGIBLE_PROOF_BLUEPRINT, PROOF_DROP_IDENT,
};
use radix_engine_interface::schema::RefTypes;

fn validate_input<'a, Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
    service: &mut SystemService<'a, Y, V>,
    blueprint_id: BlueprintId,
    blueprint_schema: &IndexedBlueprintSchema,
    fn_ident: &str,
    with_receiver: Option<(NodeId, bool)>,
    input: &IndexedScryptoValue,
) -> Result<String, RuntimeError> {
    let function_schema =
        blueprint_schema
            .functions
            .get(fn_ident)
            .ok_or(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::FnNotFound(fn_ident.to_string()),
            ))?;

    match (&function_schema.receiver, with_receiver.as_ref()) {
        (Some(receiver_info), Some((_, direct_access))) => {
            if *direct_access != receiver_info.ref_types.contains(RefTypes::DIRECT_ACCESS) {
                return Err(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::ReceiverNotMatch(fn_ident.to_string()),
                ));
            }
        }
        (None, None) => {}
        _ => {
            return Err(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::ReceiverNotMatch(fn_ident.to_string()),
            ));
        }
    }

    service
        .validate_payload(
            input.as_slice(),
            &blueprint_schema.schema,
            function_schema.input,
            SchemaOrigin::Blueprint(blueprint_id),
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
    blueprint_id: BlueprintId,
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
            SchemaOrigin::Blueprint(blueprint_id),
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
        schema_origin: SchemaOrigin,
        schema: ScryptoSchema,
        index: LocalTypeIndex,
        can_own: bool,
    },
}

#[derive(Clone)]
pub enum FieldLockData {
    Read,
    Write {
        schema_origin: SchemaOrigin,
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
    pub blueprint_schema_cache: NonIterMap<BlueprintId, IndexedBlueprintSchema>,
    pub modules: SystemModuleMixer,
}

impl<C: SystemCallbackObject> KernelCallbackObject for SystemConfig<C> {
    type LockData = SystemLockData;

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
        value_size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_read_substate(api, lock_handle, value_size, store_access)
    }

    fn on_write_substate<Y>(
        lock_handle: LockHandle,
        value_size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_write_substate(api, lock_handle, value_size, store_access)
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

    fn before_invoke<Y>(invocation: &KernelInvocation, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_invoke(api, invocation)
    }

    fn after_invoke<Y>(output_size: usize, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_invoke(api, output_size)
    }

    fn before_push_frame<Y>(
        callee: &Actor,
        update: &mut Message,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
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
                        SystemUpstreamError::SystemFunctionCallNotAllowed,
                    ))
                }
            };

            // TODO: Load dependent resources/components

            let mut vm_instance =
                { NativeVm::create_instance(&blueprint.package_address, &[PACKAGE_CODE_ID])? };
            let output = { vm_instance.invoke(&export_name, args, &mut system)? };

            output
        } else if blueprint.package_address.eq(&TRANSACTION_PROCESSOR_PACKAGE) {
            // TODO: the above special rule can be removed if we move schema validation
            // into a kernel model, and turn it off for genesis.

            let export_name = match ident {
                FnIdent::Application(ident) => ident,
                FnIdent::System(..) => {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::SystemFunctionCallNotAllowed,
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
            let output = { vm_instance.invoke(&export_name, args, &mut system)? };

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
                    let export_name = validate_input(
                        &mut system,
                        blueprint.clone(),
                        &schema,
                        &ident,
                        receiver,
                        &args,
                    )?;
                    export_name
                }
                FnIdent::System(system_func_id) => {
                    if let Some(sys_func) = schema.virtual_lazy_load_functions.get(&system_func_id)
                    {
                        sys_func.export_name.to_string()
                    } else {
                        return Err(RuntimeError::SystemUpstreamError(
                            SystemUpstreamError::SystemFunctionCallNotAllowed,
                        ));
                    }
                }
            };

            // Execute
            let output =
                { C::invoke(&blueprint.package_address, &export_name, args, &mut system)? };

            // Validate output
            match ident {
                FnIdent::Application(ident) => {
                    validate_output(&mut system, blueprint, &schema, &ident, &output)?
                }
                FnIdent::System(..) => {
                    // TODO: Validate against virtual schema
                }
            }

            output
        };

        Ok(output)
    }

    fn on_execution_finish<Y>(update: &Message, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_execution_finish(api, update)
    }

    fn auto_drop<Y>(nodes: Vec<NodeId>, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        // Note: this function is not responsible for checking if all nodes are dropped!
        for node_id in nodes {
            let type_info = TypeInfoBlueprint::get_type(&node_id, api)?;

            match type_info {
                TypeInfoSubstate::Object(ObjectInfo { blueprint, .. }) => {
                    match (blueprint.package_address, blueprint.blueprint_name.as_str()) {
                        (RESOURCE_PACKAGE, FUNGIBLE_PROOF_BLUEPRINT) => {
                            let mut system = SystemService::new(api);
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
                            let mut system = SystemService::new(api);
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
                            // no-op
                        }
                    }
                }
                _ => {}
            }
        }

        // Note that we destroy frame's auth zone at the very end of the `auto_drop` process
        // to make sure the auth zone stack is in good state for the proof dropping above.

        // Detach proofs from the auth zone
        if let Some(auth_zone_id) = api
            .kernel_get_system()
            .modules
            .auth
            .auth_zone_stack
            .last()
            .cloned()
        {
            let handle = api.kernel_lock_substate(
                &auth_zone_id,
                OBJECT_BASE_PARTITION,
                &AuthZoneField::AuthZone.into(),
                LockFlags::MUTABLE,
                SystemLockData::Default,
            )?;
            let mut auth_zone_substate: AuthZone =
                api.kernel_read_substate(handle)?.as_typed().unwrap();
            let proofs = core::mem::replace(&mut auth_zone_substate.proofs, Vec::new());
            api.kernel_write_substate(
                handle,
                IndexedScryptoValue::from_typed(&auth_zone_substate),
            )?;
            api.kernel_drop_lock(handle)?;

            // Drop the proofs
            let mut system = SystemService::new(api);
            for proof in proofs {
                let object_info = system.get_object_info(proof.0.as_node_id())?;
                system.call_function(
                    RESOURCE_PACKAGE,
                    &object_info.blueprint.blueprint_name,
                    PROOF_DROP_IDENT,
                    scrypto_encode(&ProofDropInput { proof }).unwrap(),
                )?;
            }

            // Drop the auth zone
            api.kernel_drop_node(&auth_zone_id)?;
        }

        Ok(())
    }

    fn after_pop_frame<Y>(api: &mut Y, dropped_actor: &Actor) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_pop_frame(api, dropped_actor)
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

    fn on_allocate_node_id<Y>(entity_type: EntityType, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_allocate_node_id(api, entity_type)
    }
}
