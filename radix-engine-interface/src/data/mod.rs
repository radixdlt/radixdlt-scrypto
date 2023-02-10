/// Defines the custom Scrypto schema types.
mod custom_schema;
/// Defines the model of Scrypto custom values.
mod custom_value;
/// Defines the custom value kind model that scrypto uses.
mod custom_value_kind;
/// Indexed Scrypto value.
mod indexed_value;
/// Matches a Scrypto schema type with a Scrypto value.
mod schema_matcher;
/// Defines a way to uniquely identify an element within a Scrypto schema type.
mod schema_path;
/// Scrypto custom types
pub mod types;
/// Format any Scrypto value using the Manifest syntax.
mod value_formatter;
#[cfg(feature = "serde")]
/// One-way serialize any Scrypto value.
mod value_serializer;

pub use crate::args;

pub use custom_schema::*;
pub use custom_value::*;
pub use custom_value_kind::*;
pub use indexed_value::*;
use sbor::rust::vec::Vec;
use sbor::{
    Categorize, Decode, DecodeError, Decoder, Encode, EncodeError, Encoder, Value, ValueKind,
    VecDecoder, VecEncoder,
};
pub use schema_matcher::*;
pub use schema_path::*;
pub use value_formatter::*;
#[cfg(feature = "serde")]
pub use value_serializer::*;

pub const MAX_SCRYPTO_SBOR_DEPTH: u8 = 64;

pub type ScryptoEncoder<'a> = VecEncoder<'a, ScryptoCustomValueKind, MAX_SCRYPTO_SBOR_DEPTH>;
pub type ScryptoDecoder<'a> = VecDecoder<'a, ScryptoCustomValueKind, MAX_SCRYPTO_SBOR_DEPTH>;
pub type ScryptoValueKind = ValueKind<ScryptoCustomValueKind>;
pub type ScryptoValue = Value<ScryptoCustomValueKind, ScryptoCustomValue>;

// 0x5c for [5c]rypto - (91 in decimal)
pub const SCRYPTO_SBOR_V1_PAYLOAD_PREFIX: u8 = 0x5c;

// The following trait "aliases" are to be used in parameters.
//
// They are much nicer to read than the underlying traits, but because they are "new", and are defined
// via blanket impls, they can only be used for parameters, but cannot be used for implementations.
//
// Implementations should instead implement the underlying traits:
// * Categorize<ScryptoCustomValueKind>
// * Encode<ScryptoCustomValueKind, E> (impl over all E: Encoder<ScryptoCustomValueKind>)
// * Decode<ScryptoCustomValueKind, D> (impl over all D: Decoder<ScryptoCustomValueKind>)
//
// TODO: Change these to be Trait aliases once stable in rust: https://github.com/rust-lang/rust/issues/41517
pub trait ScryptoCategorize: Categorize<ScryptoCustomValueKind> {}
impl<T: Categorize<ScryptoCustomValueKind> + ?Sized> ScryptoCategorize for T {}

pub trait ScryptoDecode: for<'a> Decode<ScryptoCustomValueKind, ScryptoDecoder<'a>> {}
impl<T: for<'a> Decode<ScryptoCustomValueKind, ScryptoDecoder<'a>>> ScryptoDecode for T {}

pub trait ScryptoEncode: for<'a> Encode<ScryptoCustomValueKind, ScryptoEncoder<'a>> {}
impl<T: for<'a> Encode<ScryptoCustomValueKind, ScryptoEncoder<'a>> + ?Sized> ScryptoEncode for T {}

/// Encodes a data structure into byte array.
pub fn scrypto_encode<T: ScryptoEncode + ?Sized>(value: &T) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::with_capacity(512);
    let encoder = ScryptoEncoder::new(&mut buf);
    encoder.encode_payload(value, SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)?;
    Ok(buf)
}

