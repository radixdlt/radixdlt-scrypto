use radix_engine_common::data::scrypto::model::NonFungibleLocalId;
use radix_engine_common::data::scrypto::*;
use radix_engine_common::*;
use sbor::rust::boxed::Box;
use sbor::rust::cell::RefCell;
use sbor::rust::collections::BTreeMap;
use sbor::rust::collections::BTreeSet;
use sbor::rust::collections::HashMap;
use sbor::rust::collections::HashSet;
use sbor::rust::hash::Hash;
use sbor::rust::rc::Rc;
use sbor::rust::string::String;
use sbor::rust::vec;
use sbor::*;

#[test]
fn test_args() {
    #[derive(Sbor)]
    struct A {
        a: u32,
        b: String,
    }

    assert_eq!(
        scrypto_args!(1u32, "abc"),
        scrypto_encode(&A {
            a: 1,
            b: "abc".to_owned(),
        })
        .unwrap()
    )
}

#[test]
fn test_args_with_non_fungible_local_id() {
    let id = NonFungibleLocalId::integer(1);
    let _x = scrypto_args!(BTreeSet::from([id]));
}

#[test]
fn test_encode_deep_scrypto_values() {
    // This test tests that the ScryptoValue Encode implementation correctly increments the depth

    // Test deep scrypto value vecs
    let valid_value = build_value_of_vec_of_depth(SCRYPTO_SBOR_V1_MAX_DEPTH);
    assert!(scrypto_encode(&valid_value).is_ok());

    let invalid_value = build_value_of_vec_of_depth(SCRYPTO_SBOR_V1_MAX_DEPTH + 1);
    assert_eq!(
        scrypto_encode(&invalid_value),
        Err(EncodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );

    // Test deep scrypto value tuples
    let valid_value = build_value_of_tuple_of_depth(SCRYPTO_SBOR_V1_MAX_DEPTH);
    assert!(scrypto_encode(&valid_value).is_ok());

    let invalid_value = build_value_of_tuple_of_depth(SCRYPTO_SBOR_V1_MAX_DEPTH + 1);
    assert_eq!(
        scrypto_encode(&invalid_value),
        Err(EncodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );
}

#[test]
fn test_decode_deep_scrypto_values() {
    // This test tests that the ScryptoValue Decode implementation correctly increments the depth

    // Test deep scrypto value vecs
    let valid_payload =
        encode_ignore_depth(&build_value_of_vec_of_depth(SCRYPTO_SBOR_V1_MAX_DEPTH));
    assert!(scrypto_decode::<ScryptoValue>(&valid_payload).is_ok());

    let invalid_payload =
        encode_ignore_depth(&build_value_of_vec_of_depth(SCRYPTO_SBOR_V1_MAX_DEPTH + 1));
    assert_eq!(
        scrypto_decode::<ScryptoValue>(&invalid_payload),
        Err(DecodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );

    // Test deep scrypto value tuples
    let valid_payload =
        encode_ignore_depth(&build_value_of_tuple_of_depth(SCRYPTO_SBOR_V1_MAX_DEPTH));
    assert!(scrypto_decode::<ScryptoValue>(&valid_payload).is_ok());

    let invalid_payload = encode_ignore_depth(&build_value_of_tuple_of_depth(
        SCRYPTO_SBOR_V1_MAX_DEPTH + 1,
    ));
    assert_eq!(
        scrypto_decode::<ScryptoValue>(&invalid_payload),
        Err(DecodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );
}

#[test]
fn test_encode_deep_typed_codecs() {
    // This test tests that various typed codecs have an Encode implementation which correctly increments the depth
    // It also tests that depth behaves identically to the ScryptoValue interpretation

    // Test deep vecs
    let valid_value = wrap_in_64_collections(Option::<String>::None);
    assert!(scrypto_encode(&valid_value).is_ok());
    let valid_value_as_scrypto_value =
        decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&valid_value));
    assert!(scrypto_encode(&valid_value_as_scrypto_value).is_ok());

    let invalid_value = vec![wrap_in_64_collections(Option::<String>::None)];
    assert_eq!(
        scrypto_encode(&invalid_value),
        Err(EncodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );
    let invalid_value_as_scrypto_value =
        decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&invalid_value));
    assert_eq!(
        scrypto_encode(&invalid_value_as_scrypto_value),
        Err(EncodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );

    // Test deep nested types
    let valid_value = build_nested_struct_of_depth(SCRYPTO_SBOR_V1_MAX_DEPTH);
    assert!(scrypto_encode(&valid_value).is_ok());
    let valid_value_as_scrypto_value =
        decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&valid_value));
    assert!(scrypto_encode(&valid_value_as_scrypto_value).is_ok());

    let invalid_value = vec![build_nested_struct_of_depth(SCRYPTO_SBOR_V1_MAX_DEPTH)];
    assert_eq!(
        scrypto_encode(&invalid_value),
        Err(EncodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );
    let invalid_value_as_scrypto_value =
        decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&invalid_value));
    assert_eq!(
        scrypto_encode(&invalid_value_as_scrypto_value),
        Err(EncodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );

    // Test hashmaps
    let valid_value = wrap_in_hashmap(wrap_in_hashmap(build_nested_struct_of_depth(
        SCRYPTO_SBOR_V1_MAX_DEPTH - 2,
    )));
    assert!(scrypto_encode(&valid_value).is_ok());
    let valid_value_as_scrypto_value =
        decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&valid_value));
    assert!(scrypto_encode(&valid_value_as_scrypto_value).is_ok());
    let invalid_value = wrap_in_hashmap(wrap_in_hashmap(wrap_in_hashmap(
        build_nested_struct_of_depth(SCRYPTO_SBOR_V1_MAX_DEPTH - 2),
    )));
    assert_eq!(
        scrypto_encode(&invalid_value),
        Err(EncodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );
    let invalid_value_as_scrypto_value =
        decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&invalid_value));
    assert_eq!(
        scrypto_encode(&invalid_value_as_scrypto_value),
        Err(EncodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );

    // Test hashsets + tuples
    let valid_value = wrap_in_61_vecs(Some(wrap_in_tuple_single(wrap_in_hashset("hello"))));
    assert!(scrypto_encode(&valid_value).is_ok());
    let valid_value_as_scrypto_value =
        decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&valid_value));
    assert!(scrypto_encode(&valid_value_as_scrypto_value).is_ok());

    let invalid_value = vec![wrap_in_61_vecs(Some(wrap_in_tuple_single(
        wrap_in_hashset("hello"),
    )))];
    assert_eq!(
        scrypto_encode(&invalid_value),
        Err(EncodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );
    let invalid_value_as_scrypto_value =
        decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&invalid_value));
    assert_eq!(
        scrypto_encode(&invalid_value_as_scrypto_value),
        Err(EncodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );
}

