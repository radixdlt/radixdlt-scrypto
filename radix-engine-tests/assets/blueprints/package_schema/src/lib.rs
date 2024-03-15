use sbor::*;
use scrypto::prelude::*;
use scrypto::radix_blueprint_schema_init::*;

#[blueprint]
mod schema_component {
    struct SchemaComponent {}

    impl SchemaComponent {
        pub fn create_component() -> Global<SchemaComponent> {
            let component = Self {}.instantiate();
            component.prepare_to_globalize(OwnerRole::None).globalize()
        }
    }
}

#[no_mangle]
pub extern "C" fn SchemaComponent2_invalid_output(_input: u64) -> Slice {
    ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&()).unwrap())
}

#[no_mangle]
pub extern "C" fn dummy_export(_input: u64) -> Slice {
    ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&()).unwrap())
}

#[no_mangle]
pub extern "C" fn SchemaComponent2_schema() -> Slice {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
    let mut fields = Vec::new();
    fields.push(FieldSchema::static_field(
        aggregator.add_child_type_and_descendents::<()>(),
    ));

    let mut functions = index_map_new();

    functions.insert(
        "invalid_output".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<u8>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "unit".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "bool".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<bool>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "i8".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<i8>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "i16".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<i16>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "i32".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<i32>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "i64".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<i64>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "i128".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<i128>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "u8".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<u8>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "u16".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<u16>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "u32".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<u32>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "u64".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<u64>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "u128".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<u128>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "result".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<Result<(), ()>>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "tree_map".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<IndexMap<(), ()>>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );
    functions.insert(
        "hash_set".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(aggregator.add_child_type_and_descendents::<HashSet<()>>()),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "dummy_export".to_string(),
        },
    );

    let schema = BlueprintSchemaInit {
        generics: vec![],
        schema: generate_full_schema(aggregator),
        state: BlueprintStateSchemaInit {
            fields,
            collections: vec![],
        },
        events: BlueprintEventSchemaInit::default(),
        types: BlueprintTypeSchemaInit::default(),
        functions: BlueprintFunctionsSchemaInit { functions },
        hooks: BlueprintHooksInit::default(),
    };

    let function_auth: IndexMap<String, AccessRule> = indexmap!(
        "invalid_output".to_string() => AccessRule::AllowAll,
        "unit".to_string() => AccessRule::AllowAll,
        "bool".to_string() => AccessRule::AllowAll,
        "i8".to_string() => AccessRule::AllowAll,
        "i16".to_string() => AccessRule::AllowAll,
        "i32".to_string() => AccessRule::AllowAll,
        "i64".to_string() => AccessRule::AllowAll,
        "i128".to_string() => AccessRule::AllowAll,
        "u8".to_string() => AccessRule::AllowAll,
        "u16".to_string() => AccessRule::AllowAll,
        "u32".to_string() => AccessRule::AllowAll,
        "u64".to_string() => AccessRule::AllowAll,
        "u128".to_string() => AccessRule::AllowAll,
        "result".to_string() => AccessRule::AllowAll,
        "tree_map".to_string() => AccessRule::AllowAll,
        "hash_set".to_string() => AccessRule::AllowAll,
    );

    let return_data = scrypto::blueprints::package::BlueprintDefinitionInit {
        blueprint_type: scrypto::blueprints::package::BlueprintType::default(),
        is_transient: false,
        dependencies: indexset!(),
        feature_set: indexset!(),
        schema,
        royalty_config: PackageRoyaltyConfig::default(),
        auth_config: scrypto::blueprints::package::AuthConfig {
            function_auth: scrypto::blueprints::package::FunctionAuth::AccessRules(function_auth),
            method_auth: scrypto::blueprints::package::MethodAuthTemplate::AllowAll,
        },
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap(),
    )
}

#[blueprint]
mod simple {
    struct Simple {
        state: u32,
    }

    impl Simple {
        pub fn new() -> Global<Simple> {
            Self { state: 0 }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn get_state(&self) -> u32 {
            self.state
        }

        pub fn set_state(&mut self, new_state: u32) {
            self.state = new_state;
        }

        pub fn custom_types() -> (
            Decimal,
            PackageAddress,
            KeyValueStore<String, String>,
            Bucket,
            Proof,
            Vault,
        ) {
            unreachable!()
        }
    }
}
