use sbor::basic_well_known_types::*;
use sbor::*;
use scrypto::prelude::*;
use scrypto::radix_blueprint_schema_init::*;

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

    let mut functions = index_map_new();
    functions.insert(
        "f".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
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
        types: BlueprintTypeSchemaInit::default(),
        functions: BlueprintFunctionsSchemaInit { functions },
        hooks: BlueprintHooksInit::default(),
    };

    let return_data = scrypto::blueprints::package::BlueprintDefinitionInit {
        blueprint_type: scrypto::blueprints::package::BlueprintType::default(),
        is_transient: false,
        dependencies: indexset!(),
        feature_set: indexset!(),
        schema,
        royalty_config: PackageRoyaltyConfig::default(),
        auth_config: scrypto::blueprints::package::AuthConfig {
            function_auth: scrypto::blueprints::package::FunctionAuth::AllowAll,
            method_auth: scrypto::blueprints::package::MethodAuthTemplate::AllowAll,
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

    let mut functions = index_map_new();
    functions.insert(
        "f".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
            export: "MaxReturnSize_f".to_string(),
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

    let return_data = scrypto::blueprints::package::BlueprintDefinitionInit {
        blueprint_type: scrypto::blueprints::package::BlueprintType::default(),
        is_transient: false,
        dependencies: indexset!(),
        feature_set: indexset!(),
        schema,
        royalty_config: PackageRoyaltyConfig::default(),
        auth_config: scrypto::blueprints::package::AuthConfig {
            function_auth: scrypto::blueprints::package::FunctionAuth::AllowAll,
            method_auth: scrypto::blueprints::package::MethodAuthTemplate::AllowAll,
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

    let mut functions = index_map_new();
    functions.insert(
        "f".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
            output: TypeRef::Static(aggregator.add_child_type_and_descendents::<()>()),
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
        types: BlueprintTypeSchemaInit::default(),
        functions: BlueprintFunctionsSchemaInit { functions },
        hooks: BlueprintHooksInit::default(),
    };

    let return_data = scrypto::blueprints::package::BlueprintDefinitionInit {
        blueprint_type: scrypto::blueprints::package::BlueprintType::default(),
        is_transient: false,
        dependencies: indexset!(),
        feature_set: indexset!(),
        schema,
        royalty_config: PackageRoyaltyConfig::default(),
        auth_config: scrypto::blueprints::package::AuthConfig {
            function_auth: scrypto::blueprints::package::FunctionAuth::AllowAll,
            method_auth: scrypto::blueprints::package::MethodAuthTemplate::AllowAll,
        },
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&return_data).unwrap(),
    )
}