/// Decodes a data structure from a byte array.
pub fn scrypto_decode<T: ScryptoDecode>(buf: &[u8]) -> Result<T, DecodeError> {
    ScryptoDecoder::new(buf).decode_payload(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
}

#[macro_export]
macro_rules! count {
    () => {0usize};
    ($a:expr) => {1usize};
    ($a:expr, $($rest:expr),*) => {1usize + radix_engine_interface::count!($($rest),*)};
}

/// Constructs argument list for Scrypto function/method invocation.
#[macro_export]
macro_rules! args {
    ($($args: expr),*) => {{
        use ::sbor::Encoder;
        let mut buf = ::sbor::rust::vec::Vec::new();
        let mut encoder = radix_engine_interface::data::ScryptoEncoder::new(&mut buf);
        encoder.write_payload_prefix(radix_engine_interface::data::SCRYPTO_SBOR_V1_PAYLOAD_PREFIX).unwrap();
        encoder.write_value_kind(radix_engine_interface::data::ScryptoValueKind::Tuple).unwrap();
        // Hack: stringify to skip ownership move semantics
        encoder.write_size(radix_engine_interface::count!($(stringify!($args)),*)).unwrap();
        $(
            let arg = $args;
            encoder.encode(&arg).unwrap();
        )*
        buf
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use crate::*;
    use sbor::rust::borrow::ToOwned;
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

    #[test]
    fn test_args() {
        #[derive(ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
        struct A {
            a: u32,
            b: String,
        }

        assert_eq!(
            args!(1u32, "abc"),
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
        let _x = args!(BTreeSet::from([id]));
    }

    #[test]
    fn test_encode_deep_scrypto_values() {
        // This test tests that the ScryptoValue Encode implementation correctly increments the depth

        // Test deep scrypto value vecs
        let valid_value = build_value_of_vec_of_depth(MAX_SCRYPTO_SBOR_DEPTH);
        assert!(scrypto_encode(&valid_value).is_ok());

        let invalid_value = build_value_of_vec_of_depth(MAX_SCRYPTO_SBOR_DEPTH + 1);
        assert_eq!(
            scrypto_encode(&invalid_value),
            Err(EncodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );

        // Test deep scrypto value tuples
        let valid_value = build_value_of_tuple_of_depth(MAX_SCRYPTO_SBOR_DEPTH);
        assert!(scrypto_encode(&valid_value).is_ok());

        let invalid_value = build_value_of_tuple_of_depth(MAX_SCRYPTO_SBOR_DEPTH + 1);
        assert_eq!(
            scrypto_encode(&invalid_value),
            Err(EncodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );
    }

    #[test]
    fn test_decode_deep_scrypto_values() {
        // This test tests that the ScryptoValue Decode implementation correctly increments the depth

        // Test deep scrypto value vecs
        let valid_payload =
            encode_ignore_depth(&build_value_of_vec_of_depth(MAX_SCRYPTO_SBOR_DEPTH));
        assert!(scrypto_decode::<ScryptoValue>(&valid_payload).is_ok());

        let invalid_payload =
            encode_ignore_depth(&build_value_of_vec_of_depth(MAX_SCRYPTO_SBOR_DEPTH + 1));
        assert_eq!(
            scrypto_decode::<ScryptoValue>(&invalid_payload),
            Err(DecodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );

        // Test deep scrypto value tuples
        let valid_payload =
            encode_ignore_depth(&build_value_of_tuple_of_depth(MAX_SCRYPTO_SBOR_DEPTH));
        assert!(scrypto_decode::<ScryptoValue>(&valid_payload).is_ok());

        let invalid_payload =
            encode_ignore_depth(&build_value_of_tuple_of_depth(MAX_SCRYPTO_SBOR_DEPTH + 1));
        assert_eq!(
            scrypto_decode::<ScryptoValue>(&invalid_payload),
            Err(DecodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
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
            Err(EncodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );
        let invalid_value_as_scrypto_value =
            decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&invalid_value));
        assert_eq!(
            scrypto_encode(&invalid_value_as_scrypto_value),
            Err(EncodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );

        // Test deep nested types
        let valid_value = build_nested_struct_of_depth(MAX_SCRYPTO_SBOR_DEPTH);
        assert!(scrypto_encode(&valid_value).is_ok());
        let valid_value_as_scrypto_value =
            decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&valid_value));
        assert!(scrypto_encode(&valid_value_as_scrypto_value).is_ok());

        let invalid_value = vec![build_nested_struct_of_depth(MAX_SCRYPTO_SBOR_DEPTH)];
        assert_eq!(
            scrypto_encode(&invalid_value),
            Err(EncodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );
        let invalid_value_as_scrypto_value =
            decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&invalid_value));
        assert_eq!(
            scrypto_encode(&invalid_value_as_scrypto_value),
            Err(EncodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );

        // Test hashmaps
        let valid_value = wrap_in_hashmap(wrap_in_hashmap(build_nested_struct_of_depth(
            MAX_SCRYPTO_SBOR_DEPTH - 2,
        )));
        assert!(scrypto_encode(&valid_value).is_ok());
        let valid_value_as_scrypto_value =
            decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&valid_value));
        assert!(scrypto_encode(&valid_value_as_scrypto_value).is_ok());
        let invalid_value = wrap_in_hashmap(wrap_in_hashmap(wrap_in_hashmap(
            build_nested_struct_of_depth(MAX_SCRYPTO_SBOR_DEPTH - 2),
        )));
        assert_eq!(
            scrypto_encode(&invalid_value),
            Err(EncodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );
        let invalid_value_as_scrypto_value =
            decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&invalid_value));
        assert_eq!(
            scrypto_encode(&invalid_value_as_scrypto_value),
            Err(EncodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
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
            Err(EncodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );
        let invalid_value_as_scrypto_value =
            decode_ignore_depth::<ScryptoValue>(&encode_ignore_depth(&invalid_value));
        assert_eq!(
            scrypto_encode(&invalid_value_as_scrypto_value),
            Err(EncodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
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
            Err(DecodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );
        assert_eq!(
            scrypto_decode::<ScryptoValue>(&invalid_payload),
            Err(DecodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );

        // Test deep nested types
        let valid_payload =
            encode_ignore_depth(&build_nested_struct_of_depth(MAX_SCRYPTO_SBOR_DEPTH));
        assert!(scrypto_decode::<NestedType>(&valid_payload).is_ok());
        assert!(scrypto_decode::<ScryptoValue>(&valid_payload).is_ok());

        let invalid_payload =
            encode_ignore_depth(&vec![build_nested_struct_of_depth(MAX_SCRYPTO_SBOR_DEPTH)]); // 65 deep
        assert!(matches!(
            scrypto_decode::<Vec<NestedType>>(&invalid_payload),
            Err(DecodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        ));
        assert_eq!(
            scrypto_decode::<ScryptoValue>(&invalid_payload),
            Err(DecodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );

        // Test hashmaps
        let valid_payload = encode_ignore_depth(&wrap_in_hashmap(wrap_in_hashmap(
            build_nested_struct_of_depth(MAX_SCRYPTO_SBOR_DEPTH - 2),
        )));
        assert!(scrypto_decode::<HashMap<u8, HashMap<u8, NestedType>>>(&valid_payload).is_ok());
        assert!(scrypto_decode::<ScryptoValue>(&valid_payload).is_ok());
        let invalid_payload = encode_ignore_depth(&wrap_in_hashmap(wrap_in_hashmap(
            wrap_in_hashmap(build_nested_struct_of_depth(MAX_SCRYPTO_SBOR_DEPTH - 2)),
        )));
        assert!(matches!(
            scrypto_decode::<HashMap<u8, HashMap<u8, HashMap<u8, NestedType>>>>(&invalid_payload),
            Err(DecodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        ));
        assert_eq!(
            scrypto_decode::<ScryptoValue>(&invalid_payload),
            Err(DecodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );

        // Test hashsets + tuples
        let valid_payload = encode_ignore_depth(&wrap_in_61_vecs(Some(wrap_in_tuple_single(
            wrap_in_hashset("hello"),
        ))));
        assert!(scrypto_decode::<SixtyOneDeepVec<(HashSet<String>,)>>(&valid_payload).is_ok());
        assert!(scrypto_decode::<ScryptoValue>(&valid_payload).is_ok());

        let invalid_payload = encode_ignore_depth(&vec![wrap_in_61_vecs(Some(
            wrap_in_tuple_single(wrap_in_hashset("hello")),
        ))]);
        assert_eq!(
            scrypto_decode::<Vec<SixtyOneDeepVec<(HashSet<String>,)>>>(&invalid_payload),
            Err(DecodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );
        assert_eq!(
            scrypto_decode::<ScryptoValue>(&invalid_payload),
            Err(DecodeError::MaxDepthExceeded(MAX_SCRYPTO_SBOR_DEPTH))
        );
    }

    fn encode_ignore_depth<
        V: for<'a> Encode<ScryptoCustomValueKind, VecEncoder<'a, ScryptoCustomValueKind, 255>>,
    >(
        value: &V,
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        let encoder = VecEncoder::<ScryptoCustomValueKind, 255>::new(&mut buf);
        encoder
            .encode_payload(value, SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap();
        buf
    }

    fn decode_ignore_depth<
        'a,
        T: Decode<ScryptoCustomValueKind, VecDecoder<'a, ScryptoCustomValueKind, 255>>,
    >(
        payload: &'a [u8],
    ) -> T {
        let decoder = VecDecoder::<ScryptoCustomValueKind, 255>::new(payload);
        decoder
            .decode_payload(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap()
    }

    fn build_value_of_vec_of_depth(depth: u8) -> ScryptoValue {
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

    fn build_value_of_tuple_of_depth(depth: u8) -> ScryptoValue {
        let mut value = ScryptoValue::Tuple { fields: vec![] };
        let loop_count = depth - 1;
        for _ in 0..loop_count {
            value = ScryptoValue::Tuple {
                fields: vec![value],
            };
        }
        value
    }

    #[derive(
        ScryptoCategorize,
        ScryptoEncode,
        ScryptoDecode,
        Debug,
        Clone,
        Eq,
        PartialEq,
        Ord,
        PartialOrd,
    )]
    struct NestedType {
        inner: Box<Rc<Option<RefCell<NestedType>>>>,
    }

    fn build_nested_struct_of_depth(depth: u8) -> NestedType {
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

    type SixtyOneDeepVec<T> = SixteenDeepVec<
        SixteenDeepVec<SixteenDeepVec<FourDeepVec<FourDeepVec<FourDeepVec<Vec<T>>>>>>,
    >;
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

    type SixtyFourDeepCollection<T> = SixteenDeepCollection<
        SixteenDeepCollection<SixteenDeepCollection<SixteenDeepCollection<T>>>,
    >;

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
}
