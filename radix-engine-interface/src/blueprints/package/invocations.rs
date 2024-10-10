use crate::blueprints::resource::*;
use crate::internal_prelude::*;
use crate::types::*;
use radix_blueprint_schema_init::TypeRef;
use radix_blueprint_schema_init::{
    BlueprintCollectionSchema, BlueprintKeyValueSchema, FunctionSchemaInit,
};
use radix_blueprint_schema_init::{BlueprintFunctionsSchemaInit, ReceiverInfo};
use radix_blueprint_schema_init::{BlueprintSchemaInit, BlueprintStateSchemaInit, FieldSchema};
use radix_common::data::manifest::model::ManifestAddressReservation;
use radix_common::data::manifest::model::ManifestBlobRef;
use radix_engine_interface::object_modules::metadata::MetadataInit;
use sbor::basic_well_known_types::ANY_TYPE;
use sbor::rust::prelude::*;
use sbor::LocalTypeId;

pub const PACKAGE_BLUEPRINT: &str = "Package";

pub const PACKAGE_PUBLISH_WASM_IDENT: &str = "publish_wasm";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct PackagePublishWasmInput {
    pub definition: PackageDefinition,
    pub code: Vec<u8>,
    pub metadata: MetadataInit,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishWasmManifestInput {
    pub definition: PackageDefinition,
    pub code: ManifestBlobRef,
    pub metadata: MetadataInit,
}

pub type PackagePublishWasmOutput = (PackageAddress, Bucket);

pub const PACKAGE_PUBLISH_WASM_ADVANCED_IDENT: &str = "publish_wasm_advanced";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackagePublishWasmAdvancedInput {
    pub owner_role: OwnerRole,
    pub definition: PackageDefinition,
    pub code: Vec<u8>,
    pub metadata: MetadataInit,
    pub package_address: Option<GlobalAddressReservation>,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishWasmAdvancedManifestInput {
    pub owner_role: OwnerRole,
    pub definition: PackageDefinition,
    pub code: ManifestBlobRef,
    pub metadata: MetadataInit,
    pub package_address: Option<ManifestAddressReservation>,
}

pub type PackagePublishWasmAdvancedOutput = PackageAddress;

pub const PACKAGE_PUBLISH_NATIVE_IDENT: &str = "publish_native";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackagePublishNativeInput {
    pub definition: PackageDefinition,
    pub native_package_code_id: u64,
    pub metadata: MetadataInit,
    pub package_address: Option<GlobalAddressReservation>,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishNativeManifestInput {
    pub definition: PackageDefinition,
    pub native_package_code_id: u64,
    pub metadata: MetadataInit,
    pub package_address: Option<ManifestAddressReservation>,
}

pub type PackagePublishNativeOutput = PackageAddress;

pub const PACKAGE_CLAIM_ROYALTIES_IDENT: &str = "PackageRoyalty_claim_royalties";

#[cfg_attr(
    feature = "fuzzing",
    derive(arbitrary::Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct PackageClaimRoyaltiesInput {}

pub type PackageClaimRoyaltiesManifestInput = PackageClaimRoyaltiesInput;

pub type PackageClaimRoyaltiesOutput = Bucket;

/// The set of blueprints and their associated definitions for a package
#[derive(Debug, Clone, Eq, PartialEq, Default, ScryptoSbor, ManifestSbor)]
pub struct PackageDefinition {
    pub blueprints: IndexMap<String, BlueprintDefinitionInit>,
}

/// A blueprint may be specified as either an Outer or Inner Blueprint. If an inner blueprint, an associated outer
/// blueprint must be specified and only a component of the outer blueprint may instantiate the inner blueprint.
/// Inner blueprint components may access the state of its outer blueprint component directly.
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub enum BlueprintType {
    Outer,
    Inner { outer_blueprint: String },
}

impl Default for BlueprintType {
    fn default() -> Self {
        BlueprintType::Outer
    }
}

/// Structure which defines static interface qualities of a Blueprint
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintDefinitionInit {
    /// Whether the blueprint is an Outer or Inner Blueprint
    pub blueprint_type: BlueprintType,

    /// If true, all components of this blueprint type may not be persisted.
    pub is_transient: bool,

    /// The set of all possible features a component instantiator may specify.
    pub feature_set: IndexSet<String>,

    /// A set of addresses which will always be visible to call frames of this blueprint.
    pub dependencies: IndexSet<GlobalAddress>,

    /// The schema of the blueprint including interface, state, and events
    pub schema: BlueprintSchemaInit,

    /// Blueprint module: Royalty configuration
    pub royalty_config: PackageRoyaltyConfig,

    /// Blueprint module: Auth configuration such as role definitions
    pub auth_config: AuthConfig,
}

impl Default for BlueprintDefinitionInit {
    fn default() -> Self {
        Self {
            blueprint_type: BlueprintType::default(),
            is_transient: false,
            feature_set: IndexSet::default(),
            dependencies: IndexSet::default(),
            schema: BlueprintSchemaInit::default(),
            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default, ScryptoSbor, ManifestSbor)]
pub struct AuthConfig {
    pub function_auth: FunctionAuth,
    pub method_auth: MethodAuthTemplate,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub enum FunctionAuth {
    /// All functions are accessible
    AllowAll,
    /// Functions are protected by an access rule
    AccessRules(IndexMap<String, AccessRule>),
    /// Only the root call frame may call all functions.
    /// Used primarily for transaction processor functions, any other use would
    /// essentially make the function inaccessible for any normal transaction
    RootOnly,
}

impl Default for FunctionAuth {
    fn default() -> Self {
        FunctionAuth::AllowAll
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub enum MethodAuthTemplate {
    /// All methods are accessible
    AllowAll,
    /// Methods are protected by a static method to roles mapping
    StaticRoleDefinition(StaticRoleDefinition),
}

impl Default for MethodAuthTemplate {
    fn default() -> Self {
        MethodAuthTemplate::AllowAll
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub enum RoleSpecification {
    /// Roles are specified in the current blueprint and defined in the instantiated object.
    /// The map contains keys for all possible roles, mapping to a list of roles which may update
    /// the access rule for each role.
    Normal(IndexMap<RoleKey, RoleList>),
    /// Roles are specified in the *outer* blueprint and defined in the instantiated *outer* object.
    /// This may only be used by inner blueprints and is currently used by the Vault blueprints
    UseOuter,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct StaticRoleDefinition {
    pub roles: RoleSpecification,
    pub methods: IndexMap<MethodKey, MethodAccessibility>,
}

impl Default for StaticRoleDefinition {
    fn default() -> Self {
        Self {
            methods: index_map_new(),
            roles: RoleSpecification::Normal(index_map_new()),
        }
    }
}

impl PackageDefinition {
    // For testing only
    pub fn new_single_function_test_definition(
        blueprint_name: &str,
        function_name: &str,
    ) -> PackageDefinition {
        Self::new_functions_only_test_definition(
            blueprint_name,
            vec![(
                function_name,
                format!("{}_{}", blueprint_name, function_name).as_str(),
                false,
            )],
        )
    }

    // For testing only
    pub fn new_roles_only_test_definition(
        blueprint_name: &str,
        roles: IndexMap<RoleKey, RoleList>,
    ) -> PackageDefinition {
        let mut blueprints = index_map_new();
        blueprints.insert(
            blueprint_name.to_string(),
            BlueprintDefinitionInit {
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::StaticRoleDefinition(StaticRoleDefinition {
                        roles: RoleSpecification::Normal(roles),
                        ..Default::default()
                    }),
                },
                ..Default::default()
            },
        );
        PackageDefinition { blueprints }
    }

    // For testing only
    pub fn new_functions_only_test_definition(
        blueprint_name: &str,
        functions: Vec<(&str, &str, bool)>,
    ) -> PackageDefinition {
        let mut blueprints = index_map_new();
        blueprints.insert(
            blueprint_name.to_string(),
            BlueprintDefinitionInit {
                schema: BlueprintSchemaInit {
                    functions: BlueprintFunctionsSchemaInit {
                        functions: functions
                            .into_iter()
                            .map(|(function_name, export_name, has_receiver)| {
                                let schema = FunctionSchemaInit {
                                    receiver: if has_receiver {
                                        Some(ReceiverInfo::normal_ref())
                                    } else {
                                        None
                                    },
                                    input: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
                                    output: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
                                    export: export_name.to_string(),
                                };
                                (function_name.to_string(), schema)
                            })
                            .collect(),
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        PackageDefinition { blueprints }
    }

    // For testing only
    pub fn new_with_fields_test_definition(
        blueprint_name: &str,
        num_fields: usize,
        functions: Vec<(&str, &str, bool)>,
    ) -> PackageDefinition {
        let mut blueprints = index_map_new();
        blueprints.insert(
            blueprint_name.to_string(),
            BlueprintDefinitionInit {
                schema: BlueprintSchemaInit {
                    state: BlueprintStateSchemaInit {
                        fields: (0..num_fields)
                            .map(|_| FieldSchema::static_field(LocalTypeId::WellKnown(ANY_TYPE)))
                            .collect(),
                        ..Default::default()
                    },
                    functions: BlueprintFunctionsSchemaInit {
                        functions: functions
                            .into_iter()
                            .map(|(function_name, export_name, has_receiver)| {
                                let schema = FunctionSchemaInit {
                                    receiver: if has_receiver {
                                        Some(ReceiverInfo::normal_ref())
                                    } else {
                                        None
                                    },
                                    input: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
                                    output: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
                                    export: export_name.to_string(),
                                };
                                (function_name.to_string(), schema)
                            })
                            .collect(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        PackageDefinition { blueprints }
    }

    pub fn new_with_field_test_definition(
        blueprint_name: &str,
        functions: Vec<(&str, &str, bool)>,
    ) -> PackageDefinition {
        Self::new_with_fields_test_definition(blueprint_name, 1, functions)
    }

    // For testing only
    pub fn new_with_kv_collection_test_definition(
        blueprint_name: &str,
        functions: Vec<(&str, &str, bool)>,
    ) -> PackageDefinition {
        let mut blueprints = index_map_new();
        blueprints.insert(
            blueprint_name.to_string(),
            BlueprintDefinitionInit {
                schema: BlueprintSchemaInit {
                    state: BlueprintStateSchemaInit {
                        collections: vec![BlueprintCollectionSchema::KeyValueStore(
                            BlueprintKeyValueSchema {
                                key: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
                                value: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
                                allow_ownership: true,
                            },
                        )],
                        ..Default::default()
                    },
                    functions: BlueprintFunctionsSchemaInit {
                        functions: functions
                            .into_iter()
                            .map(|(function_name, export_name, has_receiver)| {
                                let schema = FunctionSchemaInit {
                                    receiver: if has_receiver {
                                        Some(ReceiverInfo::normal_ref())
                                    } else {
                                        None
                                    },
                                    input: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
                                    output: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
                                    export: export_name.to_string(),
                                };
                                (function_name.to_string(), schema)
                            })
                            .collect(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        PackageDefinition { blueprints }
    }
}
