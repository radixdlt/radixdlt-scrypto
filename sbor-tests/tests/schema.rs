#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::borrow::Cow;
use sbor::rust::boxed::Box;
use sbor::rust::collections::{BTreeSet, HashMap, IndexMap};
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

#[derive(TypeId, Encode, Decode, Schema)]
pub struct UnitStruct;

#[derive(TypeId, Encode, Decode, Schema)]
pub struct BasicSample {
    pub a: (),
    pub b: UnitStruct,
}

#[derive(TypeId, Encode, Decode, Schema)]
#[sbor(generic_type_id_bounds = "S,T")]
pub struct AdvancedSample<T, S> {
    pub a: (),
    pub b: u32,
    pub c: (u8, Vec<T>),
    pub d: String,
    pub e: u128,
    pub f: S,
    pub g: T,
    pub h: Vec<u8>,
    pub i: Vec<S>,
    pub j: T,
    pub k: HashMap<[u8; 3], IndexMap<i64, BTreeSet<i32>>>,
}

#[derive(TypeId, Encode, Decode, Schema)]
pub struct Recursive<T> {
    pub hello: Option<Box<Recursive<T>>>,
    pub what: T,
}

#[derive(TypeId, Encode, Decode, Schema)]
pub struct IndirectRecursive1(
    Vec<IndirectRecursive2<Recursive<u8>>>,
    Recursive<String>,
    Box<IndirectRecursiveEnum3>,
);

#[derive(TypeId, Encode, Decode, Schema)]
pub struct IndirectRecursive2<T>(Recursive<T>, IndirectRecursive1);

#[derive(TypeId, Encode, Decode, Schema)]
pub enum IndirectRecursiveEnum3 {
    Variant1,
    Variant2(Box<IndirectRecursive1>),
    Variant3 {
        x: Box<IndirectRecursive2<IndirectRecursive1>>,
    },
}

#[test]
fn create_unit_struct_schema_works_correctly() {
    let (type_ref, schema) =
        generate_full_schema_from_single_type::<UnitStruct, NoCustomTypeSchema>(); // The original type should be the first type in the schema
    assert!(matches!(type_ref, SchemaLocalTypeRef::SchemaLocal(0)));
    assert_eq!(schema.custom_types.len(), 1);
    assert_eq!(schema.naming.len(), 1);
    assert_eq!(schema.naming[0].type_name, "UnitStruct");
    assert!(matches!(&schema.naming[0].child_names, ChildNames::None));
}

#[test]
fn create_basic_sample_schema_works_correctly() {
    let (root_type_ref, schema) =
        generate_full_schema_from_single_type::<BasicSample, NoCustomTypeSchema>(); // The original type should be the first type in the schema

    assert!(matches!(root_type_ref, SchemaLocalTypeRef::SchemaLocal(0)));
    assert_eq!(schema.custom_types.len(), 2);
    assert_eq!(schema.naming.len(), 2);

    // Test Root Type

    let type_data = schema.resolve(SchemaLocalTypeRef::SchemaLocal(0)).unwrap();
    assert_eq!(type_data.naming.type_name, "BasicSample");
    assert!(
        matches!(&type_data.naming.child_names, ChildNames::FieldNames(field_names) if matches!(field_names[..], [
            Cow::Borrowed("a"),
            Cow::Borrowed("b"),
        ]))
    );
    assert!(
        matches!(type_data.schema.into_owned(), TypeSchema::Tuple { field_types } if matches!(field_types[..], [
            SchemaLocalTypeRef::WellKnown(well_known_basic_schemas::UNIT_INDEX),
            SchemaLocalTypeRef::SchemaLocal(1),
        ]))
    );

    // Test Further Types

    let type_data = schema.resolve(SchemaLocalTypeRef::SchemaLocal(1)).unwrap();
    assert_eq!(type_data.naming.type_name, "UnitStruct");
    assert!(matches!(type_data.naming.child_names, ChildNames::None));
    assert!(matches!(type_data.schema.into_owned(), TypeSchema::Unit));
}

