use scrypto::prelude::*;
use scrypto::radix_blueprint_schema_init::*;

#[no_mangle]
pub extern "C" fn BadFunctionSchema_f(_args: u64) -> Slice {
    ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto::scrypto_encode(&()).unwrap())
}

#[no_mangle]
pub extern "C" fn BadFunctionSchema_schema() -> Slice {
    let mut functions = index_map_new();
    functions.insert(
        "f".to_string(),
        FunctionSchemaInit {
            receiver: None,
            input: TypeRef::Static(LocalTypeId::SchemaLocalIndex(1usize)),
            output: TypeRef::Static(LocalTypeId::SchemaLocalIndex(2usize)),
            export: "BadFunctionSchema_f".to_string(),
        },
    );

    // Empty Schema
    let empty_schema = Schema::empty().into_versioned();

    let return_data = scrypto::blueprints::package::BlueprintDefinitionInit {
        blueprint_type: scrypto::blueprints::package::BlueprintType::default(),
        is_transient: false,
        dependencies: indexset!(),
        feature_set: indexset!(),
        schema: BlueprintSchemaInit {
            generics: vec![],
            schema: empty_schema,
            state: BlueprintStateSchemaInit {
                fields: vec![],
                collections: vec![],
            },
            events: BlueprintEventSchemaInit::default(),
            types: BlueprintTypeSchemaInit::default(),
            functions: BlueprintFunctionsSchemaInit { functions },
            hooks: BlueprintHooksInit::default(),
        },
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
