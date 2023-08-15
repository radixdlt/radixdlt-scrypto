use crate::blueprints::macros::*;
use crate::blueprints::util::SecurifiedRoleAssignment;
use crate::errors::*;
use crate::kernel::kernel_api::{KernelApi, KernelSubstateApi};
use crate::system::node_init::type_info_partition;
use crate::system::node_modules::metadata::MetadataEntrySubstate;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system_modules::costing::{apply_royalty_cost, RoyaltyRecipient};
use crate::track::interface::NodeSubstates;
use crate::types::*;
use crate::vm::wasm::PrepareError;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::role_assignment::RoleAssignment;
use native_sdk::resource::NativeVault;
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use radix_engine_interface::api::*;
pub use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::{require, Bucket};
use radix_engine_interface::schema::*;
use sbor::LocalTypeIndex;
use syn::Ident;

// Import and re-export substate types
use crate::roles_template;
use crate::system::node_modules::role_assignment::RoleAssignmentNativePackage;
use crate::system::node_modules::royalty::RoyaltyUtil;
use crate::system::system::*;
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::auth::{AuthError, ResolvedPermission};
use crate::vm::VmPackageValidation;

use super::*;

pub const PACKAGE_ROYALTY_FEATURE: &str = "package_royalty";

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PackageError {
    InvalidWasm(PrepareError),

    InvalidBlueprintSchema(SchemaValidationError),
    TooManySubstateSchemas,

    FailedToResolveLocalSchema {
        local_type_index: LocalTypeIndex,
    },
    EventNameMismatch {
        expected: String,
        actual: Option<String>,
    },
    InvalidEventSchema,
    InvalidSystemFunction,
    InvalidTypeParent,
    InvalidName(String),
    MissingOuterBlueprint,
    WasmUnsupported(String),
    InvalidLocalTypeIndex(LocalTypeIndex),
    InvalidGenericId(u8),
    EventGenericTypeNotSupported,

    InvalidAuthSetup,
    DefiningReservedRoleKey(String, RoleKey),
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
}

fn validate_package_schema<'a, I: Iterator<Item = &'a BlueprintSchemaInit>>(
    blueprints: I,
) -> Result<(), PackageError> {
    for bp_init in blueprints {
        validate_schema(&bp_init.schema).map_err(|e| PackageError::InvalidBlueprintSchema(e))?;

        if bp_init.state.fields.len() > 0xff {
            return Err(PackageError::TooManySubstateSchemas);
        }

        // FIXME: Also add validation for local_type_index in Instance and KVStore schema type references

        for field in &bp_init.state.fields {
            validate_package_schema_type_ref(bp_init, field.field)?;
        }

        for collection in &bp_init.state.collections {
            match collection {
                BlueprintCollectionSchema::KeyValueStore(kv_store_schema) => {
                    validate_package_schema_type_ref(bp_init, kv_store_schema.key)?;
                    validate_package_schema_type_ref(bp_init, kv_store_schema.value)?;
                }
                BlueprintCollectionSchema::SortedIndex(kv_store_schema) => {
                    validate_package_schema_type_ref(bp_init, kv_store_schema.key)?;
                    validate_package_schema_type_ref(bp_init, kv_store_schema.value)?;
                }
                BlueprintCollectionSchema::Index(kv_store_schema) => {
                    validate_package_schema_type_ref(bp_init, kv_store_schema.key)?;
                    validate_package_schema_type_ref(bp_init, kv_store_schema.value)?;
                }
            }
        }

        for (_name, event) in &bp_init.events.event_schema {
            validate_package_schema_type_ref(bp_init, *event)?;
        }

        for (_name, function) in &bp_init.functions.functions {
            validate_package_schema_type_ref(bp_init, function.input)?;
            validate_package_schema_type_ref(bp_init, function.output)?;
        }
    }

    Ok(())
}

