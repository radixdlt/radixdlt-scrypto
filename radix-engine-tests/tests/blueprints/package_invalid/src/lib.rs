use sbor::*;
use scrypto::prelude::*;

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
            input: TypeRef::Static(LocalTypeIndex::SchemaLocalIndex(1usize)),
            output: TypeRef::Static(LocalTypeIndex::SchemaLocalIndex(2usize)),
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
        blueprint_type: scrypto::blueprints::package::BlueprintType::default(),
        is_transient: false,
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
