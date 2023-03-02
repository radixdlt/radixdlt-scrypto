#![cfg_attr(not(feature = "std"), no_std)]

use sbor::*;

#[derive(Sbor, PartialEq, Eq, Debug)]
#[sbor(transparent)]
pub struct TestStructNamed {
    pub state: u32,
}

#[derive(Sbor, PartialEq, Eq, Debug)]
#[sbor(transparent)]
pub struct TestStructUnnamed(u32);

#[derive(Sbor, PartialEq, Eq, Debug)]
#[sbor(transparent)]
pub struct TestStruct<T> {
    #[sbor(skip)]
    pub abc: u32,
    pub state: T,
}

#[test]
fn categorize_is_correct() {
    // With inner u32
    assert_eq!(get_value_kind::<TestStructNamed>(), ValueKind::U32);
    assert_eq!(get_value_kind::<TestStructUnnamed>(), ValueKind::U32);
    assert_eq!(get_value_kind::<TestStruct::<u32>>(), ValueKind::U32);

    // And with inner tuple
    assert_eq!(get_value_kind::<TestStruct::<()>>(), ValueKind::Tuple);

    // With multiple layers of transparent
    assert_eq!(
        get_value_kind::<TestStruct::<TestStructNamed>>(),
        ValueKind::U32
    );
}

fn get_value_kind<T: Categorize<NoCustomValueKind>>() -> ValueKind<NoCustomValueKind> {
    T::value_kind()
}

#[test]
fn encode_is_correct() {
    // With inner u32
    let inner_value = 45u32;
    assert_eq!(
        basic_encode(&TestStructNamed { state: inner_value }).unwrap(),
        basic_encode(&inner_value).unwrap()
    );
    assert_eq!(
        basic_encode(&TestStructUnnamed(inner_value)).unwrap(),
        basic_encode(&inner_value).unwrap()
    );
    assert_eq!(
        basic_encode(&TestStruct::<u32> {
            state: inner_value,
            abc: 0
        })
        .unwrap(),
        basic_encode(&inner_value).unwrap()
    );

    // With inner tuple
    let inner_value = ();
    assert_eq!(
        basic_encode(&TestStruct::<()> {
            state: inner_value,
            abc: 0
        })
        .unwrap(),
        basic_encode(&()).unwrap()
    );

    // With multiple layers of transparent
    let inner_value = 45u32;
    assert_eq!(
        basic_encode(&TestStruct::<TestStructNamed> {
            state: TestStructNamed { state: inner_value },
            abc: 0
        })
        .unwrap(),
        basic_encode(&inner_value).unwrap()
    );
}

#[test]
fn decode_is_correct() {
    // With inner u32
    let inner_value = 45u32;
    let payload = basic_encode(&inner_value).unwrap();
    assert_eq!(
        basic_decode::<TestStructNamed>(&payload).unwrap(),
        TestStructNamed { state: inner_value }
    );
    assert_eq!(
        basic_decode::<TestStructUnnamed>(&payload).unwrap(),
        TestStructUnnamed(inner_value)
    );
    assert_eq!(
        basic_decode::<TestStruct::<u32>>(&payload).unwrap(),
        TestStruct::<u32> {
            state: inner_value,
            abc: Default::default()
        }
    );

    // With inner tuple
    let inner_value = ();
    let payload = basic_encode(&inner_value).unwrap();
    assert_eq!(
        basic_decode::<TestStruct::<()>>(&payload).unwrap(),
        TestStruct {
            state: inner_value,
            abc: Default::default()
        }
    );

    // With multiple layers of transparent
    let inner_value = 45u32;
    let payload = basic_encode(&inner_value).unwrap();
    assert_eq!(
        basic_decode::<TestStruct::<TestStructNamed>>(&payload).unwrap(),
        TestStruct {
            state: TestStructNamed { state: inner_value },
            abc: 0
        }
    );
}

#[test]
fn describe_is_correct() {
    // With inner u32
    check_identical_schemas::<TestStructNamed, u32>();
    check_identical_schemas::<TestStructUnnamed, u32>();
    check_identical_schemas::<TestStruct<u32>, u32>();

    // With inner tuple
    check_identical_schemas::<TestStruct<()>, ()>();

    // With multiple layers of transparent
    check_identical_schemas::<TestStruct<TestStructNamed>, u32>();
}

fn check_identical_schemas<T1: Describe<NoCustomTypeKind>, T2: Describe<NoCustomTypeKind>>() {
    let (type_index1, schema1) =
        generate_full_schema_from_single_type::<T1, NoCustomTypeExtension>();
    let (type_index2, schema2) =
        generate_full_schema_from_single_type::<T2, NoCustomTypeExtension>();
    assert_eq!(type_index1, type_index2);
    assert_eq!(schema1, schema2);
}
