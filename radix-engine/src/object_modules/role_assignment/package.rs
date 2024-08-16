use crate::blueprints::models::*;
use crate::blueprints::package::PackageAuthNativeBlueprint;
use crate::blueprints::util::*;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::object_modules::role_assignment::{LockOwnerRoleEvent, SetOwnerRoleEvent};
use crate::system::system::SystemService;
use crate::system::system_callback::*;
use crate::system::system_modules::auth::{AuthError, ResolvedPermission};
use crate::system::system_substates::FieldSubstate;
use crate::{errors::*, event_schema};
use radix_blueprint_schema_init::*;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::role_assignment::*;
use radix_engine_interface::types::*;
use radix_native_sdk::runtime::Runtime;

use super::*;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum RoleAssignmentError {
    UsedReservedRole(String),
    UsedReservedSpace,
    ExceededMaxRoleNameLen { limit: usize, actual: usize },
    ExceededMaxAccessRuleDepth,
    ExceededMaxAccessRuleNodes,
    InvalidName(InvalidNameError),
    ExceededMaxRoles,
    CannotSetRoleIfNotAttached,
}

pub struct RoleAssignmentNativePackage;

impl RoleAssignmentNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let state = RoleAssignmentStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = index_map_new();
        functions.insert(
            ROLE_ASSIGNMENT_CREATE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<RoleAssignmentCreateInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<RoleAssignmentCreateOutput>(),
                ),
                export: ROLE_ASSIGNMENT_CREATE_IDENT.to_string(),
            },
        );
        functions.insert(
            ROLE_ASSIGNMENT_SET_OWNER_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<RoleAssignmentSetOwnerInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<RoleAssignmentSetOwnerOutput>(),
                ),
                export: ROLE_ASSIGNMENT_SET_OWNER_IDENT.to_string(),
            },
        );
        functions.insert(
            ROLE_ASSIGNMENT_LOCK_OWNER_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<RoleAssignmentLockOwnerInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<RoleAssignmentLockOwnerOutput>(),
                ),
                export: ROLE_ASSIGNMENT_LOCK_OWNER_IDENT.to_string(),
            },
        );
        functions.insert(
            ROLE_ASSIGNMENT_SET_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<RoleAssignmentSetInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<RoleAssignmentSetOutput>(),
                ),
                export: ROLE_ASSIGNMENT_SET_IDENT.to_string(),
            },
        );
        functions.insert(
            ROLE_ASSIGNMENT_GET_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<RoleAssignmentGetInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<RoleAssignmentGetOutput>(),
                ),
                export: ROLE_ASSIGNMENT_GET_IDENT.to_string(),
            },
        );

        let events = event_schema! {
            aggregator,
            [
                SetOwnerRoleEvent,
                SetRoleEvent,
                LockOwnerRoleEvent
            ]
        };

        let schema = generate_full_schema(aggregator);
        let blueprints = indexmap!(
            ROLE_ASSIGNMENT_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                is_transient: true,
                feature_set: indexset!(),
                dependencies: indexset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state,
                    events,
                    types: BlueprintTypeSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                    },
                    hooks: BlueprintHooksInit::default(),
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::AllowAll, // Mocked
                },
            }
        );

        PackageDefinition { blueprints }
    }

    pub fn authorization<Y: SystemBasedKernelApi>(
        global_address: &GlobalAddress,
        ident: &str,
        input: &IndexedScryptoValue,
        api: &mut SystemService<Y>,
    ) -> Result<ResolvedPermission, RuntimeError> {
        let permission = match ident {
            ROLE_ASSIGNMENT_SET_IDENT => {
                let input: RoleAssignmentSetInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let role_list = Self::resolve_update_role_method_permission(
                    global_address.as_node_id(),
                    input.module,
                    &input.role_key,
                    api,
                )?;
                ResolvedPermission::RoleList {
                    role_assignment_of: global_address.clone(),
                    role_list,
                    module_id: input.module,
                }
            }
            ROLE_ASSIGNMENT_SET_OWNER_IDENT => {
                Self::resolve_update_owner_role_method_permission(global_address.as_node_id(), api)?
            }
            ROLE_ASSIGNMENT_LOCK_OWNER_IDENT => {
                Self::resolve_update_owner_role_method_permission(global_address.as_node_id(), api)?
            }
            ROLE_ASSIGNMENT_GET_IDENT | ROLE_ASSIGNMENT_GET_OWNER_ROLE_IDENT => {
                ResolvedPermission::AllowAll
            }
            _ => {
                return Err(RuntimeError::SystemModuleError(
                    SystemModuleError::AuthError(AuthError::NoMethodMapping(FnIdentifier {
                        blueprint_id: BlueprintId::new(
                            &ROLE_ASSIGNMENT_MODULE_PACKAGE,
                            ROLE_ASSIGNMENT_BLUEPRINT,
                        ),
                        ident: ident.to_string(),
                    })),
                ));
            }
        };

        Ok(permission)
    }

    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            ROLE_ASSIGNMENT_CREATE_IDENT => {
                let input: RoleAssignmentCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::create(input.owner_role, input.roles, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ROLE_ASSIGNMENT_SET_OWNER_IDENT => {
                let input: RoleAssignmentSetOwnerInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::set_owner_role(input.rule, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ROLE_ASSIGNMENT_LOCK_OWNER_IDENT => {
                let _input: RoleAssignmentLockOwnerInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::lock_owner_role(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ROLE_ASSIGNMENT_SET_IDENT => {
                let input: RoleAssignmentSetInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::set_role(input.module, input.role_key, input.rule, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ROLE_ASSIGNMENT_GET_IDENT => {
                let input: RoleAssignmentGetInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::get_role(input.module, input.role_key, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    /// Checks if a role key is a reserved.
    ///
    /// The system has reserved all role keys starting with `_`.
    pub fn is_role_key_reserved(role_key: &RoleKey) -> bool {
        return role_key.key.starts_with("_");
    }

    /// Checks if a role key is a reserved and has been defined.
    ///
    /// Currently there are only two such roles, i.e. `OWNER_ROLE` and `SELF_ROLE`, which can be referenced in role list.
    pub fn is_role_key_reserved_and_defined(role_key: &RoleKey) -> bool {
        Self::is_role_key_reserved(role_key)
            && (role_key.key.eq(OWNER_ROLE) || role_key.key.eq(SELF_ROLE))
    }

    pub fn verify_access_rule(access_rule: &AccessRule) -> Result<(), RoleAssignmentError> {
        pub struct AccessRuleVerifier(usize);
        impl AccessRuleVisitor for AccessRuleVerifier {
            type Error = RoleAssignmentError;
            fn visit(
                &mut self,
                _node: &CompositeRequirement,
                depth: usize,
            ) -> Result<(), Self::Error> {
                // This is to protect unbounded native stack usage during authorization
                if depth > MAX_ACCESS_RULE_DEPTH {
                    return Err(RoleAssignmentError::ExceededMaxAccessRuleDepth);
                }

                self.0 += 1;

                if self.0 > MAX_COMPOSITE_REQUIREMENTS {
                    return Err(RoleAssignmentError::ExceededMaxAccessRuleNodes);
                }

                Ok(())
            }
        }

        access_rule.dfs_traverse_nodes(&mut AccessRuleVerifier(0))
    }

    fn resolve_update_owner_role_method_permission<Y: SystemBasedKernelApi>(
        receiver: &NodeId,
        api: &mut SystemService<Y>,
    ) -> Result<ResolvedPermission, RuntimeError> {
        let handle = api.kernel_open_substate(
            receiver,
            ROLE_ASSIGNMENT_BASE_PARTITION
                .at_offset(ROLE_ASSIGNMENT_FIELDS_PARTITION_OFFSET)
                .unwrap(),
            &SubstateKey::Field(0u8),
            LockFlags::read_only(),
            SystemLockData::default(),
        )?;

        let owner_role_substate: FieldSubstate<RoleAssignmentOwnerFieldPayload> =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        api.kernel_close_substate(handle)?;

        let owner_role = owner_role_substate
            .into_payload()
            .fully_update_and_into_latest_version();

        let rule = match owner_role.owner_role_entry.updater {
            OwnerRoleUpdater::None => AccessRule::DenyAll,
            OwnerRoleUpdater::Owner => owner_role.owner_role_entry.rule,
            OwnerRoleUpdater::Object => rule!(require(global_caller(GlobalAddress::new_or_panic(
                receiver.0
            )))),
        };

        Ok(ResolvedPermission::AccessRule(rule))
    }

    fn resolve_update_role_method_permission<Y: SystemBasedKernelApi>(
        receiver: &NodeId,
        module: ModuleId,
        role_key: &RoleKey,
        service: &mut SystemService<Y>,
    ) -> Result<RoleList, RuntimeError> {
        if Self::is_role_key_reserved(&role_key) || module.eq(&ModuleId::RoleAssignment) {
            return Ok(RoleList::none());
        }

        let blueprint_id = service
            .get_blueprint_info(receiver, module.into())?
            .blueprint_id;

        let auth_template = PackageAuthNativeBlueprint::get_bp_auth_template(
            blueprint_id.package_address.as_node_id(),
            &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
            service.api(),
        )?
        .method_auth;

        match auth_template {
            MethodAuthTemplate::AllowAll => Ok(RoleList::none()),
            MethodAuthTemplate::StaticRoleDefinition(roles) => match roles.roles {
                RoleSpecification::Normal(roles) => match roles.get(role_key) {
                    Some(role_list) => Ok(role_list.clone()),
                    None => Ok(RoleList::none()),
                },
                RoleSpecification::UseOuter => Ok(RoleList::none()),
            },
        }
    }

    pub fn init_system_struct(
        owner_role: OwnerRoleEntry,
        roles: IndexMap<ModuleId, RoleAssignmentInit>,
    ) -> Result<
        (
            IndexMap<u8, FieldValue>,
            IndexMap<u8, IndexMap<Vec<u8>, KVEntry>>,
        ),
        RoleAssignmentError,
    > {
        if roles.contains_key(&ModuleId::RoleAssignment) {
            return Err(RoleAssignmentError::UsedReservedSpace);
        }

        Self::verify_access_rule(&owner_role.rule)?;

        let owner_role_substate = OwnerRoleSubstate {
            owner_role_entry: owner_role.clone(),
        };

        let owner_role_field = match owner_role.updater {
            OwnerRoleUpdater::None => FieldValue::immutable(
                &RoleAssignmentOwnerFieldPayload::from_content_source(owner_role_substate),
            ),
            OwnerRoleUpdater::Owner | OwnerRoleUpdater::Object => FieldValue::new(
                &RoleAssignmentOwnerFieldPayload::from_content_source(owner_role_substate),
            ),
        };

        let mut role_entries = index_map_new();

        for (module, roles) in roles {
            if roles.data.len() > MAX_ROLES {
                return Err(RoleAssignmentError::ExceededMaxRoles);
            }

            for (role_key, role_def) in roles.data {
                if Self::is_role_key_reserved(&role_key) {
                    return Err(RoleAssignmentError::UsedReservedRole(
                        role_key.key.to_string(),
                    ));
                }
                check_name(&role_key.key).map_err(RoleAssignmentError::InvalidName)?;

                if role_key.key.len() > MAX_ROLE_NAME_LEN {
                    return Err(RoleAssignmentError::ExceededMaxRoleNameLen {
                        limit: MAX_ROLE_NAME_LEN,
                        actual: role_key.key.len(),
                    });
                }

                let module_role_key = ModuleRoleKey::new(module, role_key);

                if let Some(access_rule) = &role_def {
                    Self::verify_access_rule(access_rule)?;
                }

                let value = role_def.map(|rule| {
                    scrypto_encode(&RoleAssignmentAccessRuleEntryPayload::from_content_source(
                        rule,
                    ))
                    .unwrap()
                });

                let kv_entry = KVEntry {
                    value,
                    locked: false,
                };

                role_entries.insert(scrypto_encode(&module_role_key).unwrap(), kv_entry);
            }
        }

        Ok((
            indexmap!(RoleAssignmentField::Owner.field_index() => owner_role_field),
            indexmap!(RoleAssignmentCollection::AccessRuleKeyValue.collection_index() => role_entries),
        ))
    }

    pub(crate) fn create<Y: SystemApi<RuntimeError>>(
        owner_role: OwnerRoleEntry,
        roles: IndexMap<ModuleId, RoleAssignmentInit>,
        api: &mut Y,
    ) -> Result<Own, RuntimeError> {
        let (fields, kv_entries) = Self::init_system_struct(owner_role, roles).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(e))
        })?;

        let component_id = api.new_object(
            ROLE_ASSIGNMENT_BLUEPRINT,
            vec![],
            GenericArgs::default(),
            fields,
            kv_entries,
        )?;

        Ok(Own(component_id))
    }

    fn set_owner_role<Y: SystemApi<RuntimeError>>(
        rule: AccessRule,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        Self::verify_access_rule(&rule).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(e))
        })?;

        let handle = api.actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::MUTABLE)?;

        let mut owner_role = api
            .field_read_typed::<RoleAssignmentOwnerFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        owner_role.owner_role_entry.rule = rule.clone();
        api.field_write_typed(
            handle,
            &RoleAssignmentOwnerFieldPayload::from_content_source(owner_role),
        )?;
        api.field_close(handle)?;

        Runtime::emit_event(api, SetOwnerRoleEvent { rule })?;

        Ok(())
    }

    fn lock_owner_role<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::MUTABLE)?;
        let mut owner_role = api
            .field_read_typed::<RoleAssignmentOwnerFieldPayload>(handle)?
            .fully_update_and_into_latest_version();
        owner_role.owner_role_entry.updater = OwnerRoleUpdater::None;
        api.field_write_typed(
            handle,
            &RoleAssignmentOwnerFieldPayload::from_content_source(owner_role),
        )?;
        api.field_lock(handle)?;
        api.field_close(handle)?;

        Runtime::emit_event(api, LockOwnerRoleEvent {})?;

        Ok(())
    }

    fn set_role<Y: SystemApi<RuntimeError>>(
        module: ModuleId,
        role_key: RoleKey,
        rule: AccessRule,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        if module.eq(&ModuleId::RoleAssignment) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::RoleAssignmentError(RoleAssignmentError::UsedReservedSpace),
            ));
        }
        if Self::is_role_key_reserved(&role_key) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::RoleAssignmentError(RoleAssignmentError::UsedReservedRole(
                    role_key.key.to_string(),
                )),
            ));
        }
        if role_key.key.len() > MAX_ROLE_NAME_LEN {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::RoleAssignmentError(
                    RoleAssignmentError::ExceededMaxRoleNameLen {
                        limit: MAX_ROLE_NAME_LEN,
                        actual: role_key.key.len(),
                    },
                ),
            ));
        }
        check_name(&role_key.key).map_err(|error| {
            RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(
                RoleAssignmentError::InvalidName(error),
            ))
        })?;

        let module_role_key = ModuleRoleKey::new(module, role_key.clone());

        Self::verify_access_rule(&rule).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(e))
        })?;

        // Only allow this method to be called on attached role assignment modules.
        // This is currently implemented to prevent unbounded number of roles from
        // being created.
        api.actor_get_node_id(ACTOR_REF_GLOBAL)
            .map_err(|e| match e {
                RuntimeError::SystemError(SystemError::GlobalAddressDoesNotExist) => {
                    RuntimeError::ApplicationError(ApplicationError::RoleAssignmentError(
                        RoleAssignmentError::CannotSetRoleIfNotAttached,
                    ))
                }
                _ => e,
            })?;

        let handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            RoleAssignmentCollection::AccessRuleKeyValue.collection_index(),
            &scrypto_encode(&module_role_key).unwrap(),
            LockFlags::MUTABLE,
        )?;

        // Overwrite whatever access rule (or empty) is there
        api.key_value_entry_set_typed(
            handle,
            RoleAssignmentAccessRuleEntryPayload::from_content_source(rule.clone()),
        )?;
        api.key_value_entry_close(handle)?;

        Runtime::emit_event(api, SetRoleEvent { role_key, rule })?;

        Ok(())
    }

    pub(crate) fn get_role<Y: SystemApi<RuntimeError>>(
        module: ModuleId,
        role_key: RoleKey,
        api: &mut Y,
    ) -> Result<Option<AccessRule>, RuntimeError> {
        let module_role_key = ModuleRoleKey::new(module, role_key);

        let handle = api.actor_open_key_value_entry(
            ACTOR_STATE_SELF,
            RoleAssignmentCollection::AccessRuleKeyValue.collection_index(),
            &scrypto_encode(&module_role_key).unwrap(),
            LockFlags::read_only(),
        )?;

        let rule = api.key_value_entry_get_typed::<RoleAssignmentAccessRuleEntryPayload>(handle)?;

        api.key_value_entry_close(handle)?;

        Ok(rule.map(|v| v.fully_update_and_into_latest_version()))
    }
}