#[test]
fn create_advanced_sample_schema_works_correctly() {
    let (type_ref, schema) = generate_full_schema_from_single_type::<
        AdvancedSample<UnitStruct, u128>,
        NoCustomTypeSchema,
    >();

    // The original type should be the first type in the schema
    assert!(matches!(type_ref, SchemaLocalTypeRef::SchemaLocal(0)));

    // We then check each type in turn is what we expect

    let type_data = schema.resolve(SchemaLocalTypeRef::SchemaLocal(0)).unwrap();
    assert_eq!(type_data.naming.type_name, "AdvancedSample");
    assert!(
        matches!(&type_data.naming.child_names, ChildNames::FieldNames(field_names) if matches!(field_names[..], [
            Cow::Borrowed("a"),
            Cow::Borrowed("b"),
            Cow::Borrowed("c"),
            Cow::Borrowed("d"),
            Cow::Borrowed("e"),
            Cow::Borrowed("f"),
            Cow::Borrowed("g"),
            Cow::Borrowed("h"),
            Cow::Borrowed("i"),
            Cow::Borrowed("j"),
            Cow::Borrowed("k"),
        ]))
    );
    let TypeSchema::Tuple { field_types } = type_data.schema.into_owned() else {
        panic!("Type was not a Tuple");
    };
    assert!(matches!(
        field_types[..],
        [
            SchemaLocalTypeRef::WellKnown(well_known_basic_schemas::UNIT_INDEX),
            SchemaLocalTypeRef::WellKnown(well_known_basic_schemas::U32_INDEX),
            SchemaLocalTypeRef::SchemaLocal(1), // Registers (u8, Vec<T>) which also registers SchemaLocal(2) as Vec<T>
            SchemaLocalTypeRef::WellKnown(well_known_basic_schemas::STRING_INDEX),
            SchemaLocalTypeRef::WellKnown(well_known_basic_schemas::U128_INDEX),
            SchemaLocalTypeRef::WellKnown(well_known_basic_schemas::U128_INDEX), // S resolves to U128
            SchemaLocalTypeRef::SchemaLocal(3), // T resolves to UnitStruct
            SchemaLocalTypeRef::WellKnown(well_known_basic_schemas::BYTES_INDEX),
            SchemaLocalTypeRef::SchemaLocal(4), // Vec<S> = Vec<u128>, a non-well-known type
            SchemaLocalTypeRef::SchemaLocal(3), // T resolves to UnitStruct - at the same schema index as before
            SchemaLocalTypeRef::SchemaLocal(5), // HashMap<[u8; 3], IndexMap<i64, BTreeSet<i32>>>
        ]
    ));
}

#[test]
fn creating_schema_from_multiple_types_works_correctly() {
    let mut aggregator = SchemaAggregator::<NoCustomTypeSchema>::new();
    let unit_struct_type_ref = aggregator.add_child_type_and_descendents::<UnitStruct>();
    let advanced_sample_type_ref =
        aggregator.add_child_type_and_descendents::<AdvancedSample<UnitStruct, u128>>();
    let i64_type_ref = aggregator.add_child_type_and_descendents::<i64>();
    let unit_struct_type_ref_2 = aggregator.add_child_type_and_descendents::<UnitStruct>();

    // Check when adding a type that's already known, we return the existing index
    assert!(matches!(
        unit_struct_type_ref,
        SchemaLocalTypeRef::SchemaLocal(0)
    ));
    assert!(matches!(
        advanced_sample_type_ref,
        SchemaLocalTypeRef::SchemaLocal(1)
    ));
    assert!(matches!(
        i64_type_ref,
        SchemaLocalTypeRef::WellKnown(well_known_basic_schemas::I64_INDEX)
    ));
    assert!(matches!(
        unit_struct_type_ref_2,
        SchemaLocalTypeRef::SchemaLocal(0)
    )); // Repeats the first one

    let schema = generate_full_schema(aggregator);

    // Check that the AdvancedSample references UnitStruct at the correct index
    let type_data = schema.resolve(advanced_sample_type_ref).unwrap();
    let TypeSchema::Tuple { field_types } = type_data.schema.into_owned() else {
        panic!("Type was not a Tuple");
    };
    assert_eq!(field_types[6], unit_struct_type_ref); // T = UnitStruct is the 7th field in AdvancedSample<UnitStruct, u128>
}

#[test]
fn create_recursive_schema_works_correctly() {
    // Most of this test is checking that such recursive schemas can: (A) happily compile and (B) don't panic when a schema is generated
    let (type_ref, schema) =
        generate_full_schema_from_single_type::<IndirectRecursive1, NoCustomTypeSchema>();

    // The original type should be the first type in the schema
    assert!(matches!(type_ref, SchemaLocalTypeRef::SchemaLocal(0)));

    let type_data = schema.resolve(SchemaLocalTypeRef::SchemaLocal(0)).unwrap();
    assert_eq!(type_data.naming.type_name, "IndirectRecursive1");
}
