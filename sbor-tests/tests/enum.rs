#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::prelude::*;
use sbor::*;

const VARIANT_1: u8 = 4;
const VARIANT_2: u8 = 5;

#[derive(Sbor, PartialEq, Eq, Debug)]
pub enum Abc {
    #[sbor(id(VARIANT_1))]
    Variant1,
    #[sbor(id(VARIANT_2))]
    Variant2,
}

const CONST_32: u8 = 32;
const CONST_55: u8 = 55;

#[derive(Sbor, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Mixed {
    #[sbor(id = 5)]
    A,
    #[sbor(id(7))]
    B,
    #[sbor(id("8"))]
    C {
        test: String,
    },
    D = 9,
    E = 11u8,
    F = CONST_32,
    #[sbor(discriminator(14))]
    G = 111,
    #[sbor(discriminator(CONST_55))]
    H = 14,
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