fn validate_package_schema_type_ref(
    blueprint_schema_init: &BlueprintSchemaInit,
    type_ref: TypeRef<LocalTypeIndex>,
) -> Result<(), PackageError> {
    match type_ref {
        TypeRef::Static(local_type_index) => {
            if blueprint_schema_init
                .schema
                .resolve_type_kind(local_type_index)
                .is_some()
            {
                Ok(())
            } else {
                Err(PackageError::InvalidLocalTypeIndex(local_type_index))
            }
        }
        TypeRef::Generic(generic_id) => {
            if (generic_id as usize) < blueprint_schema_init.generics.len() {
                Ok(())
            } else {
                Err(PackageError::InvalidGenericId(generic_id))
            }
        }
    }
}

fn extract_package_event_static_type_index(
    blueprint_init: &BlueprintSchemaInit,
    type_ref: TypeRef<LocalTypeIndex>,
) -> Result<LocalTypeIndex, PackageError> {
    match type_ref {
        TypeRef::Static(local_type_index) => {
            if blueprint_init
                .schema
                .resolve_type_kind(local_type_index)
                .is_some()
            {
                Ok(local_type_index)
            } else {
                Err(PackageError::InvalidLocalTypeIndex(local_type_index))
            }
        }
        TypeRef::Generic(_) => Err(PackageError::EventGenericTypeNotSupported),
    }
}

