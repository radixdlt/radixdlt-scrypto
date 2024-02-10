use radix_engine_common::prelude::*;
use radix_engine_common::{define_wrapped_hash, types::PartitionOffset};
use sbor::basic_well_known_types::ANY_TYPE;
use sbor::LocalTypeId;
use sbor::Sbor;
use blueprint_schema::BlueprintHook;
use blueprint_schema::GenericBound;
use blueprint_schema::TypeRef;
use blueprint_schema::{BlueprintCollectionSchema, BlueprintKeyValueSchema, FunctionSchemaInit};
use blueprint_schema::{BlueprintFunctionsSchemaInit, ReceiverInfo};
use blueprint_schema::{BlueprintSchemaInit, BlueprintStateSchemaInit, FieldSchema};

define_wrapped_hash!(
    /// Represents a particular instance of code under a package
    CodeHash
);

#[derive(Copy, Debug, Clone, PartialEq, Eq, Sbor)]
pub enum VmType {
    Native,
    ScryptoV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum BlueprintPayloadDef {
    Static(ScopedTypeId), // Fully Resolved type is defined in package
    Generic(u8),          // Fully Resolved type is mapped directly to a generic defined by instance
                          // TODO: How to represent a structure containing a generic?
}

impl BlueprintPayloadDef {
    pub fn from_type_ref(type_ref: TypeRef<LocalTypeId>, schema_hash: SchemaHash) -> Self {
        match type_ref {
            TypeRef::Static(type_id) => {
                BlueprintPayloadDef::Static(ScopedTypeId(schema_hash, type_id))
            }
            TypeRef::Generic(index) => BlueprintPayloadDef::Generic(index),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct FunctionSchema {
    pub receiver: Option<ReceiverInfo>,
    pub input: BlueprintPayloadDef,
    pub output: BlueprintPayloadDef,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Ord, PartialOrd, Hash)]
pub struct CanonicalBlueprintId {
    pub address: PackageAddress,
    pub blueprint: String,
    pub version: BlueprintVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Ord, PartialOrd, Hash)]
pub struct BlueprintVersionKey {
    pub blueprint: String,
    pub version: BlueprintVersion,
}

impl BlueprintVersionKey {
    pub fn new_default<S: ToString>(blueprint: S) -> Self {
        Self {
            blueprint: blueprint.to_string(),
            version: BlueprintVersion::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct BlueprintDependencies {
    pub dependencies: IndexSet<GlobalAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct PackageExport {
    pub code_hash: CodeHash,
    pub export_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct BlueprintDefinition {
    // Frontend interface, this must be backward compatible with minor version updates
    pub interface: BlueprintInterface,

    // Backend implementation pointers

    // There is an implicit invariant that must be maintained in that the key set in `function_exports`
    // matches that of the `functions` under `interface`. This is currently maintained since the
    // `publish` interface uses `BlueprintDefinitionInit` rather than `BlueprintDefinition`.
    pub function_exports: IndexMap<String, PackageExport>,
    pub hook_exports: IndexMap<BlueprintHook, PackageExport>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum KeyOrValue {
    Key,
    Value,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum InputOrOutput {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum BlueprintPayloadIdentifier {
    Function(String, InputOrOutput),
    Event(String),
    Field(u8),
    KeyValueEntry(u8, KeyOrValue),
    IndexEntry(u8, KeyOrValue),
    SortedIndexEntry(u8, KeyOrValue),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum BlueprintPartitionType {
    KeyValueCollection,
    IndexCollection,
    SortedIndexCollection,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintInterface {
    pub blueprint_type: BlueprintType,
    pub is_transient: bool,
    pub generics: Vec<GenericBound>,
    pub feature_set: IndexSet<String>,
    pub state: IndexedStateSchema,
    pub functions: IndexMap<String, FunctionSchema>,
    pub events: IndexMap<String, BlueprintPayloadDef>,
    pub types: IndexMap<String, ScopedTypeId>,
}

impl BlueprintInterface {
    pub fn get_field_payload_def(&self, field_index: u8) -> Option<BlueprintPayloadDef> {
        self.state.get_field_payload_def(field_index)
    }

    pub fn get_kv_key_payload_def(&self, collection_index: u8) -> Option<BlueprintPayloadDef> {
        self.state.get_kv_key_payload_def(collection_index)
    }

    pub fn find_function(&self, ident: &str) -> Option<FunctionSchema> {
        if let Some(x) = self.functions.get(ident) {
            if x.receiver.is_none() {
                return Some(x.clone());
            }
        }
        None
    }

    pub fn find_method(&self, ident: &str) -> Option<FunctionSchema> {
        if let Some(x) = self.functions.get(ident) {
            if x.receiver.is_some() {
                return Some(x.clone());
            }
        }
        None
    }

    pub fn get_function_input_payload_def(&self, ident: &str) -> Option<BlueprintPayloadDef> {
        let schema = self.functions.get(ident)?;
        Some(schema.input.clone())
    }

    pub fn get_function_output_payload_def(&self, ident: &str) -> Option<BlueprintPayloadDef> {
        let schema = self.functions.get(ident)?;
        Some(schema.output.clone())
    }

    pub fn get_event_payload_def(&self, event_name: &str) -> Option<BlueprintPayloadDef> {
        self.events.get(event_name).cloned()
    }

    pub fn get_payload_def(
        &self,
        payload_identifier: &BlueprintPayloadIdentifier,
    ) -> Option<(BlueprintPayloadDef, bool, bool)> {
        match payload_identifier {
            BlueprintPayloadIdentifier::Function(function_ident, InputOrOutput::Input) => {
                let payload_def = self.get_function_input_payload_def(function_ident.as_str())?;
                Some((payload_def, true, true))
            }
            BlueprintPayloadIdentifier::Function(function_ident, InputOrOutput::Output) => {
                let payload_def = self.get_function_output_payload_def(function_ident.as_str())?;
                Some((payload_def, true, true))
            }
            BlueprintPayloadIdentifier::Field(field_index) => {
                let payload_def = self.get_field_payload_def(*field_index)?;
                Some((payload_def, true, self.is_transient))
            }
            BlueprintPayloadIdentifier::KeyValueEntry(collection_index, KeyOrValue::Key) => {
                let payload_def = self.get_kv_key_payload_def(*collection_index)?;
                Some((payload_def, false, self.is_transient))
            }
            BlueprintPayloadIdentifier::KeyValueEntry(collection_index, KeyOrValue::Value) => {
                let (payload_def, allow_ownership) =
                    self.state.get_kv_value_payload_def(*collection_index)?;
                Some((payload_def, allow_ownership, self.is_transient))
            }
            BlueprintPayloadIdentifier::IndexEntry(collection_index, KeyOrValue::Key) => {
                let type_pointer = self.state.get_index_payload_def_key(*collection_index)?;
                Some((type_pointer, false, self.is_transient))
            }
            BlueprintPayloadIdentifier::IndexEntry(collection_index, KeyOrValue::Value) => {
                let type_pointer = self.state.get_index_payload_def_value(*collection_index)?;
                Some((type_pointer, false, self.is_transient))
            }
            BlueprintPayloadIdentifier::SortedIndexEntry(collection_index, KeyOrValue::Key) => {
                let type_pointer = self
                    .state
                    .get_sorted_index_payload_def_key(*collection_index)?;
                Some((type_pointer, false, self.is_transient))
            }
            BlueprintPayloadIdentifier::SortedIndexEntry(collection_index, KeyOrValue::Value) => {
                let type_pointer = self
                    .state
                    .get_sorted_index_payload_def_value(*collection_index)?;
                Some((type_pointer, false, self.is_transient))
            }
            BlueprintPayloadIdentifier::Event(event_name) => {
                let type_pointer = self.get_event_payload_def(event_name.as_str())?;
                Some((type_pointer, false, false))
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SystemInstruction {
    MapCollectionToPhysicalPartition {
        collection_index: u8,
        partition_num: PartitionNumber,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor, PartialOrd, Ord, Hash)]
pub enum PartitionDescription {
    Logical(PartitionOffset),
    Physical(PartitionNumber),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct IndexedStateSchema {
    pub fields: Option<(PartitionDescription, Vec<FieldSchema<BlueprintPayloadDef>>)>,
    pub collections: Vec<(
        PartitionDescription,
        BlueprintCollectionSchema<BlueprintPayloadDef>,
    )>,
    pub num_logical_partitions: u8,
}

impl IndexedStateSchema {
    pub fn from_schema(
        schema_hash: SchemaHash,
        schema: BlueprintStateSchemaInit,
        system_mappings: IndexMap<usize, PartitionNumber>,
    ) -> Self {
        let mut partition_offset = 0u8;

        let mut fields = None;
        if !schema.fields.is_empty() {
            let schema_fields = schema
                .fields
                .into_iter()
                .map(|field_schema| FieldSchema {
                    field: BlueprintPayloadDef::from_type_ref(field_schema.field, schema_hash),
                    condition: field_schema.condition,
                    transience: field_schema.transience,
                })
                .collect();
            fields = Some((
                PartitionDescription::Logical(PartitionOffset(partition_offset)),
                schema_fields,
            ));
            partition_offset += 1;
        };

        let mut collections = Vec::new();
        for (collection_index, collection_schema) in schema.collections.into_iter().enumerate() {
            let schema = collection_schema
                .map(|type_ref| BlueprintPayloadDef::from_type_ref(type_ref, schema_hash));

            if let Some(partition_num) = system_mappings.get(&collection_index) {
                collections.push((PartitionDescription::Physical(*partition_num), schema));
            } else {
                collections.push((
                    PartitionDescription::Logical(PartitionOffset(partition_offset)),
                    schema,
                ));
                partition_offset += 1;
            }
        }

        Self {
            fields,
            collections,
            num_logical_partitions: partition_offset,
        }
    }

    pub fn num_logical_partitions(&self) -> u8 {
        self.num_logical_partitions
    }

    pub fn num_fields(&self) -> usize {
        match &self.fields {
            Some((_, indices)) => indices.len(),
            _ => 0usize,
        }
    }

    pub fn get_partition(
        &self,
        collection_index: u8,
    ) -> Option<(PartitionDescription, BlueprintPartitionType)> {
        self.collections
            .get(collection_index as usize)
            .map(|(partition, schema)| {
                let partition_type = match schema {
                    BlueprintCollectionSchema::KeyValueStore(..) => {
                        BlueprintPartitionType::KeyValueCollection
                    }
                    BlueprintCollectionSchema::Index(..) => BlueprintPartitionType::IndexCollection,
                    BlueprintCollectionSchema::SortedIndex(..) => {
                        BlueprintPartitionType::SortedIndexCollection
                    }
                };
                (*partition, partition_type)
            })
    }

    pub fn get_field_payload_def(&self, field_index: u8) -> Option<BlueprintPayloadDef> {
        let (_partition, fields) = self.fields.clone()?;
        let field_schema = fields.get(field_index.clone() as usize)?;
        Some(field_schema.field.clone())
    }

    pub fn get_kv_key_payload_def(&self, collection_index: u8) -> Option<BlueprintPayloadDef> {
        let (_partition, schema) = self.collections.get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::KeyValueStore(key_value_store) => {
                Some(key_value_store.key.clone())
            }
            _ => None,
        }
    }

    pub fn get_kv_value_payload_def(
        &self,
        collection_index: u8,
    ) -> Option<(BlueprintPayloadDef, bool)> {
        let (_partition, schema) = self.collections.get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::KeyValueStore(key_value_store) => Some((
                key_value_store.value.clone(),
                key_value_store.allow_ownership,
            )),
            _ => None,
        }
    }

    pub fn get_index_payload_def_key(&self, collection_index: u8) -> Option<BlueprintPayloadDef> {
        let (_partition, schema) = self.collections.get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::Index(index) => Some(index.key.clone()),
            _ => None,
        }
    }

    pub fn get_index_payload_def_value(&self, collection_index: u8) -> Option<BlueprintPayloadDef> {
        let (_partition, schema) = self.collections.get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::Index(index) => Some(index.value.clone()),
            _ => None,
        }
    }

    pub fn get_sorted_index_payload_def_key(
        &self,
        collection_index: u8,
    ) -> Option<BlueprintPayloadDef> {
        let (_partition, schema) = self.collections.get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::SortedIndex(index) => Some(index.key.clone()),
            _ => None,
        }
    }

    pub fn get_sorted_index_payload_def_value(
        &self,
        collection_index: u8,
    ) -> Option<BlueprintPayloadDef> {
        let (_partition, schema) = self.collections.get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::SortedIndex(index) => Some(index.value.clone()),
            _ => None,
        }
    }

    pub fn fields_partition(&self) -> Option<PartitionDescription> {
        match &self.fields {
            Some((partition, ..)) => Some(partition.clone()),
            _ => None,
        }
    }

    pub fn field(
        &self,
        field_index: u8,
    ) -> Option<(PartitionDescription, FieldSchema<BlueprintPayloadDef>)> {
        match &self.fields {
            Some((partition, fields)) => {
                let field_index: usize = field_index.into();
                fields
                    .get(field_index)
                    .cloned()
                    .map(|f| (partition.clone(), f))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default, ScryptoSbor, ManifestSbor)]
pub struct PackageDefinition {
    pub blueprints: IndexMap<String, BlueprintDefinitionInit>,
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
    pub feature_set: IndexSet<String>,
    pub dependencies: IndexSet<GlobalAddress>,
    pub schema: BlueprintSchemaInit,
    pub royalty_config: PackageRoyaltyConfig,
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
