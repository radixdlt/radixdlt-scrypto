#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::prelude::*;
use sbor::*;

const VARIANT_1: u8 = 4;
const VARIANT_2: u8 = 5;

#[derive(Sbor, PartialEq, Eq, Debug)]
pub enum Abc {
    #[sbor(discriminator(VARIANT_1))]
    Variant1,
    #[sbor(discriminator(VARIANT_2))]
    Variant2,
}

const CONST_55_U8: u8 = 55;
const CONST_32_U32: u32 = 32;
const CONST_32_U8: u8 = 32;

/// This enum demonstrates the following:
/// * `#[sbor(use_repr_discriminators)]` works and provides the default value
/// * `#[sbor(discriminator(X)))]` overrides the repr if both are provided
/// * The Sbor macro is flexible at picking up different discriminators - including:
///   - Binary
///   - Non-u8 integer literals
///   - U8 constants (nb - doesn't support non-u8 constants - need to override with `#[sbor(discriminator(X)))])`
///
/// You can also play about with the errors if you set up duplicate discriminators in different modes.
///
/// The combination of correct treatment of spans with `#[deny(unreachable_patterns)]` on the decode implementation
/// means that duplicate values (even for constants) results in a compile error, flagged at the duplicated value.
#[derive(Sbor, PartialEq, Eq, Debug)]
#[repr(u32)]
#[sbor(use_repr_discriminators)]
pub enum Mixed {
    #[sbor(discriminator = 5)]
    A,
    #[sbor(discriminator(7))]
    B,
    #[sbor(discriminator("8"))]
    C {
        test: String,
    },
    D = 9,
    E = 11u32,
    #[sbor(discriminator(CONST_32_U8))]
    F = CONST_32_U32,
    #[sbor(discriminator(14))]
    G = 111,
    #[sbor(discriminator(CONST_55_U8))]
    H = 14,
    I = 0b11011,
}

#[test]
fn can_encode_and_decode() {
    check_encode_decode_schema(&Abc::Variant1);
    check_encode_decode_schema(&Abc::Variant2);
    check_encode_decode_schema(&Mixed::A);
    check_encode_decode_schema(&Mixed::B);
    check_encode_decode_schema(&Mixed::C {
        test: "hello".to_string(),
    });
    check_encode_decode_schema(&Mixed::D);
    check_encode_decode_schema(&Mixed::E);
    check_encode_decode_schema(&Mixed::F);
    check_encode_decode_schema(&Mixed::G);
    check_encode_decode_schema(&Mixed::H);
    check_encode_decode_schema(&Mixed::I);

    check_encode_identically(
        &Mixed::C {
            test: "hello".to_string(),
        },
        &BasicValue::Enum {
            discriminator: 8,
            fields: vec![BasicValue::String {
                value: "hello".to_string(),
            }],
        },
    );
    check_encode_identically(
        &Mixed::G,
        &BasicValue::Enum {
            discriminator: 14,
            fields: vec![],
        },
    )
}

fn check_encode_decode_schema<T: BasicEncode + BasicDecode + BasicDescribe + Eq + Debug>(
    value: &T,
) {
    assert_eq!(
        &basic_decode::<T>(&basic_encode(value).unwrap()).unwrap(),
        value
    );

    let (type_index, schema) = generate_full_schema_from_single_type::<T, NoCustomSchema>();
    validate_payload_against_schema::<NoCustomExtension, ()>(
        &basic_encode(value).unwrap(),
        &schema,
        type_index,
        &(),
    )
    .unwrap();
}

fn check_encode_identically<T1: BasicEncode, T2: BasicEncode>(value1: &T1, value2: &T2) {
    assert_eq!(basic_encode(value1).unwrap(), basic_encode(value2).unwrap());
}
