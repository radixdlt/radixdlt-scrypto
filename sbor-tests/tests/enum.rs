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

#[derive(Sbor, PartialEq, Eq, Debug)]
#[sbor(type_name = "Abc")]
pub enum AbcV2 {
    #[sbor(discriminator(VARIANT_1))]
    Variant1,
    #[sbor(unreachable)]
    UnreachableVariant,
    #[sbor(discriminator(VARIANT_2))]
    Variant2,
}

#[derive(PermitSborAttributes, PartialEq, Eq, Debug)]
#[sbor(type_name = "Abc")]
pub enum TestThatPermitSborAttributesCanCompile {
    #[sbor(discriminator(VARIANT_1))]
    Variant1,
    #[sbor(unreachable)]
    UnreachableVariant,
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

#[derive(Debug, PartialEq, Eq, Sbor, Clone)]
enum FlattenEnum {
    #[sbor(impl_variant_trait)]
    A {
        #[sbor(skip)]
        skipped: u8,
        #[sbor(flatten)]
        y: (u32, MyOtherType),
    },
    B(#[sbor(skip)] u8, #[sbor(flatten)] (u32,)),
    #[sbor(impl_variant_trait)]
    C(#[sbor(flatten)] MyInnerStruct),
    D,
    #[sbor(impl_variant_trait)]
    E(MyOtherTypeTwo),
}

#[derive(Debug, PartialEq, Eq, Sbor, Clone)]
struct MyInnerStruct {
    hello: String,
    world: MyInnerInnerType, // This checks that we properly capture descendents in Describe
}

#[derive(Debug, PartialEq, Eq, Sbor, Clone)]
struct MyOtherType(u8);

#[derive(Debug, PartialEq, Eq, Sbor, Clone)]
struct MyInnerInnerType(u8);

#[derive(Debug, PartialEq, Eq, Sbor, Clone)]
struct MyOtherTypeTwo(u8);

#[derive(Debug, PartialEq, Eq, Sbor, Clone)]
#[sbor(type_name = "FlattenEnum")]
enum FlattenedEnum {
    A(u32, MyOtherType),
    B(u32),
    C {
        hello: String,
        world: MyInnerInnerType,
    },
    D,
    E(MyOtherTypeTwo),
}

#[derive(Debug, PartialEq, Eq, Categorize, Clone)]
#[sbor(impl_variant_traits)]
enum OnlyCategorizeAndImplVariantTraits {
    A(OnlyCategorizeInnerType),
}

#[derive(Debug, PartialEq, Eq, Categorize, Clone)]
struct OnlyCategorizeInnerType(u8);

#[test]
fn test_encode_decode_and_schemas() {
    check_encode_decode_schema(&Abc::Variant1);
    check_encode_decode_schema(&Abc::Variant2);
    check_encode_decode_schema(&AbcV2::Variant1);
    check_encode_decode_schema(&AbcV2::Variant2);
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
    check_encode_decode_schema(&FlattenEnum::A {
        skipped: 0,
        y: (1, MyOtherType(5)),
    });
    check_encode_identically(
        &FlattenEnum::A {
            skipped: 0,
            y: (1, MyOtherType(5)),
        },
        &FlattenedEnum::A(1, MyOtherType(5)),
    );
    check_encode_decode_schema(&FlattenEnum::B(0, (7,)));
    check_encode_identically(&FlattenEnum::B(0, (7,)), &FlattenedEnum::B(7));
    check_encode_decode_schema(&FlattenEnum::C(MyInnerStruct {
        hello: "howdy".to_string(),
        world: MyInnerInnerType(13),
    }));
    check_encode_identically(
        &FlattenEnum::C(MyInnerStruct {
            hello: "howdy".to_string(),
            world: MyInnerInnerType(13),
        }),
        &FlattenedEnum::C {
            hello: "howdy".to_string(),
            world: MyInnerInnerType(13),
        },
    );
    check_encode_decode_schema(&FlattenEnum::D);
    check_encode_identically(&FlattenEnum::D, &FlattenedEnum::D);
    check_encode_decode_schema(&FlattenEnum::E(MyOtherTypeTwo(7)));
    check_encode_identically(
        &FlattenEnum::E(MyOtherTypeTwo(7)),
        &FlattenedEnum::E(MyOtherTypeTwo(7)),
    );

    check_schema_equality::<Abc, AbcV2>();
    check_schema_equality::<FlattenEnum, FlattenedEnum>();

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
    );
    check_encode_identically(&Abc::Variant1, &AbcV2::Variant1);
    check_encode_identically(&Abc::Variant2, &AbcV2::Variant2);
}

#[test]
fn test_impl_variant_trait() {
    // First test flattened variant
    assert_sbor_enum_variant_equivalency(
        MyInnerStruct {
            hello: "hi".to_string(),
            world: MyInnerInnerType(31),
        },
        FlattenEnum::C(MyInnerStruct {
            hello: "hi".to_string(),
            world: MyInnerInnerType(31),
        }),
    );
    // Then test unflattened variant
    assert_sbor_enum_variant_equivalency(MyOtherTypeTwo(3), FlattenEnum::E(MyOtherTypeTwo(3)));

    assert_eq!(
        <OnlyCategorizeInnerType as SborEnumVariantFor<
            OnlyCategorizeAndImplVariantTraits,
            NoCustomValueKind,
        >>::into_enum(OnlyCategorizeInnerType(1)),
        OnlyCategorizeAndImplVariantTraits::A(OnlyCategorizeInnerType(1)),
    );
}

fn assert_sbor_enum_variant_equivalency<
    TEnum: SborEnum<NoCustomValueKind> + BasicSbor + Eq + Debug + Clone,
    TInner: SborEnumVariantFor<TEnum, NoCustomValueKind> + BasicSbor + Eq + Debug + Clone,
>(
    inner_value: TInner,
    value_in_enum: TEnum,
) where
    for<'a> TInner::BorrowedVariant<'a>: BasicEncode,
    for<'a> TInner::OwnedVariant: BasicDecode,
{
    let encoded_inner_value_as_variant = basic_encode(&inner_value.as_encodable_variant()).unwrap();
    let encoded_enum_value = basic_encode(&value_in_enum).unwrap();
    assert_eq!(
        encoded_inner_value_as_variant, encoded_enum_value,
        "Encodable variant should encoded identically to the value in the enum"
    );
    assert_eq!(
        inner_value.clone().into_enum(),
        value_in_enum.clone(),
        "Converting the inner value into_enum should give the value in the enum"
    );
    assert_eq!(
        TInner::from_decoded_variant(basic_decode(&encoded_enum_value).unwrap()),
        inner_value,
        "Decoding into the inner type from an encoded enum payload should work"
    );
}

#[test]
#[should_panic]
fn test_encoding_unreachable_variant_panics() {
    let _ignored = basic_encode(&AbcV2::UnreachableVariant);
}

fn check_encode_decode_schema<T: BasicEncode + BasicDecode + BasicDescribe + Eq + Debug>(
    value: &T,
) {
    assert_eq!(
        &basic_decode::<T>(&basic_encode(value).unwrap()).unwrap(),
        value
    );

    let (type_id, schema) = generate_full_schema_from_single_type::<T, NoCustomSchema>();
    validate_payload_against_schema::<NoCustomExtension, ()>(
        &basic_encode(value).unwrap(),
        schema.v1(),
        type_id,
        &(),
        64,
    )
    .unwrap();
}

fn check_encode_identically<T1: BasicEncode, T2: BasicEncode>(value1: &T1, value2: &T2) {
    assert_eq!(basic_encode(value1).unwrap(), basic_encode(value2).unwrap());
}

fn check_schema_equality<T1: BasicDescribe, T2: BasicDescribe>() {
    assert_eq!(
        generate_full_schema_from_single_type::<T1, NoCustomSchema>(),
        generate_full_schema_from_single_type::<T2, NoCustomSchema>(),
    );
}
