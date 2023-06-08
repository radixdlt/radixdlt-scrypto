use sbor::*;
use scrypto::prelude::*;
use scrypto::schema::*;

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
    fields.push(FieldSchema::normal(
        aggregator.add_child_type_and_descendents::<()>(),
    ));

    let mut functions = BTreeMap::new();

    functions.insert(
        "invalid_output".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<()>(),
            output: aggregator.add_child_type_and_descendents::<u8>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "unit".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<()>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "bool".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<bool>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "i8".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<i8>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "i16".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<i16>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "i32".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<i32>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "i64".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<i64>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "i128".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<i128>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "u8".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<u8>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "u16".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<u16>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "u32".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<u32>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "u64".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<u64>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "u128".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<u128>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "result".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<Result<(), ()>>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "tree_map".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<BTreeMap<(), ()>>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );
    functions.insert(
        "hash_set".to_string(),
        FunctionSchema {
            receiver: None,
            input: aggregator.add_child_type_and_descendents::<HashSet<()>>(),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("dummy_export"),
        },
    );

    let schema = generate_full_schema(aggregator);

    let blueprint = BlueprintSchema {
        outer_blueprint: None,
        fields,
        collections: vec![],
        functions,
        dependencies: btreeset!(),
        features: btreeset!(),
    };

    let function_auth: BTreeMap<String, AccessRule> = btreemap!(
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

    let return_data = scrypto::blueprints::package::BlueprintSetup {
        schema,
        blueprint,
        event_schema: [].into(),
        function_auth,
        royalty_config: RoyaltyConfig::default(),
        template: scrypto::blueprints::package::BlueprintTemplate {
            method_auth_template: btreemap!(),
            outer_method_auth_template: btreemap!(),
        },
        virtual_lazy_load_functions: BTreeMap::new(),
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
            todo!()
        }
    }
}
