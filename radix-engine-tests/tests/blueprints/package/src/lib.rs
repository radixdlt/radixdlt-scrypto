use sbor::basic_well_known_types::*;
use sbor::*;
use scrypto::prelude::*;
use scrypto::schema::*;

static LARGE: u32 = u32::MAX / 2;
static MAX: u32 = u32::MAX;
static ZERO: u32 = 0;

#[no_mangle]
pub extern "C" fn LargeReturnSize_f(_args: u64) -> Slice {
    Slice(LARGE as u64)
}

#[no_mangle]
pub extern "C" fn MaxReturnSize_f(_args: u64) -> Slice {
    Slice(MAX as u64)
}

#[no_mangle]
pub extern "C" fn ZeroReturnSize_f(_args: u64) -> Slice {
    Slice(ZERO as u64)
}

#[no_mangle]
pub extern "C" fn LargeReturnSize_schema() -> Slice {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

    let mut fields = Vec::new();
    fields.push(FieldSchema::static_field(
        aggregator.add_child_type_and_descendents::<()>(),
    ));

    let mut functions = BTreeMap::new();
    functions.insert(
        "f".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: LocalTypeIndex::WellKnown(ANY_ID),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: "LargeReturnSize_f".to_string(),
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
        functions: BlueprintFunctionsSchemaInit {
            functions,
            virtual_lazy_load_functions: BTreeMap::default(),
        },
    };

    let function_auth: BTreeMap<String, AccessRule> = btreemap!(
        "f".to_string() => AccessRule::AllowAll,
    );

    let return_data = scrypto::blueprints::package::BlueprintDefinitionInit {
        outer_blueprint: None,
        dependencies: btreeset!(),
        feature_set: btreeset!(),
        schema,
        royalty_config: RoyaltyConfig::default(),
        auth_config: scrypto::blueprints::package::AuthConfig {
            function_auth,
            method_auth: scrypto::blueprints::package::MethodAuthTemplate::Static {
                auth: btreemap!(),
                outer_auth: btreemap!(),
            },
        },
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap(),
    )
}

#[no_mangle]
pub extern "C" fn MaxReturnSize_schema() -> Slice {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

    let mut fields = Vec::new();
    fields.push(FieldSchema::static_field(
        aggregator.add_child_type_and_descendents::<()>(),
    ));

    let mut functions = BTreeMap::new();
    functions.insert(
        "f".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: LocalTypeIndex::WellKnown(ANY_ID),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: "MaxReturnSize_f".to_string(),
        },
    );

    let function_auth: BTreeMap<String, AccessRule> = btreemap!(
        "f".to_string() => AccessRule::AllowAll,
    );

    let schema = BlueprintSchemaInit {
        generics: vec![],
        schema: generate_full_schema(aggregator),
        state: BlueprintStateSchemaInit {
            fields,
            collections: vec![],
        },
        events: BlueprintEventSchemaInit::default(),
        functions: BlueprintFunctionsSchemaInit {
            functions,
            virtual_lazy_load_functions: BTreeMap::default(),
        },
    };

    let return_data = scrypto::blueprints::package::BlueprintDefinitionInit {
        outer_blueprint: None,
        dependencies: btreeset!(),
        feature_set: btreeset!(),
        schema,
        royalty_config: RoyaltyConfig::default(),
        auth_config: scrypto::blueprints::package::AuthConfig {
            function_auth,
            method_auth: scrypto::blueprints::package::MethodAuthTemplate::Static {
                auth: btreemap!(),
                outer_auth: btreemap!(),
            },
        },
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap(),
    )
}

#[no_mangle]
pub extern "C" fn ZeroReturnSize_schema() -> Slice {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

    let mut fields = Vec::new();
    fields.push(FieldSchema::static_field(
        aggregator.add_child_type_and_descendents::<()>(),
    ));

    let mut functions = BTreeMap::new();
    functions.insert(
        "f".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: LocalTypeIndex::WellKnown(ANY_ID),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: "ZeroReturnSize_f".to_string(),
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
        functions: BlueprintFunctionsSchemaInit {
            functions,
            virtual_lazy_load_functions: BTreeMap::default(),
        },
    };

    let function_auth: BTreeMap<String, AccessRule> = btreemap!(
        "f".to_string() => AccessRule::AllowAll,
    );

    let return_data = scrypto::blueprints::package::BlueprintDefinitionInit {
        outer_blueprint: None,
        dependencies: btreeset!(),
        feature_set: btreeset!(),
        schema,
        royalty_config: RoyaltyConfig::default(),
        auth_config: scrypto::blueprints::package::AuthConfig {
            function_auth,
            method_auth: scrypto::blueprints::package::MethodAuthTemplate::Static {
                auth: btreemap!(),
                outer_auth: btreemap!(),
            },
        },
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap(),
    )
}

#[no_mangle]
pub extern "C" fn BadFunctionSchema_f(_args: u64) -> Slice {
    ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&()).unwrap())
}

#[no_mangle]
pub extern "C" fn BadFunctionSchema_schema() -> Slice {
    let mut functions = BTreeMap::new();
    functions.insert(
        "f".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: LocalTypeIndex::SchemaLocalIndex(1usize),
            output: LocalTypeIndex::SchemaLocalIndex(2usize),
            export: "BadFunctionSchema_f".to_string(),
        },
    );

    // Empty Schema
    let empty_schema = ScryptoSchema {
        type_kinds: Vec::new(),
        type_metadata: Vec::new(),
        type_validations: Vec::new(),
    };

    let return_data = scrypto::blueprints::package::BlueprintDefinitionInit {
        outer_blueprint: None,
        dependencies: btreeset!(),
        feature_set: btreeset!(),
        schema: BlueprintSchemaInit {
            generics: vec![],
            schema: empty_schema,
            state: BlueprintStateSchemaInit {
                fields: vec![],
                collections: vec![],
            },
            events: BlueprintEventSchemaInit::default(),
            functions: BlueprintFunctionsSchemaInit {
                functions,
                virtual_lazy_load_functions: BTreeMap::default(),
            },
        },
        royalty_config: RoyaltyConfig::default(),
        auth_config: scrypto::blueprints::package::AuthConfig {
            function_auth: btreemap!(
                "f".to_string() => AccessRule::AllowAll,
            ),
            method_auth: scrypto::blueprints::package::MethodAuthTemplate::Static {
                auth: btreemap!(),
                outer_auth: btreemap!(),
            },
        },
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap(),
    )
}
