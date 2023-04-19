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

    let mut substates = Vec::new();
    substates.push(aggregator.add_child_type_and_descendents::<()>());

    let mut functions = BTreeMap::new();
    functions.insert(
        "f".to_string(),
        FunctionSchema {
            receiver: None,
            input: LocalTypeIndex::WellKnown(ANY_ID),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export_name: "LargeReturnSize_f".to_string(),
        },
    );

    let schema = BlueprintSchema {
        parent: None,
        schema: generate_full_schema(aggregator),
        substates,
        functions,
        virtual_lazy_load_functions: BTreeMap::new(),
        event_schema: [].into(),
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&schema).unwrap(),
    )
}

#[no_mangle]
pub extern "C" fn MaxReturnSize_schema() -> Slice {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
    let mut substates = Vec::new();
    substates.push(aggregator.add_child_type_and_descendents::<()>());

    let mut functions = BTreeMap::new();
    functions.insert(
        "f".to_string(),
        FunctionSchema {
            receiver: None,
            input: LocalTypeIndex::WellKnown(ANY_ID),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export_name: "MaxReturnSize_f".to_string(),
        },
    );

    let schema = BlueprintSchema {
        parent: None,
        schema: generate_full_schema(aggregator),
        substates,
        functions,
        virtual_lazy_load_functions: BTreeMap::new(),
        event_schema: [].into(),
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&schema).unwrap(),
    )
}

#[no_mangle]
pub extern "C" fn ZeroReturnSize_schema() -> Slice {
    let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

    let mut substates = Vec::new();
    substates.push(aggregator.add_child_type_and_descendents::<()>());

    let mut functions = BTreeMap::new();
    functions.insert(
        "f".to_string(),
        FunctionSchema {
            receiver: None,
            input: LocalTypeIndex::WellKnown(ANY_ID),
            output: aggregator.add_child_type_and_descendents::<()>(),
            export_name: "ZeroReturnSize_f".to_string(),
        },
    );

    let schema = BlueprintSchema {
        parent: None,
        schema: generate_full_schema(aggregator),
        substates,
        functions,
        virtual_lazy_load_functions: BTreeMap::new(),
        event_schema: [].into(),
    };

    ::scrypto::engine::wasm_api::forget_vec(
        ::scrypto::data::scrypto::scrypto_encode(&schema).unwrap(),
    )
}