fn validate_package_event_schema<'a, I: Iterator<Item = &'a BlueprintDefinitionInit>>(
    blueprints: I,
) -> Result<(), PackageError> {
    for blueprint_init in blueprints {
        let blueprint_schema_init = &blueprint_init.schema;
        let BlueprintSchemaInit { schema, events, .. } = blueprint_schema_init;

        for (expected_event_name, type_ref) in events.event_schema.iter() {
            let local_type_index =
                extract_package_event_static_type_index(blueprint_schema_init, *type_ref)?;

            // Checking that the event is either a struct or an enum
            let type_kind = schema.resolve_type_kind(local_type_index).map_or(
                Err(PackageError::FailedToResolveLocalSchema { local_type_index }),
                Ok,
            )?;
            match type_kind {
                // Structs and Enums are allowed
                TypeKind::Enum { .. } | TypeKind::Tuple { .. } => Ok(()),
                _ => Err(PackageError::InvalidEventSchema),
            }?;

            // Checking that the event name is indeed what the user claims it to be
            let actual_event_name = schema.resolve_type_metadata(local_type_index).map_or(
                Err(PackageError::FailedToResolveLocalSchema {
                    local_type_index: local_type_index,
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

fn validate_royalties<Y>(definition: &PackageDefinition, api: &mut Y) -> Result<(), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
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

fn validate_auth(definition: &PackageDefinition) -> Result<(), PackageError> {
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
            }
        }

        match (
            &definition_init.blueprint_type,
            &definition_init.auth_config.method_auth,
        ) {
            (_, MethodAuthTemplate::AllowAll) => {}
            (blueprint_type, MethodAuthTemplate::StaticRoles(StaticRoles { roles, methods })) => {
                let role_specification = match (blueprint_type, roles) {
                    (_, RoleSpecification::Normal(roles)) => roles,
                    (BlueprintType::Inner { outer_blueprint }, RoleSpecification::UseOuter) => {
                        if let Some(blueprint) = definition.blueprints.get(outer_blueprint) {
                            match &blueprint.auth_config.method_auth {
                                MethodAuthTemplate::StaticRoles(StaticRoles {
                                    roles: RoleSpecification::Normal(roles),
                                    ..
                                }) => roles,
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
                        if RoleAssignmentNativePackage::is_reserved_role_key(role_key) {
                            continue;
                        }
                        if !role_specification.contains_key(role_key) {
                            return Err(PackageError::MissingRole(role_key.clone()));
                        }
                    }
                    Ok(())
                };

                if let RoleSpecification::Normal(roles) = roles {
                    for (role_key, role_list) in roles {
                        check_list(role_list)?;
                        if RoleAssignmentNativePackage::is_reserved_role_key(role_key) {
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
    // All names should follow Rust Identifier specification
    let condition = |name| {
        syn::parse_str::<Ident>(name).map_err(|_| PackageError::InvalidName(name.to_string()))
    };

    for (bp_name, bp_init) in definition.blueprints.iter() {
        condition(bp_name)?;

        for (name, _) in bp_init.schema.events.event_schema.iter() {
            condition(name)?;
        }

        for (name, _) in bp_init.schema.functions.functions.iter() {
            condition(name)?;
        }

        for (_, name) in bp_init.schema.hooks.hooks.iter() {
            condition(name)?;
        }

        for name in bp_init.feature_set.iter() {
            condition(name)?;
        }

        if let PackageRoyaltyConfig::Enabled(list) = &bp_init.royalty_config {
            for (name, _) in list.iter() {
                condition(name)?;
            }
        }

        if let FunctionAuth::AccessRules(list) = &bp_init.auth_config.function_auth {
            for (name, _) in list.iter() {
                condition(name)?;
            }
        }

        if let MethodAuthTemplate::StaticRoles(static_roles) = &bp_init.auth_config.method_auth {
            if let RoleSpecification::Normal(list) = &static_roles.roles {
                for (role_key, _) in list.iter() {
                    condition(&role_key.key)?;
                }
            }
            for (key, accessibility) in static_roles.methods.iter() {
                condition(&key.ident)?;
                if let MethodAccessibility::RoleProtected(role_list) = accessibility {
                    for role_key in &role_list.list {
                        condition(&role_key.key)?;
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

pub fn create_bootstrap_package_partitions(
    package_state_init: PackageStateInit,
    metadata: MetadataInit,
) -> NodeSubstates {
    // No features necessary
    let own_features = PackageFeatureSet {
        package_royalty: false,
    };

    //-----------------
    // MAIN PARTITIONS:
    //-----------------

    let mut partitions = package_state_init
        .into_kernel_main_partitions(own_features.into())
        .expect("Expected that correct substates are present for given features");

    //-------------------
    // MODULE PARTITIONS:
    //-------------------
    {
        let mut metadata_partition = BTreeMap::new();
        for (key, value) in metadata.data {
            let mutability = if value.lock {
                SubstateMutability::Immutable
            } else {
                SubstateMutability::Mutable
            };
            let value = MetadataEntrySubstate {
                value: value.value,
                mutability,
            };

            metadata_partition.insert(
                SubstateKey::Map(scrypto_encode(&key).unwrap()),
                IndexedScryptoValue::from_typed(&value),
            );
        }
        partitions.insert(METADATA_BASE_PARTITION, metadata_partition);
    }

    //-------------------
    // SYSTEM PARTITIONS:
    //-------------------
    {
        partitions.insert(
            TYPE_INFO_FIELD_PARTITION,
            type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
                global: true,
                module_versions: btreemap!(
                    ObjectModuleId::Main => BlueprintVersion::default(),
                    ObjectModuleId::Metadata => BlueprintVersion::default(),
                    ObjectModuleId::RoleAssignment => BlueprintVersion::default(),
                ),

                blueprint_info: BlueprintInfo {
                    blueprint_id: BlueprintId::new(&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT),
                    outer_obj_info: OuterObjectInfo::default(),
                    features: own_features.feature_names_string_set(),
                    instance_schema: None,
                },
            })),
        );
    }

    partitions
}

fn globalize_package<Y>(
    package_address_reservation: Option<GlobalAddressReservation>,
    mut package_state_init: PackageStateInit,
    metadata: Own,
    role_assignment: RoleAssignment,
    api: &mut Y,
) -> Result<PackageAddress, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    package_state_init.royalty = Some(
        PackageRoyaltyAccumulator {
            royalty_vault: Vault(ResourceManager(XRD).new_empty_vault(api)?),
        }
        .into_locked_substate(),
    );
    let package_object = package_state_init.into_new_object(
        api,
        PackageFeatureSet {
            package_royalty: true,
        },
        None,
    )?;

    let address = api.globalize(
        btreemap!(
            ObjectModuleId::Main => package_object,
            ObjectModuleId::Metadata => metadata.0,
            ObjectModuleId::RoleAssignment => role_assignment.0.0,
        ),
        package_address_reservation,
    )?;

    Ok(PackageAddress::new_or_panic(address.into_node_id().0))
}

pub struct PackageNativePackage;

impl PackageNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let state = PackageStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = BTreeMap::new();
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
        let blueprints = btreemap!(
            PACKAGE_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                is_transient: false,
                feature_set: btreeset!(
                    PACKAGE_ROYALTY_FEATURE.to_string(),
                ),
                dependencies: btreeset!(
                    PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
                    PACKAGE_OWNER_BADGE.into(),
                ),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state,
                    events: BlueprintEventSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                    },
                    hooks: BlueprintHooksInit::default(),
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AccessRules(
                        btreemap!(
                            PACKAGE_PUBLISH_WASM_IDENT.to_string() => rule!(require(package_of_direct_caller(TRANSACTION_PROCESSOR_PACKAGE))),
                            PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string() => rule!(require(package_of_direct_caller(TRANSACTION_PROCESSOR_PACKAGE))),
                            PACKAGE_PUBLISH_NATIVE_IDENT.to_string() => rule!(require(AuthAddresses::system_role())),
                        )
                    ),
                    method_auth: MethodAuthTemplate::StaticRoles(
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

    pub fn invoke_export<Y>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
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
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            PACKAGE_PUBLISH_WASM_IDENT => {
                let input: PackagePublishWasmInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = Self::publish_wasm(input.code, input.definition, input.metadata, api)?;

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
                    api,
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

    pub fn validate_and_build_package_state_init(
        definition: PackageDefinition,
        vm_type: VmType,
        original_code: Vec<u8>,
    ) -> Result<PackageStateInit, RuntimeError> {
        // Validate schema
        validate_package_schema(definition.blueprints.values().map(|s| &s.schema))
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_package_event_schema(definition.blueprints.values())
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_auth(&definition)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;
        validate_names(&definition)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::PackageError(e)))?;

        // Validate VM specific properties
        let instrumented_code =
            VmPackageValidation::validate(&definition, vm_type, &original_code)?;

        // Build Package structure
        let mut blueprint_version_definitions = IndexMap::default();
        let mut blueprint_version_dependencies = IndexMap::default();
        let mut schemas = IndexMap::default();
        let mut blueprint_version_royalty_configs = IndexMap::default();
        let mut blueprint_version_auth_configs = IndexMap::default();
        let mut vm_type_substates = IndexMap::default();
        let mut original_code_substates = IndexMap::default();
        let mut instrumented_code_substates = IndexMap::default();

        let code_hash = CodeHash::from(hash(&original_code));
        vm_type_substates.insert(
            code_hash.into_key(),
            PackageCodeVmType { vm_type }.into_locked_substate(),
        );
        original_code_substates.insert(
            code_hash.into_key(),
            PackageCodeOriginalCode {
                code: original_code,
            }
            .into_locked_substate(),
        );
        if let Some(instrumented_code) = instrumented_code {
            instrumented_code_substates.insert(
                code_hash.into_key(),
                PackageCodeInstrumentedCode { instrumented_code }.into_locked_substate(),
            );
        };

        {
            for (blueprint_name, definition_init) in definition.blueprints {
                let blueprint_version_key = BlueprintVersionKey::new_default(blueprint_name);

                blueprint_version_auth_configs.insert(
                    blueprint_version_key.clone().into_key(),
                    definition_init.auth_config.into_locked_substate(),
                );

                let blueprint_schema = definition_init.schema.schema.clone();
                let schema_hash = blueprint_schema.generate_schema_hash();
                schemas.insert(
                    schema_hash.into_key(),
                    blueprint_schema.into_locked_substate(),
                );

                let mut functions = BTreeMap::new();
                let mut function_exports = BTreeMap::new();
                for (function, function_schema_init) in definition_init.schema.functions.functions {
                    let input = match function_schema_init.input {
                        TypeRef::Static(input_type_index) => input_type_index,
                        TypeRef::Generic(..) => {
                            return Err(RuntimeError::ApplicationError(
                                ApplicationError::PackageError(PackageError::WasmUnsupported(
                                    "Generics not supported".to_string(),
                                )),
                            ))
                        }
                    };
                    let output = match function_schema_init.output {
                        TypeRef::Static(output_type_index) => output_type_index,
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
                            input: TypePointer::Package(TypeIdentifier(schema_hash, input)),
                            output: TypePointer::Package(TypeIdentifier(schema_hash, output)),
                        },
                    );
                    let export = PackageExport {
                        code_hash,
                        export_name: function_schema_init.export.clone(),
                    };
                    function_exports.insert(function, export);
                }

                let mut events = BTreeMap::new();
                for (key, type_ref) in definition_init.schema.events.event_schema {
                    let index = match type_ref {
                        TypeRef::Static(index) => {
                            TypePointer::Package(TypeIdentifier(schema_hash, index))
                        }
                        TypeRef::Generic(index) => TypePointer::Instance(index),
                    };
                    events.insert(key, index);
                }

                let definition = BlueprintDefinition {
                    interface: BlueprintInterface {
                        blueprint_type: definition_init.blueprint_type,
                        is_transient: definition_init.is_transient,
                        generics: definition_init.schema.generics,
                        feature_set: definition_init.feature_set,
                        functions,
                        events,
                        state: IndexedStateSchema::from_schema(
                            schema_hash,
                            definition_init.schema.state,
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
                blueprint_version_definitions.insert(
                    blueprint_version_key.clone().into_key(),
                    definition.into_locked_substate(),
                );

                blueprint_version_dependencies.insert(
                    blueprint_version_key.clone().into_key(),
                    BlueprintDependencies {
                        dependencies: definition_init.dependencies,
                    }
                    .into_locked_substate(),
                );

                blueprint_version_royalty_configs.insert(
                    blueprint_version_key.into_key(),
                    definition_init.royalty_config.into_locked_substate(),
                );
            }
        };

        Ok(PackageStateInit {
            royalty: None, // This is added later, if the feature is turned on
            blueprint_version_definitions,
            blueprint_version_dependencies,
            schemas,
            blueprint_version_royalty_configs,
            blueprint_version_auth_configs,
            code_vm_type: vm_type_substates,
            code_original_code: original_code_substates,
            code_instrumented_code: instrumented_code_substates,
        })
    }

    pub(crate) fn publish_native<Y>(
        package_address: Option<GlobalAddressReservation>,
        native_package_code_id: u64,
        definition: PackageDefinition,
        metadata_init: MetadataInit,
        api: &mut Y,
    ) -> Result<PackageAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        validate_royalties(&definition, api)?;
        let package_state_init = Self::validate_and_build_package_state_init(
            definition,
            VmType::Native,
            native_package_code_id.to_be_bytes().to_vec(),
        )?;
        let role_assignment = RoleAssignment::create(OwnerRole::None, btreemap!(), api)?;
        let metadata = Metadata::create_with_data(metadata_init, api)?;

        globalize_package(
            package_address,
            package_state_init,
            metadata,
            role_assignment,
            api,
        )
    }

    pub(crate) fn publish_wasm<Y>(
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata_init: MetadataInit,
        api: &mut Y,
    ) -> Result<(PackageAddress, Bucket), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        validate_royalties(&definition, api)?;
        let package_structure =
            Self::validate_and_build_package_state_init(definition, VmType::ScryptoV1, code)?;

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

    pub(crate) fn publish_wasm_advanced<Y>(
        package_address: Option<GlobalAddressReservation>,
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata_init: MetadataInit,
        owner_role: OwnerRole,
        api: &mut Y,
    ) -> Result<PackageAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        validate_royalties(&definition, api)?;
        let package_structure =
            Self::validate_and_build_package_state_init(definition, VmType::ScryptoV1, code)?;
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
    pub fn charge_package_royalty<Y, V>(
        receiver: &NodeId,
        bp_version_key: &BlueprintVersionKey,
        ident: &str,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        V: SystemCallbackObject,
        Y: KernelApi<SystemConfig<V>>,
    {
        {
            let mut service = SystemService::new(api);
            if !service.is_feature_enabled(
                receiver,
                ObjectModuleId::Main,
                PACKAGE_ROYALTY_FEATURE,
            )? {
                return Ok(());
            }
        }

        let handle = api.kernel_open_substate_with_default(
            receiver,
            PackagePartition::BlueprintVersionRoyaltyConfigKeyValue.as_main_partition(),
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
            .value
            .and_then(|royalty_config| match royalty_config.0.into_latest() {
                PackageRoyaltyConfig::Enabled(royalty_amounts) => {
                    royalty_amounts.get(ident).cloned()
                }
                PackageRoyaltyConfig::Disabled => Some(RoyaltyAmount::Free),
            })
            .unwrap_or(RoyaltyAmount::Free);

        if royalty_charge.is_non_zero() {
            let handle = api.kernel_open_substate(
                receiver,
                PackagePartition::Field.as_main_partition(),
                &PackageField::RoyaltyAccumulator.into(),
                LockFlags::MUTABLE,
                SystemLockData::default(),
            )?;

            let substate: PackageRoyaltyAccumulatorFieldSubstate =
                api.kernel_read_substate(handle)?.as_typed().unwrap();

            let vault_id = substate.value.0 .0.into_latest().royalty_vault.0;
            let package_address = PackageAddress::new_or_panic(receiver.0);
            apply_royalty_cost(
                api,
                royalty_charge,
                RoyaltyRecipient::Package(package_address),
                vault_id.0,
            )?;

            api.kernel_close_substate(handle)?;
        }

        Ok(())
    }

    pub(crate) fn claim_royalties<Y>(api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if !api.actor_is_feature_enabled(OBJECT_HANDLE_SELF, PACKAGE_ROYALTY_FEATURE)? {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::PackageError(PackageError::RoyaltiesNotEnabled),
            ));
        }

        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            PackageField::RoyaltyAccumulator.into(),
            LockFlags::read_only(),
        )?;

        let substate: PackageRoyaltyAccumulatorFieldPayload = api.field_read_typed(handle)?;
        let bucket = substate.0.into_latest().royalty_vault.take_all(api)?;

        Ok(bucket)
    }
}

pub struct PackageAuthNativeBlueprint;

impl PackageAuthNativeBlueprint {
    pub fn resolve_function_permission<Y, V>(
        receiver: &NodeId,
        bp_version_key: &BlueprintVersionKey,
        ident: &str,
        api: &mut Y,
    ) -> Result<ResolvedPermission, RuntimeError>
    where
        Y: KernelSubstateApi<SystemLockData> + KernelApi<SystemConfig<V>>,
        V: SystemCallbackObject,
    {
        let auth_config = Self::get_bp_auth_template(receiver, bp_version_key, api)?;
        match auth_config.function_auth {
            FunctionAuth::AllowAll => Ok(ResolvedPermission::AllowAll),
            FunctionAuth::RootOnly => {
                if api.kernel_get_current_depth() == 0 {
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

    pub fn get_bp_auth_template<Y, V>(
        receiver: &NodeId,
        bp_version_key: &BlueprintVersionKey,
        api: &mut Y,
    ) -> Result<AuthConfig, RuntimeError>
    where
        Y: KernelSubstateApi<SystemLockData> + KernelApi<SystemConfig<V>>,
        V: SystemCallbackObject,
    {
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
            PackagePartition::BlueprintVersionAuthConfigKeyValue.as_main_partition(),
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

        let template = match auth_template.value {
            Some(template) => template.0.into_latest(),
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
