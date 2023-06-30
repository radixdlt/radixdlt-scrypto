use super::node_modules::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::blueprints::resource::AuthZone;
use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::{KernelApi, KernelInvocation};
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::module::SystemModule;
use crate::system::system::KeyValueEntrySubstate;
use crate::system::system::SystemService;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::SystemModuleMixer;
use crate::track::interface::StoreAccessInfo;
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::system_modules::virtualization::VirtualLazyLoadInput;
use radix_engine_interface::api::ClientBlueprintApi;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::account::{
    ACCOUNT_BLUEPRINT, ACCOUNT_CREATE_VIRTUAL_ED25519_ID, ACCOUNT_CREATE_VIRTUAL_SECP256K1_ID,
};
use radix_engine_interface::blueprints::identity::{
    IDENTITY_BLUEPRINT, IDENTITY_CREATE_VIRTUAL_ED25519_ID, IDENTITY_CREATE_VIRTUAL_SECP256K1_ID,
};
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::{
    Proof, ProofDropInput, FUNGIBLE_PROOF_BLUEPRINT, NON_FUNGIBLE_PROOF_BLUEPRINT, PROOF_DROP_IDENT,
};
use radix_engine_interface::schema::{InstanceSchema, RefTypes};

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
    BlueprintWrite {
        blueprint_id: BlueprintId,
        instance_schema: Option<InstanceSchema>,
        type_pointer: TypePointer,
        can_own: bool,
    },
}

#[derive(Clone)]
pub enum FieldLockData {
    Read,
    Write {
        blueprint_id: BlueprintId,
        type_pointer: TypePointer,
    },
}

impl SystemLockData {
    pub fn is_kv_entry(&self) -> bool {
        matches!(self, SystemLockData::KeyValueEntry(..))
    }
}

pub struct SystemConfig<C: SystemCallbackObject> {
    pub callback_obj: C,
    pub blueprint_cache: NonIterMap<CanonicalBlueprintId, BlueprintDefinition>,
    pub schema_cache: NonIterMap<Hash, ScryptoSchema>,
    pub auth_cache: NonIterMap<CanonicalBlueprintId, AuthConfig>,
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

    fn after_drop_node<Y>(api: &mut Y, total_substate_size: usize) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_drop_node(api, total_substate_size)
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

    fn before_open_substate<Y>(
        node_id: &NodeId,
        partition_num: &PartitionNumber,
        substate_key: &SubstateKey,
        flags: &LockFlags,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::before_open_substate(api, node_id, partition_num, substate_key, flags)
    }

    fn after_open_substate<Y>(
        handle: LockHandle,
        node_id: &NodeId,
        size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_open_substate(api, handle, node_id, store_access, size)
    }

    fn on_close_substate<Y>(
        lock_handle: LockHandle,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_close_substate(api, lock_handle, store_access)
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

    fn on_set_substate<Y>(
        value_size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_set_substate(api, value_size, store_access)
    }

    fn on_take_substates<Y>(store_access: &StoreAccessInfo, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_take_substates(api, store_access)
    }

    fn after_create_node<Y>(
        node_id: &NodeId,
        total_substate_size: usize,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_create_node(api, node_id, total_substate_size, store_access)
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

    fn on_execution_finish<Y>(update: &Message, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_execution_finish(api, update)
    }

    fn after_pop_frame<Y>(api: &mut Y, dropped_actor: &Actor) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_pop_frame(api, dropped_actor)
    }

    fn on_allocate_node_id<Y>(entity_type: EntityType, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::on_allocate_node_id(api, entity_type)
    }

    fn after_move_modules<Y>(
        src_node_id: &NodeId,
        dest_node_id: &NodeId,
        store_access: &StoreAccessInfo,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        SystemModuleMixer::after_move_modules(api, src_node_id, dest_node_id, store_access)
    }

    //--------------------------------------------------------------------------
    // Note that the following logic doesn't go through mixer and is not costed
    //--------------------------------------------------------------------------

    fn invoke_upstream<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelApi<SystemConfig<C>>,
    {
        let mut system = SystemService::new(api);
        let receiver = system.actor_get_receiver_node_id();
        let FnIdentifier {
            blueprint_id,
            ident,
        } = system.actor_get_fn_identifier()?;

        let output = {
            // Make dependent resources/components visible
            let key = BlueprintVersionKey {
                blueprint: blueprint_id.blueprint_name.clone(),
                version: BlueprintVersion::default(),
            };

            let handle = system.kernel_open_substate_with_default(
                blueprint_id.package_address.as_node_id(),
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINT_DEPENDENCIES_PARTITION_OFFSET)
                    .unwrap(),
                &SubstateKey::Map(scrypto_encode(&key).unwrap()),
                LockFlags::read_only(),
                Some(|| {
                    let kv_entry = KeyValueEntrySubstate::<()>::default();
                    IndexedScryptoValue::from_typed(&kv_entry)
                }),
                SystemLockData::default(),
            )?;
            system.kernel_read_substate(handle)?;
            system.kernel_close_substate(handle)?;

            //  Validate input
            let definition = system.get_blueprint_definition(
                blueprint_id.package_address,
                &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
            )?;

            let export = match &ident {
                FnIdent::Application(ident) => {
                    let input_type_pointer = definition
                        .interface
                        .get_function_input_type_pointer(ident.as_str())
                        .ok_or_else(|| {
                            RuntimeError::SystemUpstreamError(SystemUpstreamError::FnNotFound(
                                ident.to_string(),
                            ))
                        })?;

                    system.validate_payload_against_blueprint_schema(
                        &blueprint_id,
                        &None,
                        &[(input.as_vec_ref(), input_type_pointer)],
                    )?;

                    let function_schema = definition
                        .interface
                        .functions
                        .get(ident)
                        .expect("Should exist due to schema check");

                    match (&function_schema.receiver, receiver) {
                        (Some(receiver_info), Some((_, direct_access))) => {
                            if direct_access
                                != receiver_info.ref_types.contains(RefTypes::DIRECT_ACCESS)
                            {
                                return Err(RuntimeError::SystemUpstreamError(
                                    SystemUpstreamError::ReceiverNotMatch(ident.to_string()),
                                ));
                            }
                        }
                        (None, None) => {}
                        _ => {
                            return Err(RuntimeError::SystemUpstreamError(
                                SystemUpstreamError::ReceiverNotMatch(ident.to_string()),
                            ));
                        }
                    }

                    definition
                        .function_exports
                        .get(ident)
                        .expect("Schema should have validated this exists")
                        .clone()
                }
                FnIdent::System(system_func_id) => {
                    if let Some(package_export) =
                        definition.virtual_lazy_load_functions.get(&system_func_id)
                    {
                        package_export.clone()
                    } else {
                        return Err(RuntimeError::SystemUpstreamError(
                            SystemUpstreamError::SystemFunctionCallNotAllowed,
                        ));
                    }
                }
            };

            // Execute
            let output = { C::invoke(&blueprint_id.package_address, export, input, &mut system)? };

            // Validate output
            match ident {
                FnIdent::Application(ident) => {
                    let output_type_pointer = definition
                        .interface
                        .get_function_output_type_pointer(ident.as_str())
                        .expect("Schema verification should enforce that this exists.");

                    system.validate_payload_against_blueprint_schema(
                        &blueprint_id,
                        &None,
                        &[(output.as_vec_ref(), output_type_pointer)],
                    )?;
                }
                FnIdent::System(..) => {
                    // FIXME: Validate against virtual schema
                }
            }

            output
        };

        Ok(output)
    }

