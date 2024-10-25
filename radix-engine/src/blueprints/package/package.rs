use super::substates::*;
use crate::blueprints::util::{check_name, InvalidNameError, SecurifiedRoleAssignment};
use crate::internal_prelude::*;
use crate::object_modules::metadata::{validate_metadata_init, MetadataNativePackage};
use crate::system::node_init::type_info_partition;
use crate::system::system_modules::costing::{apply_royalty_cost, RoyaltyRecipient};
use crate::system::type_info::TypeInfoSubstate;
use crate::track::interface::NodeSubstates;
use crate::vm::wasm::PrepareError;
use radix_blueprint_schema_init::*;
use radix_engine_interface::api::*;
pub use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::{require, Bucket};
use radix_engine_interface::object_modules::metadata::MetadataInit;
use radix_engine_interface::object_modules::role_assignment::ROLE_ASSIGNMENT_BLUEPRINT;
use radix_native_sdk::modules::metadata::Metadata;
use radix_native_sdk::modules::role_assignment::RoleAssignment;
use radix_native_sdk::resource::NativeVault;
use radix_native_sdk::resource::ResourceManager;
use sbor::LocalTypeId;

// Import and re-export substate types
use crate::object_modules::role_assignment::*;
use crate::object_modules::royalty::RoyaltyUtil;
use crate::roles_template;
use crate::system::system::*;
use crate::system::system_callback::*;
use crate::system::system_modules::auth::{AuthError, ResolvedPermission};
use crate::system::system_type_checker::SystemMapper;
use crate::vm::{VmApi, VmPackageValidation};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Sbor)]
pub enum PackageV1MinorVersion {
    Zero,
    One,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PackageError {
    InvalidWasm(PrepareError),

    InvalidBlueprintSchema(SchemaValidationError),
    TooManySubstateSchemas,
    FeatureDoesNotExist(String),
    InvalidTransientField,
    SystemInstructionsNotSupported,

    FailedToResolveLocalSchema {
        local_type_id: LocalTypeId,
    },
    EventNameMismatch {
        expected: String,
        actual: Option<String>,
    },
    TypeNameMismatch {
        expected: String,
        actual: Option<String>,
    },
    InvalidEventSchema,
    InvalidSystemFunction,
    InvalidTypeParent,
    InvalidName(InvalidNameError),
    MissingOuterBlueprint,
    WasmUnsupported(String),
    InvalidLocalTypeId(LocalTypeId),
    InvalidGenericId(u8),
    EventGenericTypeNotSupported,
    OuterBlueprintCantBeAnInnerBlueprint {
        inner: String,
        violating_outer: String,
    },
    RoleAssignmentError(RoleAssignmentError),

    InvalidAuthSetup,
    DefiningReservedRoleKey(String, RoleKey),
    ExceededMaxRoles {
        limit: usize,
        actual: usize,
    },
    ExceededMaxRoleNameLen {
        limit: usize,
        actual: usize,
    },
    ExceededMaxBlueprintNameLen {
        limit: usize,
        actual: usize,
    },
    ExceededMaxEventNameLen {
        limit: usize,
        actual: usize,
    },
    ExceededMaxTypeNameLen {
        limit: usize,
        actual: usize,
    },
    ExceededMaxFunctionNameLen {
        limit: usize,
        actual: usize,
    },
    ExceededMaxFeatureNameLen {
        limit: usize,
        actual: usize,
    },
    MissingRole(RoleKey),
    UnexpectedNumberOfMethodAuth {
        blueprint: String,
        expected: usize,
        actual: usize,
    },
    MissingMethodPermission {
        blueprint: String,
        ident: String,
    },

    UnexpectedNumberOfFunctionAuth {
        blueprint: String,
        expected: usize,
        actual: usize,
    },
    MissingFunctionPermission {
        blueprint: String,
        ident: String,
    },

    UnexpectedNumberOfFunctionRoyalties {
        blueprint: String,
        expected: usize,
        actual: usize,
    },
    MissingFunctionRoyalty {
        blueprint: String,
        ident: String,
    },
    RoyaltyAmountIsGreaterThanAllowed {
        max: RoyaltyAmount,
        actual: RoyaltyAmount,
    },

    InvalidMetadataKey(String),

    RoyaltiesNotEnabled,
    RoyaltyAmountIsNegative(RoyaltyAmount),

    ReservedRoleKeyIsNotDefined(String),
}

impl From<InvalidNameError> for PackageError {
    fn from(error: InvalidNameError) -> Self {
        Self::InvalidName(error)
    }
}

fn validate_package_schema(
    blueprints: &IndexMap<String, BlueprintDefinitionInit>,
) -> Result<(), PackageError> {
    for (bp_name, bp_def) in blueprints.iter() {
        let bp_schema = &bp_def.schema;

        match &bp_def.blueprint_type {
            BlueprintType::Outer => Ok(()),
            BlueprintType::Inner { outer_blueprint } if outer_blueprint != bp_name => {
                match blueprints
                    .get(outer_blueprint)
                    .map(|bp_def| &bp_def.blueprint_type)
                {
                    Some(BlueprintType::Outer) => Ok(()),
                    Some(BlueprintType::Inner { .. }) => {
                        Err(PackageError::OuterBlueprintCantBeAnInnerBlueprint {
                            inner: bp_name.clone(),
                            violating_outer: outer_blueprint.clone(),
                        })
                    }
                    None => Err(PackageError::MissingOuterBlueprint),
                }
            }
            BlueprintType::Inner { .. } => Err(PackageError::MissingOuterBlueprint),
        }?;

        validate_schema(bp_schema.schema.v1())
            .map_err(|e| PackageError::InvalidBlueprintSchema(e))?;

        if bp_schema.state.fields.len() > MAX_NUMBER_OF_BLUEPRINT_FIELDS {
            return Err(PackageError::TooManySubstateSchemas);
        }

        for field in &bp_schema.state.fields {
            validate_package_schema_type_ref(bp_schema, field.field)?;

            match &field.condition {
                Condition::IfFeature(feature) => {
                    if !bp_def.feature_set.contains(feature) {
                        return Err(PackageError::FeatureDoesNotExist(feature.clone()));
                    }
                }
                Condition::IfOuterFeature(feature) => match &bp_def.blueprint_type {
                    BlueprintType::Inner { outer_blueprint } => {
                        if let Some(outer_bp_def) = blueprints.get(outer_blueprint) {
                            if !outer_bp_def.feature_set.contains(feature) {
                                return Err(PackageError::FeatureDoesNotExist(feature.clone()));
                            }
                        } else {
                            // It's impossible for us to get to this point here. We have checked
                            // earlier in this same function that each inner blueprint has an outer
                            // blueprint. Thus, we can't get to this point if this invariant was not
                            // upheld.
                            return Err(PackageError::FeatureDoesNotExist(feature.clone()));
                        }
                    }
                    _ => {
                        return Err(PackageError::FeatureDoesNotExist(feature.clone()));
                    }
                },
                Condition::Always => {}
            }

            match &field.transience {
                FieldTransience::NotTransient => {}
                FieldTransience::TransientStatic { default_value } => match field.field {
                    TypeRef::Static(local_index) => {
                        validate_payload_against_schema::<ScryptoCustomExtension, ()>(
                            default_value,
                            bp_schema.schema.v1(),
                            local_index,
                            &mut (),
                            TRANSIENT_SUBSTATE_DEFAULT_VALUE_MAX_DEPTH,
                        )
                        .map_err(|_| PackageError::InvalidTransientField)?;
                    }
                    TypeRef::Generic(..) => return Err(PackageError::InvalidTransientField),
                },
            }
        }

        for collection in &bp_schema.state.collections {
            match collection {
                BlueprintCollectionSchema::KeyValueStore(kv_store_schema) => {
                    validate_package_schema_type_ref(bp_schema, kv_store_schema.key)?;
                    validate_package_schema_type_ref(bp_schema, kv_store_schema.value)?;
                }
                BlueprintCollectionSchema::SortedIndex(kv_store_schema) => {
                    validate_package_schema_type_ref(bp_schema, kv_store_schema.key)?;
                    validate_package_schema_type_ref(bp_schema, kv_store_schema.value)?;
                }
                BlueprintCollectionSchema::Index(kv_store_schema) => {
                    validate_package_schema_type_ref(bp_schema, kv_store_schema.key)?;
                    validate_package_schema_type_ref(bp_schema, kv_store_schema.value)?;
                }
            }
        }

        for (_name, event) in &bp_schema.events.event_schema {
            validate_package_schema_type_ref(bp_schema, *event)?;
        }

        for (_name, function) in &bp_schema.functions.functions {
            validate_package_schema_type_ref(bp_schema, function.input)?;
            validate_package_schema_type_ref(bp_schema, function.output)?;
        }
    }

    Ok(())
}

fn validate_package_schema_type_ref(
    radix_blueprint_schema_init: &BlueprintSchemaInit,
    type_ref: TypeRef<LocalTypeId>,
) -> Result<(), PackageError> {
    match type_ref {
        TypeRef::Static(local_type_id) => {
            if radix_blueprint_schema_init
                .schema
                .v1()
                .resolve_type_kind(local_type_id)
                .is_some()
            {
                Ok(())
            } else {
                Err(PackageError::InvalidLocalTypeId(local_type_id))
            }
        }
        TypeRef::Generic(generic_id) => {
            if (generic_id as usize) < radix_blueprint_schema_init.generics.len() {
                Ok(())
            } else {
                Err(PackageError::InvalidGenericId(generic_id))
            }
        }
    }
}

fn extract_package_event_static_type_id(
    blueprint_init: &BlueprintSchemaInit,
    type_ref: TypeRef<LocalTypeId>,
) -> Result<LocalTypeId, PackageError> {
    match type_ref {
        TypeRef::Static(local_type_id) => {
            if blueprint_init
                .schema
                .v1()
                .resolve_type_kind(local_type_id)
                .is_some()
            {
                Ok(local_type_id)
            } else {
                Err(PackageError::InvalidLocalTypeId(local_type_id))
            }
        }
        TypeRef::Generic(_) => Err(PackageError::EventGenericTypeNotSupported),
    }
}

fn validate_event_schemas<'a, I: Iterator<Item = &'a BlueprintDefinitionInit>>(
    blueprints: I,
) -> Result<(), PackageError> {
    for blueprint_init in blueprints {
        let radix_blueprint_schema_init = &blueprint_init.schema;
        let BlueprintSchemaInit { schema, events, .. } = radix_blueprint_schema_init;

        for (expected_event_name, type_ref) in events.event_schema.iter() {
            let local_type_id =
                extract_package_event_static_type_id(radix_blueprint_schema_init, *type_ref)?;

            // Checking that the event is either a struct or an enum
            let type_kind = schema.v1().resolve_type_kind(local_type_id).map_or(
                Err(PackageError::FailedToResolveLocalSchema { local_type_id }),
                Ok,
            )?;
            match type_kind {
                // Structs and Enums are allowed
                TypeKind::Enum { .. } | TypeKind::Tuple { .. } => Ok(()),
                _ => Err(PackageError::InvalidEventSchema),
            }?;

            // Checking that the event name is indeed what the user claims it to be
            let actual_event_name = schema.v1().resolve_type_metadata(local_type_id).map_or(
                Err(PackageError::FailedToResolveLocalSchema {
                    local_type_id: local_type_id,
                }),
                |metadata| Ok(metadata.get_name_string()),
            )?;

            if Some(expected_event_name) != actual_event_name.as_ref() {
                Err(PackageError::EventNameMismatch {
                    expected: expected_event_name.to_string(),
                    actual: actual_event_name,
                })?
            }
        }
    }

    Ok(())
}

fn validate_type_schemas<'a, I: Iterator<Item = &'a BlueprintDefinitionInit>>(
    blueprints: I,
) -> Result<(), PackageError> {
    for blueprint_init in blueprints {
        let radix_blueprint_schema_init = &blueprint_init.schema;
        let BlueprintSchemaInit { schema, types, .. } = radix_blueprint_schema_init;

        for (_, local_type_id) in types.type_schema.iter() {
            if schema.v1().resolve_type_kind(*local_type_id).is_none() {
                return Err(PackageError::InvalidLocalTypeId(*local_type_id));
            }

            // Notes:
            // - The "type name" length and char check is done within `validate_names`
            // - We do no require the type identifier to be equal to the type name in metadata
        }
    }

    Ok(())
}

fn validate_royalties<Y: SystemApi<RuntimeError>>(
    definition: &PackageDefinition,
    api: &mut Y,
) -> Result<(), RuntimeError> {
    for (blueprint, definition_init) in &definition.blueprints {
        match &definition_init.royalty_config {
            PackageRoyaltyConfig::Disabled => {}
            PackageRoyaltyConfig::Enabled(function_royalties) => {
                let num_functions = definition_init.schema.functions.functions.len();

                if num_functions != function_royalties.len() {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::PackageError(
                            PackageError::UnexpectedNumberOfFunctionRoyalties {
                                blueprint: blueprint.clone(),
                                expected: num_functions,
                                actual: function_royalties.len(),
                            },
                        ),
                    ));
                }

                for name in definition_init.schema.functions.functions.keys() {
                    if !function_royalties.contains_key(name) {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::PackageError(PackageError::MissingFunctionRoyalty {
                                blueprint: blueprint.clone(),
                                ident: name.clone(),
                            }),
                        ));
                    }
                }

