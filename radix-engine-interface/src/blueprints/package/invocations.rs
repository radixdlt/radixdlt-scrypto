use crate::blueprints::resource::*;
use crate::types::*;
use crate::*;
use radix_engine_common::data::manifest::model::ManifestAddressReservation;
use radix_engine_common::data::manifest::model::ManifestBlobRef;
use radix_engine_common::prelude::ScryptoSchema;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use sbor::basic_well_known_types::ANY_ID;
use sbor::basic_well_known_types::UNIT_ID;
use sbor::rust::prelude::*;
use sbor::LocalTypeIndex;
use scrypto_schema::BlueprintEventSchemaInit;
use scrypto_schema::BlueprintFunctionsSchemaInit;
use scrypto_schema::BlueprintSchemaInit;
use scrypto_schema::BlueprintStateSchemaInit;
use scrypto_schema::FieldSchema;
use scrypto_schema::FunctionSchemaInit;
use scrypto_schema::TypeRef;
use utils::btreemap;
use utils::btreeset;

pub const PACKAGE_BLUEPRINT: &str = "Package";

pub const PACKAGE_PUBLISH_WASM_IDENT: &str = "publish_wasm";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct PackagePublishWasmInput {
    pub code: Vec<u8>,
    pub setup: PackageDefinition,
    pub metadata: MetadataInit,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishWasmManifestInput {
    pub code: ManifestBlobRef,
    pub setup: PackageDefinition,
    pub metadata: MetadataInit,
}

pub type PackagePublishWasmOutput = (PackageAddress, Bucket);

pub const PACKAGE_PUBLISH_WASM_ADVANCED_IDENT: &str = "publish_wasm_advanced";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackagePublishWasmAdvancedInput {
    pub package_address: Option<GlobalAddressReservation>,
    pub code: Vec<u8>,
    pub setup: PackageDefinition,
    pub metadata: MetadataInit,
    pub owner_role: OwnerRole,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishWasmAdvancedManifestInput {
    pub package_address: Option<ManifestAddressReservation>,
    pub code: ManifestBlobRef,
    pub setup: PackageDefinition,
    pub metadata: MetadataInit,
    pub owner_role: OwnerRole,
}

pub type PackagePublishWasmAdvancedOutput = PackageAddress;

pub const PACKAGE_PUBLISH_NATIVE_IDENT: &str = "publish_native";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct PackagePublishNativeInput {
    pub package_address: Option<GlobalAddressReservation>,
    pub native_package_code_id: u64,
    pub setup: PackageDefinition,
    pub metadata: MetadataInit,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct PackagePublishNativeManifestInput {
    pub package_address: Option<ManifestAddressReservation>,
    pub native_package_code_id: u64,
    pub setup: PackageDefinition,
    pub metadata: MetadataInit,
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
    StaticRoles(StaticRoles),
}

impl Default for MethodAuthTemplate {
    fn default() -> Self {
        MethodAuthTemplate::StaticRoles(StaticRoles::default())
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
pub struct StaticRoles {
    pub roles: RoleSpecification,
    pub methods: BTreeMap<MethodKey, MethodAccessibility>,
}

impl Default for StaticRoles {
    fn default() -> Self {
        Self {
            methods: BTreeMap::new(),
            roles: RoleSpecification::Normal(BTreeMap::new()),
        }
    }
}

impl PackageDefinition {
    // For testing only
    pub fn single_test_function(blueprint_name: &str, function_name: &str) -> PackageDefinition {
        let mut blueprints = BTreeMap::new();
        blueprints.insert(
            blueprint_name.to_string(),
            BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                feature_set: btreeset!(),
                dependencies: btreeset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema: ScryptoSchema {
                        type_kinds: vec![],
                        type_metadata: vec![],
                        type_validations: vec![],
                    },
                    state: BlueprintStateSchemaInit {
                        fields: vec![FieldSchema::static_field(LocalTypeIndex::WellKnown(
                            UNIT_ID,
                        ))],
                        collections: vec![],
                    },
                    events: BlueprintEventSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        virtual_lazy_load_functions: btreemap!(),
                        functions: btreemap!(
                        function_name.to_string() => FunctionSchemaInit {
                                receiver: Option::None,
                                input: TypeRef::Static(LocalTypeIndex::WellKnown(ANY_ID)),
                                output: TypeRef::Static(LocalTypeIndex::WellKnown(ANY_ID)),
                                export: format!("{}_{}", blueprint_name, function_name),
                            }
                        ),
                    },
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::AllowAll,
                },
            },
        );
        PackageDefinition { blueprints }
    }
}
