use crate::blueprints::resource::*;
use crate::types::*;
use crate::*;
use radix_engine_common::data::manifest::model::ManifestAddressReservation;
use radix_engine_common::data::manifest::model::ManifestBlobRef;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use sbor::basic_well_known_types::ANY_TYPE;
use sbor::rust::prelude::*;
use sbor::LocalTypeIndex;
use scrypto_schema::TypeRef;
use scrypto_schema::{BlueprintCollectionSchema, BlueprintKeyValueSchema, FunctionSchemaInit};
use scrypto_schema::{BlueprintFunctionsSchemaInit, ReceiverInfo};
use scrypto_schema::{BlueprintSchemaInit, BlueprintStateSchemaInit, FieldSchema};

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

#[derive(
    Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestCategorize, ManifestEncode, ManifestDecode,
)]
pub struct PackageClaimRoyaltiesInput {}

pub type PackageClaimRoyaltiesOutput = Bucket;

#[derive(Debug, Clone, Eq, PartialEq, Default, ScryptoSbor, ManifestSbor)]
pub struct PackageDefinition {
    pub blueprints: BTreeMap<String, BlueprintDefinitionInit>,
}

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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintDefinitionInit {
    pub blueprint_type: BlueprintType,
    pub is_transient: bool,
    pub feature_set: BTreeSet<String>,
    pub dependencies: BTreeSet<GlobalAddress>,
    pub schema: BlueprintSchemaInit,
    pub royalty_config: PackageRoyaltyConfig,
    pub auth_config: AuthConfig,
}

impl Default for BlueprintDefinitionInit {
    fn default() -> Self {
        Self {
            blueprint_type: BlueprintType::default(),
            is_transient: false,
            feature_set: BTreeSet::default(),
            dependencies: BTreeSet::default(),
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
    AccessRules(BTreeMap<String, AccessRule>),
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
    Normal(BTreeMap<RoleKey, RoleList>),
    /// Roles are specified in the *outer* blueprint and defined in the instantiated *outer* object.
    /// This may only be used by inner blueprints and is currently used by the Vault blueprints
    UseOuter,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct StaticRoleDefinition {
    pub roles: RoleSpecification,
    pub methods: BTreeMap<MethodKey, MethodAccessibility>,
}

impl Default for StaticRoleDefinition {
    fn default() -> Self {
        Self {
            methods: BTreeMap::new(),
            roles: RoleSpecification::Normal(BTreeMap::new()),
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
        roles: BTreeMap<RoleKey, RoleList>,
    ) -> PackageDefinition {
        let mut blueprints = BTreeMap::new();
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
        let mut blueprints = BTreeMap::new();
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
                                    input: TypeRef::Static(LocalTypeIndex::WellKnown(ANY_TYPE)),
                                    output: TypeRef::Static(LocalTypeIndex::WellKnown(ANY_TYPE)),
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
    pub fn new_with_field_test_definition(
        blueprint_name: &str,
        functions: Vec<(&str, &str, bool)>,
    ) -> PackageDefinition {
        let mut blueprints = BTreeMap::new();
        blueprints.insert(
            blueprint_name.to_string(),
            BlueprintDefinitionInit {
                schema: BlueprintSchemaInit {
                    state: BlueprintStateSchemaInit {
                        fields: vec![FieldSchema::static_field(LocalTypeIndex::WellKnown(
                            ANY_TYPE,
                        ))],
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
                                    input: TypeRef::Static(LocalTypeIndex::WellKnown(ANY_TYPE)),
                                    output: TypeRef::Static(LocalTypeIndex::WellKnown(ANY_TYPE)),
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

    // For testing only
    pub fn new_with_kv_collection_test_definition(
        blueprint_name: &str,
        functions: Vec<(&str, &str, bool)>,
    ) -> PackageDefinition {
        let mut blueprints = BTreeMap::new();
        blueprints.insert(
            blueprint_name.to_string(),
            BlueprintDefinitionInit {
                schema: BlueprintSchemaInit {
                    state: BlueprintStateSchemaInit {
                        collections: vec![BlueprintCollectionSchema::KeyValueStore(
                            BlueprintKeyValueSchema {
                                key: TypeRef::Static(LocalTypeIndex::WellKnown(ANY_TYPE)),
                                value: TypeRef::Static(LocalTypeIndex::WellKnown(ANY_TYPE)),
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
                                    input: TypeRef::Static(LocalTypeIndex::WellKnown(ANY_TYPE)),
                                    output: TypeRef::Static(LocalTypeIndex::WellKnown(ANY_TYPE)),
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