#[test]
fn test_decode_deep_typed_codecs() {
    // This test tests that various typed codecs have a Decode implementation which correctly increments the depth
    // It also tests that depth behaves identically to the ScryptoValue interpretation

    // Test deep vecs
    let valid_payload = encode_ignore_depth(&wrap_in_64_collections(Option::<String>::None));
    assert!(scrypto_decode::<SixtyFourDeepCollection<String>>(&valid_payload).is_ok());
    assert!(scrypto_decode::<ScryptoValue>(&valid_payload).is_ok());

    let invalid_payload =
        encode_ignore_depth(&vec![wrap_in_64_collections(Option::<String>::None)]); // 65 deep
    assert_eq!(
        scrypto_decode::<Vec<SixtyFourDeepCollection<String>>>(&invalid_payload),
        Err(DecodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );
    assert_eq!(
        scrypto_decode::<ScryptoValue>(&invalid_payload),
        Err(DecodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );

    // Test deep nested types
    let valid_payload =
        encode_ignore_depth(&build_nested_struct_of_depth(SCRYPTO_SBOR_V1_MAX_DEPTH));
    assert!(scrypto_decode::<NestedType>(&valid_payload).is_ok());
    assert!(scrypto_decode::<ScryptoValue>(&valid_payload).is_ok());

    let invalid_payload = encode_ignore_depth(&vec![build_nested_struct_of_depth(
        SCRYPTO_SBOR_V1_MAX_DEPTH,
    )]); // 65 deep
    assert!(matches!(
        scrypto_decode::<Vec<NestedType>>(&invalid_payload),
        Err(DecodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    ));
    assert_eq!(
        scrypto_decode::<ScryptoValue>(&invalid_payload),
        Err(DecodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );

    // Test hashmaps
    let valid_payload = encode_ignore_depth(&wrap_in_hashmap(wrap_in_hashmap(
        build_nested_struct_of_depth(SCRYPTO_SBOR_V1_MAX_DEPTH - 2),
    )));
    assert!(scrypto_decode::<HashMap<u8, HashMap<u8, NestedType>>>(&valid_payload).is_ok());
    assert!(scrypto_decode::<ScryptoValue>(&valid_payload).is_ok());
    let invalid_payload = encode_ignore_depth(&wrap_in_hashmap(wrap_in_hashmap(wrap_in_hashmap(
        build_nested_struct_of_depth(SCRYPTO_SBOR_V1_MAX_DEPTH - 2),
    ))));
    assert!(matches!(
        scrypto_decode::<HashMap<u8, HashMap<u8, HashMap<u8, NestedType>>>>(&invalid_payload),
        Err(DecodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    ));
    assert_eq!(
        scrypto_decode::<ScryptoValue>(&invalid_payload),
        Err(DecodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );

    // Test hashsets + tuples
    let valid_payload = encode_ignore_depth(&wrap_in_61_vecs(Some(wrap_in_tuple_single(
        wrap_in_hashset("hello"),
    ))));
    assert!(scrypto_decode::<SixtyOneDeepVec<(HashSet<String>,)>>(&valid_payload).is_ok());
    assert!(scrypto_decode::<ScryptoValue>(&valid_payload).is_ok());

    let invalid_payload = encode_ignore_depth(&vec![wrap_in_61_vecs(Some(wrap_in_tuple_single(
        wrap_in_hashset("hello"),
    )))]);
    assert_eq!(
        scrypto_decode::<Vec<SixtyOneDeepVec<(HashSet<String>,)>>>(&invalid_payload),
        Err(DecodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );
    assert_eq!(
        scrypto_decode::<ScryptoValue>(&invalid_payload),
        Err(DecodeError::MaxDepthExceeded(SCRYPTO_SBOR_V1_MAX_DEPTH))
    );
}

fn encode_ignore_depth<V: ScryptoEncode>(value: &V) -> Vec<u8> {
    let mut buf = Vec::new();
    let encoder = VecEncoder::<ScryptoCustomValueKind>::new(&mut buf, 255);
    encoder
        .encode_payload(value, SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
        .unwrap();
    buf
}

fn decode_ignore_depth<'a, T: ScryptoDecode>(payload: &'a [u8]) -> T {
    let decoder = VecDecoder::<ScryptoCustomValueKind>::new(payload, 255);
    decoder
        .decode_payload(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
        .unwrap()
}

fn build_value_of_vec_of_depth(depth: usize) -> ScryptoValue {
    let mut value = ScryptoValue::Array {
        element_value_kind: ValueKind::Array,
        elements: vec![],
    };
    let loop_count = depth - 1;
    for _ in 0..loop_count {
        value = ScryptoValue::Array {
            element_value_kind: ValueKind::Array,
            elements: vec![value],
        };
    }
    value
}

fn build_value_of_tuple_of_depth(depth: usize) -> ScryptoValue {
    let mut value = ScryptoValue::Tuple { fields: vec![] };
    let loop_count = depth - 1;
    for _ in 0..loop_count {
        value = ScryptoValue::Tuple {
            fields: vec![value],
        };
    }
    value
}

#[derive(Sbor, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct NestedType {
    inner: Box<Rc<Option<RefCell<NestedType>>>>,
}

fn build_nested_struct_of_depth(depth: usize) -> NestedType {
    assert!(depth % 2 == 0);
    assert!(depth >= 2);

    // Note - each nesting introduces 2 depth - one for the NestedType, another for the Option (the Box/Rc/RefCell should be transparent)
    let mut value = NestedType {
        inner: Box::new(Rc::new(None)),
    };
    let loop_count = (depth / 2) - 1;
    for _ in 0..loop_count {
        value = NestedType {
            inner: Box::new(Rc::new(Some(RefCell::new(value)))),
        };
    }
    value
}

type SixtyOneDeepVec<T> =
    SixteenDeepVec<SixteenDeepVec<SixteenDeepVec<FourDeepVec<FourDeepVec<FourDeepVec<Vec<T>>>>>>>;
type SixteenDeepVec<T> = FourDeepVec<FourDeepVec<FourDeepVec<FourDeepVec<T>>>>;
type FourDeepVec<T> = Vec<Vec<Vec<Vec<T>>>>;

fn wrap_in_61_vecs<T>(inner: Option<T>) -> SixtyOneDeepVec<T> {
    vec![wrap_in_16_vecs(Some(wrap_in_16_vecs(Some(
        wrap_in_16_vecs(Some(wrap_in_4_vecs(Some(wrap_in_4_vecs(Some(
            wrap_in_4_vecs(inner),
        )))))),
    ))))]
}

fn wrap_in_16_vecs<T>(inner: Option<T>) -> SixteenDeepVec<T> {
    wrap_in_4_vecs(Some(wrap_in_4_vecs(Some(wrap_in_4_vecs(Some(
        wrap_in_4_vecs(inner),
    ))))))
}

fn wrap_in_4_vecs<T>(inner: Option<T>) -> FourDeepVec<T> {
    let inner = match inner {
        Some(inner) => vec![inner],
        None => vec![],
    };
    vec![vec![vec![inner]]]
}

type SixtyFourDeepCollection<T> =
    SixteenDeepCollection<SixteenDeepCollection<SixteenDeepCollection<SixteenDeepCollection<T>>>>;

fn wrap_in_64_collections<T: Eq + Ord + Eq>(inner: Option<T>) -> SixtyFourDeepCollection<T> {
    wrap_in_16_collections(Some(wrap_in_16_collections(Some(wrap_in_16_collections(
        Some(wrap_in_16_collections(inner)),
    )))))
}

type SixteenDeepCollection<T> =
    FourDeepCollection<FourDeepCollection<FourDeepCollection<FourDeepCollection<T>>>>;

fn wrap_in_16_collections<T: Eq + Ord + Eq>(inner: Option<T>) -> SixteenDeepCollection<T> {
    wrap_in_4_collections(Some(wrap_in_4_collections(Some(wrap_in_4_collections(
        Some(wrap_in_4_collections(inner)),
    )))))
}

// NB - can't use Hash stuff here because they can't nest
type FourDeepCollection<T> = BTreeMap<u8, BTreeMap<u8, BTreeSet<Vec<T>>>>;

fn wrap_in_4_collections<T: Eq + Ord + Eq>(inner: Option<T>) -> FourDeepCollection<T> {
    let inner = match inner {
        Some(inner) => vec![inner],
        None => vec![],
    };
    let mut inner2 = BTreeSet::new();
    inner2.insert(inner);
    let mut inner3 = BTreeMap::new();
    inner3.insert(1, inner2);
    let mut inner4 = BTreeMap::new();
    inner4.insert(2, inner3);
    inner4
}

fn wrap_in_hashmap<T>(inner: T) -> HashMap<u8, T> {
    let mut value = HashMap::new();
    value.insert(1, inner);
    value
}

fn wrap_in_hashset<T: Hash + Eq>(inner: T) -> HashSet<T> {
    let mut value = HashSet::new();
    value.insert(inner);
    value
}

fn wrap_in_tuple_single<T>(inner: T) -> (T,) {
    (inner,)
}
