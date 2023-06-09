use sbor::basic_well_known_types::*;
use sbor::*;
use scrypto::blueprints::package::*;
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
    fields.push(FieldSchema::normal(
        aggregator.add_child_type_and_descendents::<()>(),
    ));

    let mut functions = BTreeMap::new();
    functions.insert(
        "f".to_string(),
        FunctionSetup {
            receiver: None,
            input: LocalTypeIndex::WellKnown(ANY_ID),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("LargeReturnSize_f"),
        },
    );

    let schema = generate_full_schema(aggregator);

    let blueprint = BlueprintSchema {
        fields,
        collections: vec![],
    };

    let function_auth: BTreeMap<String, AccessRule> = btreemap!(
        "f".to_string() => AccessRule::AllowAll,
    );

    let return_data = scrypto::blueprints::package::BlueprintSetup {
        outer_blueprint: None,
        functions,
        dependencies: btreeset!(),
        features: btreeset!(),
        blueprint,
        schema,
        event_schema: [].into(),
        function_auth,
        royalty_config: RoyaltyConfig::default(),
        template: scrypto::blueprints::package::MethodAuthTemplate {
            method_auth_template: btreemap!(),
            outer_method_auth_template: btreemap!(),
        },
        virtual_lazy_load_functions: BTreeMap::new(),
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap(),
    )
}

#[no_mangle]
pub extern "C" fn MaxReturnSize_schema() -> Slice {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

    let mut fields = Vec::new();
    fields.push(FieldSchema::normal(
        aggregator.add_child_type_and_descendents::<()>(),
    ));

    let mut functions = BTreeMap::new();
    functions.insert(
        "f".to_string(),
        FunctionSetup {
            receiver: None,
            input: LocalTypeIndex::WellKnown(ANY_ID),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("MaxReturnSize_f"),
        },
    );

    let blueprint = BlueprintSchema {
        fields,
        collections: vec![],
    };

    let function_auth: BTreeMap<String, AccessRule> = btreemap!(
        "f".to_string() => AccessRule::AllowAll,
    );

    let return_data = scrypto::blueprints::package::BlueprintSetup {
        functions,
        outer_blueprint: None,
        dependencies: btreeset!(),
        features: btreeset!(),
        blueprint,
        schema: generate_full_schema(aggregator),
        event_schema: [].into(),
        function_auth,
        royalty_config: RoyaltyConfig::default(),
        template: scrypto::blueprints::package::MethodAuthTemplate {
            method_auth_template: btreemap!(),
            outer_method_auth_template: btreemap!(),
        },
        virtual_lazy_load_functions: BTreeMap::new(),
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap(),
    )
}

#[no_mangle]
pub extern "C" fn ZeroReturnSize_schema() -> Slice {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

    let mut fields = Vec::new();
    fields.push(FieldSchema::normal(
        aggregator.add_child_type_and_descendents::<()>(),
    ));

    let mut functions = BTreeMap::new();
    functions.insert(
        "f".to_string(),
        FunctionSetup {
            receiver: None,
            input: LocalTypeIndex::WellKnown(ANY_ID),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export: ExportSchema::normal("ZeroReturnSize_f"),
        },
    );

    let blueprint = BlueprintSchema {
        fields,
        collections: vec![],
    };

    let function_auth: BTreeMap<String, AccessRule> = btreemap!(
        "f".to_string() => AccessRule::AllowAll,
    );

    let return_data = scrypto::blueprints::package::BlueprintSetup {
        outer_blueprint: None,
        dependencies: btreeset!(),
        features: btreeset!(),
        schema: generate_full_schema(aggregator),
        blueprint,
        event_schema: [].into(),
        function_auth,
        royalty_config: RoyaltyConfig::default(),
        template: scrypto::blueprints::package::MethodAuthTemplate {
            method_auth_template: btreemap!(),
            outer_method_auth_template: btreemap!(),
        },
        virtual_lazy_load_functions: BTreeMap::new(),
        functions,
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap(),
    )
}