                RoyaltyUtil::verify_royalty_amounts(function_royalties.values(), false, api)?;
            }
        }
    }

    Ok(())
}

fn validate_auth(
    definition: &PackageDefinition,
    restrict_reserved_key: bool,
) -> Result<(), PackageError> {
    for (blueprint, definition_init) in &definition.blueprints {
        match &definition_init.auth_config.function_auth {
            FunctionAuth::AllowAll | FunctionAuth::RootOnly => {}
            FunctionAuth::AccessRules(functions) => {
                let num_functions = definition_init
                    .schema
                    .functions
                    .functions
                    .values()
                    .filter(|schema| schema.receiver.is_none())
                    .count();

                if num_functions != functions.len() {
                    return Err(PackageError::UnexpectedNumberOfFunctionAuth {
                        blueprint: blueprint.clone(),
                        expected: num_functions,
                        actual: functions.len(),
                    });
                }

                for (name, schema_init) in &definition_init.schema.functions.functions {
                    if schema_init.receiver.is_none() && !functions.contains_key(name) {
                        return Err(PackageError::MissingFunctionPermission {
                            blueprint: blueprint.clone(),
                            ident: name.clone(),
                        });
                    }
                }

                for access_rule in functions.values() {
                    RoleAssignmentNativePackage::verify_access_rule(access_rule)
                        .map_err(PackageError::RoleAssignmentError)?;
                }
            }
        }

        match (
            &definition_init.blueprint_type,
            &definition_init.auth_config.method_auth,
        ) {
            (_, MethodAuthTemplate::AllowAll) => {}
            (
                blueprint_type,
                MethodAuthTemplate::StaticRoleDefinition(StaticRoleDefinition { roles, methods }),
            ) => {
                let role_specification = match (blueprint_type, roles) {
                    (_, RoleSpecification::Normal(roles)) => roles,
                    (BlueprintType::Inner { outer_blueprint }, RoleSpecification::UseOuter) => {
                        if let Some(blueprint) = definition.blueprints.get(outer_blueprint) {
                            match &blueprint.auth_config.method_auth {
                                MethodAuthTemplate::StaticRoleDefinition(
                                    StaticRoleDefinition {
                                        roles: RoleSpecification::Normal(roles),
                                        ..
                                    },
                                ) => roles,
                                _ => return Err(PackageError::InvalidAuthSetup),
                            }
                        } else {
                            return Err(PackageError::InvalidAuthSetup);
                        }
                    }
                    _ => {
                        return Err(PackageError::InvalidAuthSetup);
                    }
                };

                let check_list = |list: &RoleList| {
                    for role_key in &list.list {
                        if RoleAssignmentNativePackage::is_role_key_reserved(role_key) {
                            if restrict_reserved_key
                                && !RoleAssignmentNativePackage::is_role_key_reserved_and_defined(
                                    role_key,
                                )
                            {
                                return Err(PackageError::ReservedRoleKeyIsNotDefined(
                                    role_key.key.clone(),
                                ));
                            }
                            continue;
                        }
                        if !role_specification.contains_key(role_key) {
                            return Err(PackageError::MissingRole(role_key.clone()));
                        }
                    }
                    Ok(())
                };

                if let RoleSpecification::Normal(roles) = roles {
                    if roles.len() > MAX_ROLES {
                        return Err(PackageError::ExceededMaxRoles {
                            limit: MAX_ROLES,
                            actual: roles.len(),
                        });
                    }

                    for (role_key, role_list) in roles {
                        check_list(role_list)?;

                        if RoleAssignmentNativePackage::is_role_key_reserved(role_key) {
                            return Err(PackageError::DefiningReservedRoleKey(
                                blueprint.to_string(),
                                role_key.clone(),
                            ));
                        }
                    }
                }

                for (_method, accessibility) in methods {
                    match accessibility {
                        MethodAccessibility::RoleProtected(role_list) => {
                            check_list(role_list)?;
                        }
                        MethodAccessibility::OwnPackageOnly
                        | MethodAccessibility::Public
                        | MethodAccessibility::OuterObjectOnly => {}
                    }
                }

                let num_methods = definition_init
                    .schema
                    .functions
                    .functions
                    .values()
                    .filter(|schema| schema.receiver.is_some())
                    .count();

                if num_methods != methods.len() {
                    return Err(PackageError::UnexpectedNumberOfMethodAuth {
                        blueprint: blueprint.clone(),
                        expected: num_methods,
                        actual: methods.len(),
                    });
                }

                for (name, schema_init) in &definition_init.schema.functions.functions {
                    if schema_init.receiver.is_some()
                        && !methods.contains_key(&MethodKey::new(name))
                    {
                        return Err(PackageError::MissingMethodPermission {
                            blueprint: blueprint.clone(),
                            ident: name.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

fn validate_names(definition: &PackageDefinition) -> Result<(), PackageError> {
    for (bp_name, bp_init) in definition.blueprints.iter() {
        check_name(bp_name)?;
        if bp_name.len() > MAX_BLUEPRINT_NAME_LEN {
            return Err(PackageError::ExceededMaxBlueprintNameLen {
                limit: MAX_BLUEPRINT_NAME_LEN,
                actual: bp_name.len(),
            });
        }

        for (name, _) in bp_init.schema.events.event_schema.iter() {
            if name.len() > MAX_EVENT_NAME_LEN {
                return Err(PackageError::ExceededMaxEventNameLen {
                    limit: MAX_EVENT_NAME_LEN,
                    actual: name.len(),
                });
            }

            check_name(name)?;
        }

        for (name, _) in bp_init.schema.types.type_schema.iter() {
            if name.len() > MAX_TYPE_NAME_LEN {
                return Err(PackageError::ExceededMaxTypeNameLen {
                    limit: MAX_TYPE_NAME_LEN,
                    actual: name.len(),
                });
            }

            check_name(name)?;
        }

        for (name, _) in bp_init.schema.functions.functions.iter() {
            if name.len() > MAX_FUNCTION_NAME_LEN {
                return Err(PackageError::ExceededMaxFunctionNameLen {
                    limit: MAX_FUNCTION_NAME_LEN,
                    actual: name.len(),
                });
            }

            check_name(name)?;
        }

        for (_, export_name) in bp_init.schema.hooks.hooks.iter() {
            check_name(export_name)?;
        }

        for name in bp_init.feature_set.iter() {
            if name.len() > MAX_FEATURE_NAME_LEN {
                return Err(PackageError::ExceededMaxFeatureNameLen {
                    limit: MAX_FEATURE_NAME_LEN,
                    actual: name.len(),
                });
            }

            check_name(name)?;
        }

        if let PackageRoyaltyConfig::Enabled(list) = &bp_init.royalty_config {
            for (name, _) in list.iter() {
                check_name(name)?;
            }
        }

        if let FunctionAuth::AccessRules(list) = &bp_init.auth_config.function_auth {
            for (name, _) in list.iter() {
                check_name(name)?;
            }
        }

        if let MethodAuthTemplate::StaticRoleDefinition(static_roles) =
            &bp_init.auth_config.method_auth
        {
            if let RoleSpecification::Normal(list) = &static_roles.roles {
                for (role_key, _) in list.iter() {
                    if role_key.key.len() > MAX_ROLE_NAME_LEN {
                        return Err(PackageError::ExceededMaxRoleNameLen {
                            limit: MAX_ROLE_NAME_LEN,
                            actual: role_key.key.len(),
                        });
                    }
                    check_name(&role_key.key)?;
                }
            }

            for (key, accessibility) in static_roles.methods.iter() {
                check_name(&key.ident)?;
                if let MethodAccessibility::RoleProtected(role_list) = accessibility {
                    for role_key in &role_list.list {
                        check_name(&role_key.key)?;
                    }
                }
            }
        }
    }

    Ok(())
}

const SECURIFY_OWNER_ROLE: &str = "securify_owner";

struct SecurifiedPackage;

impl SecurifiedRoleAssignment for SecurifiedPackage {
    type OwnerBadgeNonFungibleData = PackageOwnerBadgeData;
    const OWNER_BADGE: ResourceAddress = PACKAGE_OWNER_BADGE;
}

fn blueprint_state_schema(
    package: PackageDefinition,
    blueprint_name: &str,
    system_mappings: IndexMap<usize, PartitionNumber>,
) -> IndexedStateSchema {
    let package_blueprint = package.blueprints.get(blueprint_name).unwrap();
    IndexedStateSchema::from_schema(
        package_blueprint.schema.schema.generate_schema_hash(),
        package_blueprint.schema.state.clone(),
        system_mappings,
    )
}

pub fn create_package_partition_substates(
    package_structure: PackageStructure,
    metadata: MetadataInit,
    royalty_vault: Option<Vault>,
) -> NodeSubstates {
    let mut node_substates = NodeSubstates::new();

    let own_features = PackageFeatureSet {
        package_royalty: royalty_vault.is_some(),
    };

    //-----------------
    // MAIN PARTITIONS:
    //-----------------

    {
        // Note: We don't include royalty field because it's been disabled

        let package_schema = blueprint_state_schema(
            PackageNativePackage::definition(),
            PACKAGE_BLUEPRINT,
            indexmap!(PackageCollection::SchemaKeyValue.collection_index() as usize => SCHEMAS_PARTITION),
        );
        let package_system_struct =
            PackageNativePackage::init_system_struct(royalty_vault, package_structure);
        let package_substates = SystemMapper::system_struct_to_node_substates(
            &package_schema,
            package_system_struct,
            MAIN_BASE_PARTITION,
        );
        node_substates.extend(package_substates);
    }

    //-------------------
    // MODULE PARTITIONS:
    //-------------------

    // Metadata
    {
        let metadata_schema = blueprint_state_schema(
            MetadataNativePackage::definition(),
            METADATA_BLUEPRINT,
            indexmap!(),
        );
        // Additional validation has been added as part of this commit.
        // The logic is backward compatible, as it's used by protocol updates only.
        let metadata_system_struct =
            MetadataNativePackage::init_system_struct(validate_metadata_init(metadata).unwrap())
                .unwrap();
        let metadata_substates = SystemMapper::system_struct_to_node_substates(
            &metadata_schema,
            metadata_system_struct,
            METADATA_BASE_PARTITION,
        );
        node_substates.extend(metadata_substates);
    }

    {
        let role_assignment_schema = blueprint_state_schema(
            RoleAssignmentNativePackage::definition(),
            ROLE_ASSIGNMENT_BLUEPRINT,
            indexmap!(),
        );
        let role_assignment_system_struct =
            RoleAssignmentNativePackage::init_system_struct(OwnerRole::None.into(), indexmap!())
                .unwrap();
        let role_assignment_substates = SystemMapper::system_struct_to_node_substates(
            &role_assignment_schema,
            role_assignment_system_struct,
            ROLE_ASSIGNMENT_BASE_PARTITION,
        );
        node_substates.extend(role_assignment_substates);
    }

    //-------------------
    // SYSTEM PARTITIONS:
    //-------------------
    {
        node_substates.insert(
            TYPE_INFO_FIELD_PARTITION,
            type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
                blueprint_info: BlueprintInfo {
                    blueprint_id: BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
                    blueprint_version: BlueprintVersion::default(),
                    outer_obj_info: OuterObjectInfo::default(),
                    features: own_features.feature_names_string_set(),
                    generic_substitutions: vec![],
                },
                object_type: ObjectType::Global {
                    modules: indexmap!(
                        AttachedModuleId::Metadata => BlueprintVersion::default(),
                        AttachedModuleId::RoleAssignment => BlueprintVersion::default(),
                    ),
                },
            })),
        );
    }

    node_substates
}

fn globalize_package<Y: SystemApi<RuntimeError>>(
    package_address_reservation: Option<GlobalAddressReservation>,
    package_structure: PackageStructure,
    metadata: Own,
    role_assignment: RoleAssignment,
    api: &mut Y,
) -> Result<PackageAddress, RuntimeError> {
    let vault = Vault(ResourceManager(XRD).new_empty_vault(api)?);

    let (fields, kv_entries) =
        PackageNativePackage::init_system_struct(Some(vault), package_structure);

    let package_object = api.new_object(
        PACKAGE_BLUEPRINT,
        vec![PackageFeature::PackageRoyalty.feature_name()],
        GenericArgs::default(),
        fields,
        kv_entries,
    )?;

    let address = api.globalize(
        package_object,
        indexmap!(
            AttachedModuleId::Metadata => metadata.0,
            AttachedModuleId::RoleAssignment => role_assignment.0.0,
        ),
        package_address_reservation,
    )?;

    Ok(PackageAddress::new_or_panic(address.into_node_id().0))
}

pub struct PackageStructure {
    pub definitions: IndexMap<String, PackageBlueprintVersionDefinitionEntryPayload>,
    pub dependencies: IndexMap<String, PackageBlueprintVersionDependenciesEntryPayload>,
    pub schemas: IndexMap<SchemaHash, PackageSchemaEntryPayload>,
    pub vm_type: IndexMap<CodeHash, PackageCodeVmTypeEntryPayload>,
    pub original_code: IndexMap<CodeHash, PackageCodeOriginalCodeEntryPayload>,
    pub instrumented_code: IndexMap<CodeHash, PackageCodeInstrumentedCodeEntryPayload>,
    pub auth_configs: IndexMap<String, PackageBlueprintVersionAuthConfigEntryPayload>,
    pub package_royalties: IndexMap<String, PackageBlueprintVersionRoyaltyConfigEntryPayload>,
}

pub struct PackageNativePackage;

impl PackageNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let state = PackageStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = index_map_new();
        functions.insert(
            PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackagePublishWasmInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackagePublishWasmOutput>(),
                ),
                export: PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            },
        );
        functions.insert(
            PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackagePublishWasmAdvancedInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackagePublishWasmAdvancedOutput>(),
                ),
                export: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            },
        );
        functions.insert(
            PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackagePublishNativeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackagePublishNativeOutput>(),
                ),
                export: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            },
        );
        functions.insert(
            PACKAGE_CLAIM_ROYALTIES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackageClaimRoyaltiesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<PackageClaimRoyaltiesOutput>(),
                ),
                export: PACKAGE_CLAIM_ROYALTIES_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = indexmap!(
            PACKAGE_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                is_transient: false,
                feature_set: PackageFeatureSet::all_features(),
                dependencies: indexset!(
                    PACKAGE_OF_DIRECT_CALLER_RESOURCE.into(),
                    PACKAGE_OWNER_BADGE.into(),
                ),
                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state,
                    events: BlueprintEventSchemaInit::default(),
                    types: BlueprintTypeSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                    },
                    hooks: BlueprintHooksInit::default(),
                },
                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AccessRules(
                        indexmap!(
                            PACKAGE_PUBLISH_WASM_IDENT.to_string() => rule!(require(package_of_direct_caller(TRANSACTION_PROCESSOR_PACKAGE))),
                            PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string() => rule!(require(package_of_direct_caller(TRANSACTION_PROCESSOR_PACKAGE))),
                            PACKAGE_PUBLISH_NATIVE_IDENT.to_string() => rule!(require(system_execution(SystemExecution::Protocol))),
                        )
                    ),
                    method_auth: MethodAuthTemplate::StaticRoleDefinition(
                        roles_template! {
                            roles {
                                SECURIFY_OWNER_ROLE;
                            },
                            methods {
                                PACKAGE_CLAIM_ROYALTIES_IDENT => [SECURIFY_OWNER_ROLE];
                            }
                        },
                    ),
                },
            }
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<Y: SystemApi<RuntimeError>, V: VmApi>(
        export_name: &str,
        input: &IndexedScryptoValue,
        version: PackageV1MinorVersion,
        api: &mut Y,
        vm_api: &V,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let restrict_reserved_key = match version {
            PackageV1MinorVersion::Zero => false,
            PackageV1MinorVersion::One => true,
        };

        match export_name {
            PACKAGE_PUBLISH_NATIVE_IDENT => {
                let input: PackagePublishNativeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::publish_native(
                    input.package_address,
                    input.native_package_code_id,
                    input.definition,
                    input.metadata,
                    restrict_reserved_key,
                    api,
                    vm_api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            PACKAGE_PUBLISH_WASM_IDENT => {
                let input: PackagePublishWasmInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::publish_wasm(
                    input.code,
                    input.definition,
                    input.metadata,
                    restrict_reserved_key,
                    api,
                    vm_api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            PACKAGE_PUBLISH_WASM_ADVANCED_IDENT => {
                let input: PackagePublishWasmAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::publish_wasm_advanced(
                    input.package_address,
                    input.code,
                    input.definition,
                    input.metadata,
                    input.owner_role,
                    restrict_reserved_key,
                    api,
                    vm_api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            PACKAGE_CLAIM_ROYALTIES_IDENT => {
                let _input: PackageClaimRoyaltiesInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = PackageRoyaltyNativeBlueprint::claim_royalties(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    pub(crate) fn init_system_struct(
        royalty_vault: Option<Vault>,
        package_structure: PackageStructure,
    ) -> (
        IndexMap<u8, FieldValue>,
        IndexMap<u8, IndexMap<Vec<u8>, KVEntry>>,
    ) {
        let mut fields = index_map_new();
        if let Some(vault) = royalty_vault {
            let royalty = PackageRoyaltyAccumulator {
                royalty_vault: vault,
            }
            .into_payload();
            fields.insert(0u8, FieldValue::immutable(&royalty));
        }

        let mut kv_entries: IndexMap<u8, IndexMap<Vec<u8>, KVEntry>> = index_map_new();
        {
            let mut definition_partition = index_map_new();
            for (blueprint, definition) in package_structure.definitions {
                let key = BlueprintVersionKey::new_default(blueprint);
                let entry = KVEntry {
                    value: Some(scrypto_encode(&definition).unwrap()),
                    locked: true,
                };
                definition_partition.insert(scrypto_encode(&key).unwrap(), entry);
            }
            kv_entries.insert(
                PackageCollection::BlueprintVersionDefinitionKeyValue.collection_index(),
                definition_partition,
            );
        }

        {
            let mut dependency_partition = index_map_new();
            for (blueprint, dependencies) in package_structure.dependencies {
                let key = BlueprintVersionKey::new_default(blueprint);
                let entry = KVEntry {
                    value: Some(scrypto_encode(&dependencies).unwrap()),
                    locked: true,
                };
                dependency_partition.insert(scrypto_encode(&key).unwrap(), entry);
            }
            kv_entries.insert(
                PackageCollection::BlueprintVersionDependenciesKeyValue.collection_index(),
                dependency_partition,
            );
        }

        {
            let mut package_royalties_partition = index_map_new();
            for (blueprint, package_royalty) in package_structure.package_royalties {
                let key = BlueprintVersionKey::new_default(blueprint);
                let entry = KVEntry {
                    value: Some(scrypto_encode(&package_royalty).unwrap()),
                    locked: true,
                };
                package_royalties_partition.insert(scrypto_encode(&key).unwrap(), entry);
            }
            kv_entries.insert(
                PackageCollection::BlueprintVersionRoyaltyConfigKeyValue.collection_index(),
                package_royalties_partition,
            );
        }

        {
            let mut auth_partition = index_map_new();
            for (blueprint, auth_config) in package_structure.auth_configs {
                let key = BlueprintVersionKey::new_default(blueprint);
                let entry = KVEntry {
                    value: Some(scrypto_encode(&auth_config).unwrap()),
                    locked: true,
                };
                auth_partition.insert(scrypto_encode(&key).unwrap(), entry);
            }
            kv_entries.insert(
                PackageCollection::BlueprintVersionAuthConfigKeyValue.collection_index(),
                auth_partition,
            );
        }

        {
            let mut vm_type_partition = index_map_new();
            for (hash, vm_type) in package_structure.vm_type {
                let entry = KVEntry {
                    value: Some(scrypto_encode(&vm_type).unwrap()),
                    locked: true,
                };
                vm_type_partition.insert(scrypto_encode(&hash).unwrap(), entry);
            }
            kv_entries.insert(
                PackageCollection::CodeVmTypeKeyValue.collection_index(),
                vm_type_partition,
            );
        }

        {
            let mut original_code_partition = index_map_new();
            for (hash, code_substate) in package_structure.original_code {
                let entry = KVEntry {
                    value: Some(scrypto_encode(&code_substate).unwrap()),
                    locked: true,
                };
                original_code_partition.insert(scrypto_encode(&hash).unwrap(), entry);
            }
            kv_entries.insert(
                PackageCollection::CodeOriginalCodeKeyValue.collection_index(),
                original_code_partition,
            );
        }

        {
            let mut instrumented_code_partition = index_map_new();
            for (hash, code_substate) in package_structure.instrumented_code {
                let entry = KVEntry {
                    value: Some(scrypto_encode(&code_substate).unwrap()),
                    locked: true,
                };
                instrumented_code_partition.insert(scrypto_encode(&hash).unwrap(), entry);
            }
            kv_entries.insert(
                PackageCollection::CodeInstrumentedCodeKeyValue.collection_index(),
                instrumented_code_partition,
            );
        }

        {
            let mut schemas_partition = index_map_new();
            for (hash, schema) in package_structure.schemas {
                let entry = KVEntry {
                    value: Some(scrypto_encode(&schema).unwrap()),
                    locked: true,
                };
                schemas_partition.insert(scrypto_encode(&hash).unwrap(), entry);
            }
            kv_entries.insert(
                PackageCollection::SchemaKeyValue.collection_index(),
                schemas_partition,
            );
        }

        (fields, kv_entries)
    }

    pub fn validate_and_build_package_structure<V: VmApi>(
        definition: PackageDefinition,
        vm_type: VmType,
        original_code: Vec<u8>,
        system_instructions: BTreeMap<String, Vec<SystemInstruction>>,
        restrict_reserved_key: bool,
        vm_api: &V,
    ) -> Result<PackageStructure, RuntimeError> {
        // Validate schema
        validate_package_schema(&definition.blueprints)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_event_schemas(definition.blueprints.values())
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_type_schemas(definition.blueprints.values())
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_auth(&definition, restrict_reserved_key)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_names(&definition)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;

        // Validate VM specific properties
        let instrumented_code =
            VmPackageValidation::validate(&definition, vm_type, &original_code, vm_api)?;

        // Build Package structure
        let mut definitions = index_map_new();
        let mut dependencies = index_map_new();
        let mut schemas = index_map_new();
        let mut package_royalties = index_map_new();
        let mut auth_configs = index_map_new();
        let mut vm_type_substates = index_map_new();
        let mut original_code_substates = index_map_new();
        let mut instrumented_code_substates = index_map_new();

        let code_hash = CodeHash::from_hash(hash(&original_code));
        vm_type_substates.insert(code_hash, PackageCodeVmType { vm_type }.into_payload());
        original_code_substates.insert(
            code_hash,
            PackageCodeOriginalCode {
                code: original_code,
            }
            .into_payload(),
        );
        if let Some(instrumented_code) = instrumented_code {
            instrumented_code_substates.insert(
                code_hash,
                PackageCodeInstrumentedCode { instrumented_code }.into_payload(),
            );
        };

        {
            for (blueprint, definition_init) in definition.blueprints {
                auth_configs.insert(
                    blueprint.clone(),
                    definition_init.auth_config.into_payload(),
                );

                let blueprint_schema = definition_init.schema.schema.clone();
                let schema_hash = blueprint_schema.generate_schema_hash();
                schemas.insert(schema_hash, blueprint_schema.into_payload());

                let mut functions = index_map_new();
                let mut function_exports = index_map_new();
                for (function, function_schema_init) in definition_init.schema.functions.functions {
                    let input = match function_schema_init.input {
                        TypeRef::Static(input_type_id) => input_type_id,
                        TypeRef::Generic(..) => {
                            return Err(RuntimeError::ApplicationError(
                                ApplicationError::PackageError(PackageError::WasmUnsupported(
                                    "Generics not supported".to_string(),
                                )),
                            ))
                        }
                    };
                    let output = match function_schema_init.output {
                        TypeRef::Static(output_type_id) => output_type_id,
                        TypeRef::Generic(..) => {
                            return Err(RuntimeError::ApplicationError(
                                ApplicationError::PackageError(PackageError::WasmUnsupported(
                                    "Generics not supported".to_string(),
                                )),
                            ))
                        }
                    };
                    functions.insert(
                        function.clone(),
                        FunctionSchema {
                            receiver: function_schema_init.receiver,
                            input: BlueprintPayloadDef::Static(ScopedTypeId(schema_hash, input)),
                            output: BlueprintPayloadDef::Static(ScopedTypeId(schema_hash, output)),
                        },
                    );
                    let export = PackageExport {
                        code_hash,
                        export_name: function_schema_init.export.clone(),
                    };
                    function_exports.insert(function, export);
                }

                let mut events = index_map_new();
                for (key, type_ref) in definition_init.schema.events.event_schema {
                    events.insert(
                        key,
                        BlueprintPayloadDef::from_type_ref(type_ref, schema_hash),
                    );
                }

                let mut types = index_map_new();
                for (key, local_type_id) in definition_init.schema.types.type_schema {
                    types.insert(key, ScopedTypeId(schema_hash, local_type_id));
                }

                let system_instructions = system_instructions
                    .get(&blueprint)
                    .cloned()
                    .unwrap_or_default();

                let mut system_mappings = index_map_new();
                for system_instruction in system_instructions {
                    match system_instruction {
                        SystemInstruction::MapCollectionToPhysicalPartition {
                            collection_index,
                            partition_num,
                        } => {
                            system_mappings.insert(collection_index as usize, partition_num);
                        }
                    }
                }

                let definition = BlueprintDefinition {
                    interface: BlueprintInterface {
                        blueprint_type: definition_init.blueprint_type,
                        is_transient: definition_init.is_transient,
                        generics: definition_init.schema.generics,
                        feature_set: definition_init.feature_set,
                        functions,
                        events,
                        types,
                        state: IndexedStateSchema::from_schema(
                            schema_hash,
                            definition_init.schema.state,
                            system_mappings,
                        ),
                    },
                    function_exports,
                    hook_exports: {
                        definition_init
                            .schema
                            .hooks
                            .hooks
                            .into_iter()
                            .map(|(k, v)| {
                                (
                                    k,
                                    PackageExport {
                                        code_hash,
                                        export_name: v,
                                    },
                                )
                            })
                            .collect()
                    },
                };
                definitions.insert(blueprint.clone(), definition.into_payload());

                let minor_version_config = BlueprintDependencies {
                    dependencies: definition_init.dependencies,
                };
                dependencies.insert(blueprint.clone(), minor_version_config.into_payload());

                package_royalties.insert(
                    blueprint.clone(),
                    definition_init.royalty_config.into_payload(),
                );
            }
        };

        let package_structure = PackageStructure {
            definitions,
            dependencies,
            schemas,
            vm_type: vm_type_substates,
            original_code: original_code_substates,
            instrumented_code: instrumented_code_substates,
            auth_configs,
            package_royalties,
        };

        Ok(package_structure)
    }

    pub(crate) fn publish_native<Y: SystemApi<RuntimeError>, V: VmApi>(
        package_address: Option<GlobalAddressReservation>,
        native_package_code_id: u64,
        definition: PackageDefinition,
        metadata_init: MetadataInit,
        restrict_reserved_key: bool,
        api: &mut Y,
        vm_api: &V,
    ) -> Result<PackageAddress, RuntimeError> {
        validate_royalties(&definition, api)?;
        let package_structure = Self::validate_and_build_package_structure(
            definition,
            VmType::Native,
            native_package_code_id.to_be_bytes().to_vec(),
            Default::default(),
            restrict_reserved_key,
            vm_api,
        )?;
        let role_assignment = RoleAssignment::create(OwnerRole::None, indexmap!(), api)?;
        let metadata = Metadata::create_with_data(metadata_init, api)?;

        globalize_package(
            package_address,
            package_structure,
            metadata,
            role_assignment,
            api,
        )
    }

    pub(crate) fn publish_wasm<Y: SystemApi<RuntimeError>, V: VmApi>(
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata_init: MetadataInit,
        restrict_reserved_key: bool,
        api: &mut Y,
        vm_api: &V,
    ) -> Result<(PackageAddress, Bucket), RuntimeError> {
        validate_royalties(&definition, api)?;

        let package_structure = Self::validate_and_build_package_structure(
            definition,
            VmType::ScryptoV1,
            code,
            Default::default(),
            restrict_reserved_key,
            vm_api,
        )?;

        let (address_reservation, address) = api.allocate_global_address(BlueprintId {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
        })?;

        let (role_assignment, bucket) = SecurifiedPackage::create_securified(
            PackageOwnerBadgeData {
                name: "Package Owner Badge".to_owned(),
                package: address.try_into().expect("Impossible Case"),
            },
            Some(NonFungibleLocalId::bytes(address.as_node_id().0).unwrap()),
            api,
        )?;
        let metadata = Metadata::create_with_data(metadata_init, api)?;

        let address = globalize_package(
            Some(address_reservation),
            package_structure,
            metadata,
            role_assignment,
            api,
        )?;

        Ok((address, bucket))
    }

    pub(crate) fn publish_wasm_advanced<Y: SystemApi<RuntimeError>, V: VmApi>(
        package_address: Option<GlobalAddressReservation>,
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata_init: MetadataInit,
        owner_role: OwnerRole,
        restrict_reserved_key: bool,
        api: &mut Y,
        vm_api: &V,
    ) -> Result<PackageAddress, RuntimeError> {
        validate_royalties(&definition, api)?;
        let package_structure = Self::validate_and_build_package_structure(
            definition,
            VmType::ScryptoV1,
            code,
            Default::default(),
            restrict_reserved_key,
            vm_api,
        )?;
        let metadata = Metadata::create_with_data(metadata_init, api)?;
        let role_assignment = SecurifiedPackage::create_advanced(owner_role, api)?;

        globalize_package(
            package_address,
            package_structure,
            metadata,
            role_assignment,
            api,
        )
    }
}

pub struct PackageRoyaltyNativeBlueprint;

impl PackageRoyaltyNativeBlueprint {
    pub fn charge_package_royalty<Y: SystemBasedKernelApi>(
        receiver: &NodeId,
        bp_version_key: &BlueprintVersionKey,
        ident: &str,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        {
            let mut service = SystemService::new(api);
            if !service.is_feature_enabled(
                receiver,
                None,
                PackageFeature::PackageRoyalty.feature_name(),
            )? {
                return Ok(());
            }
        }

        let handle = api.kernel_open_substate_with_default(
            receiver,
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_ROYALTY_PARTITION_OFFSET)
                .unwrap(),
            &SubstateKey::Map(scrypto_encode(&bp_version_key).unwrap()),
            LockFlags::read_only(),
            Some(|| {
                let kv_entry = KeyValueEntrySubstate::<()>::default();
                IndexedScryptoValue::from_typed(&kv_entry)
            }),
            SystemLockData::default(),
        )?;

        let substate: PackageBlueprintVersionRoyaltyConfigEntrySubstate =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        api.kernel_close_substate(handle)?;

        let royalty_charge = substate
            .into_value()
            .and_then(|royalty_config| {
                match royalty_config.fully_update_and_into_latest_version() {
                    PackageRoyaltyConfig::Enabled(royalty_amounts) => {
                        royalty_amounts.get(ident).cloned()
                    }
                    PackageRoyaltyConfig::Disabled => Some(RoyaltyAmount::Free),
                }
            })
            .unwrap_or(RoyaltyAmount::Free);

        // we check for negative royalties at the instantiation time of the royalty module.
        assert!(!royalty_charge.is_negative());

        if royalty_charge.is_non_zero() {
            let handle = api.kernel_open_substate(
                receiver,
                MAIN_BASE_PARTITION,
                &PackageField::RoyaltyAccumulator.into(),
                LockFlags::MUTABLE,
                SystemLockData::default(),
            )?;

            let substate: PackageRoyaltyAccumulatorFieldSubstate =
                api.kernel_read_substate(handle)?.as_typed().unwrap();

            let vault_id = substate
                .into_payload()
                .fully_update_and_into_latest_version()
                .royalty_vault
                .0;
            let package_address = PackageAddress::new_or_panic(receiver.0);
            apply_royalty_cost(
                &mut api.system_module_api(),
                royalty_charge,
                RoyaltyRecipient::Package(package_address, vault_id.0),
            )?;

            api.kernel_close_substate(handle)?;
        }

        Ok(())
    }

    pub(crate) fn claim_royalties<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError> {
        if !api.actor_is_feature_enabled(
            ACTOR_STATE_SELF,
            PackageFeature::PackageRoyalty.feature_name(),
        )? {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::PackageError(PackageError::RoyaltiesNotEnabled),
            ));
        }

        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            PackageField::RoyaltyAccumulator.into(),
            LockFlags::read_only(),
        )?;

        let substate: PackageRoyaltyAccumulatorFieldPayload = api.field_read_typed(handle)?;
        let bucket = substate
            .fully_update_and_into_latest_version()
            .royalty_vault
            .take_all(api)?;

        Ok(bucket)
    }
}

pub struct PackageAuthNativeBlueprint;

impl PackageAuthNativeBlueprint {
    pub fn resolve_function_permission(
        receiver: &NodeId,
        bp_version_key: &BlueprintVersionKey,
        ident: &str,
        api: &mut impl SystemBasedKernelApi,
    ) -> Result<ResolvedPermission, RuntimeError> {
        let auth_config = Self::get_bp_auth_template(receiver, bp_version_key, api)?;
        match auth_config.function_auth {
            FunctionAuth::AllowAll => Ok(ResolvedPermission::AllowAll),
            FunctionAuth::RootOnly => {
                let is_root = api.kernel_get_system_state().current_call_frame.is_root();
                if is_root {
                    Ok(ResolvedPermission::AllowAll)
                } else {
                    Ok(ResolvedPermission::AccessRule(AccessRule::DenyAll))
                }
            }
            FunctionAuth::AccessRules(rules) => {
                let access_rule = rules.get(ident);
                if let Some(access_rule) = access_rule {
                    Ok(ResolvedPermission::AccessRule(access_rule.clone()))
                } else {
                    let package_address = PackageAddress::new_or_panic(receiver.0.clone());
                    let blueprint_id =
                        BlueprintId::new(&package_address, &bp_version_key.blueprint);
                    Err(RuntimeError::SystemModuleError(
                        SystemModuleError::AuthError(AuthError::NoFunction(FnIdentifier {
                            blueprint_id,
                            ident: ident.to_string(),
                        })),
                    ))
                }
            }
        }
    }

    pub fn get_bp_auth_template(
        receiver: &NodeId,
        bp_version_key: &BlueprintVersionKey,
        api: &mut impl SystemBasedKernelApi,
    ) -> Result<AuthConfig, RuntimeError> {
        let package_bp_version_id = CanonicalBlueprintId {
            address: PackageAddress::new_or_panic(receiver.0.clone()),
            blueprint: bp_version_key.blueprint.to_string(),
            version: bp_version_key.version.clone(),
        };

        let auth_template = api
            .kernel_get_system_state()
            .system
            .auth_cache
            .get(&package_bp_version_id);
        if let Some(auth_template) = auth_template {
            return Ok(auth_template.clone());
        }

        let handle = api.kernel_open_substate_with_default(
            receiver,
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_AUTH_TEMPLATE_PARTITION_OFFSET)
                .unwrap(),
            &SubstateKey::Map(scrypto_encode(&bp_version_key).unwrap()),
            LockFlags::read_only(),
            Some(|| {
                let kv_entry = KeyValueEntrySubstate::<()>::default();
                IndexedScryptoValue::from_typed(&kv_entry)
            }),
            SystemLockData::default(),
        )?;

        let auth_template: PackageBlueprintVersionAuthConfigEntrySubstate =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        api.kernel_close_substate(handle)?;

        let template = match auth_template.into_value() {
            Some(template) => template.fully_update_and_into_latest_version(),
            None => {
                return Err(RuntimeError::SystemError(
                    SystemError::AuthTemplateDoesNotExist(package_bp_version_id),
                ))
            }
        };

        api.kernel_get_system_state()
            .system
            .auth_cache
            .insert(package_bp_version_id, template.clone());

        Ok(template)
    }
}

#[derive(ScryptoSbor)]
pub struct PackageOwnerBadgeData {
    pub name: String,
    pub package: PackageAddress,
}

impl NonFungibleData for PackageOwnerBadgeData {
    const MUTABLE_FIELDS: &'static [&'static str] = &[];
}