pub struct RoleAssignmentBottlenoseExtension;

impl RoleAssignmentBottlenoseExtension {
    pub fn added_functions_schema() -> (
        IndexMap<String, FunctionSchemaInit>,
        VersionedSchema<ScryptoCustomSchema>,
    ) {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let mut functions = index_map_new();
        functions.insert(
            ROLE_ASSIGNMENT_GET_OWNER_ROLE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<RoleAssignmentGetOwnerRoleInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<RoleAssignmentGetOwnerRoleOutput>(),
                ),
                export: ROLE_ASSIGNMENT_GET_OWNER_ROLE_IDENT.to_string(),
            },
        );
        let schema = generate_full_schema(aggregator);
        (functions, schema)
    }

    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            ROLE_ASSIGNMENT_GET_OWNER_ROLE_IDENT => {
                input
                    .as_typed::<RoleAssignmentGetOwnerRoleInput>()
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;

                let rtn = Self::get_owner_role(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    pub(crate) fn get_owner_role<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<OwnerRoleEntry, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            RoleAssignmentField::Owner.field_index(),
            LockFlags::read_only(),
        )?;
        let owner_role_entry = api
            .field_read_typed::<RoleAssignmentOwnerFieldPayload>(handle)?
            .fully_update_and_into_latest_version()
            .owner_role_entry;
        api.field_close(handle)?;

        Ok(owner_role_entry)
    }
}