    fn auto_drop<Y>(nodes: Vec<NodeId>, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        // Note: this function is not responsible for checking if all nodes are dropped!
        for node_id in nodes {
            let type_info = TypeInfoBlueprint::get_type(&node_id, api)?;

            match type_info {
                TypeInfoSubstate::Object(ObjectInfo {
                    blueprint_id: blueprint,
                    ..
                }) => {
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
        if let Some(auth_zone_id) = api.kernel_get_system().modules.auth_zone_id() {
            let handle = api.kernel_open_substate(
                &auth_zone_id,
                MAIN_BASE_PARTITION,
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
            api.kernel_close_substate(handle)?;

            // Drop the proofs
            let mut system = SystemService::new(api);
            for proof in proofs {
                let object_info = system.get_object_info(proof.0.as_node_id())?;
                system.call_function(
                    RESOURCE_PACKAGE,
                    &object_info.blueprint_id.blueprint_name,
                    PROOF_DROP_IDENT,
                    scrypto_encode(&ProofDropInput { proof }).unwrap(),
                )?;
            }

            // Drop the auth zone
            api.kernel_drop_node(&auth_zone_id)?;
        }

        Ok(())
    }

    fn on_substate_lock_fault<Y>(
        node_id: NodeId,
        _partition_num: PartitionNumber,
        _offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: KernelApi<Self>,
    {
        match node_id.entity_type() {
            // FIXME: Need to have a schema check in place before this in order to not create virtual components when accessing illegal substates
            Some(entity_type) => {
                // Lazy create component if missing
                let (blueprint, virtual_func_id) = match entity_type {
                    EntityType::GlobalVirtualSecp256k1Account => (
                        BlueprintId::new(&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT),
                        ACCOUNT_CREATE_VIRTUAL_SECP256K1_ID,
                    ),
                    EntityType::GlobalVirtualEd25519Account => (
                        BlueprintId::new(&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT),
                        ACCOUNT_CREATE_VIRTUAL_ED25519_ID,
                    ),
                    EntityType::GlobalVirtualSecp256k1Identity => (
                        BlueprintId::new(&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT),
                        IDENTITY_CREATE_VIRTUAL_SECP256K1_ID,
                    ),
                    EntityType::GlobalVirtualEd25519Identity => (
                        BlueprintId::new(&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT),
                        IDENTITY_CREATE_VIRTUAL_ED25519_ID,
                    ),
                    _ => return Ok(false),
                };

                let mut args = [0u8; NodeId::RID_LENGTH];
                args.copy_from_slice(&node_id.as_ref()[1..]);

                let invocation = KernelInvocation {
                    actor: Actor::VirtualLazyLoad {
                        blueprint_id: blueprint.clone(),
                        ident: virtual_func_id,
                    },
                    args: IndexedScryptoValue::from_typed(&VirtualLazyLoadInput { id: args }),
                };

                let rtn: Vec<u8> = api.kernel_invoke(Box::new(invocation))?.into();

                let modules: BTreeMap<ObjectModuleId, Own> = scrypto_decode(&rtn).unwrap();
                let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();
                let address = GlobalAddress::new_or_panic(node_id.into());

                let mut system = SystemService::new(api);
                let address_reservation =
                    system.allocate_virtual_global_address(blueprint, address)?;
                system.globalize(modules, Some(address_reservation))?;

                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
