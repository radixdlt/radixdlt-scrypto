#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::borrow::Cow;
use sbor::rust::boxed::Box;
use sbor::rust::collections::*;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

#[derive(Categorize, Encode, Decode, Describe)]
pub struct UnitStruct;

#[derive(Categorize, Encode, Decode, Describe)]
pub struct BasicSample {
    pub a: (),
    pub b: UnitStruct,
}

#[derive(Categorize, Encode, Decode, Describe)]
#[sbor(generic_categorize_bounds = "S,T")]
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
    pub k: HashMap<[u8; 3], BTreeMap<i64, BTreeSet<i32>>>,
}

#[derive(Categorize, Encode, Decode, Describe)]
pub struct Recursive<T> {
    pub hello: Option<Box<Recursive<T>>>,
    pub what: T,
}

#[derive(Categorize, Encode, Decode, Describe)]
pub struct IndirectRecursive1(
    Vec<IndirectRecursive2<Recursive<u8>>>,
    Recursive<String>,
    Box<IndirectRecursiveEnum3>,
);

#[derive(Categorize, Encode, Decode, Describe)]
pub struct IndirectRecursive2<T>(Recursive<T>, IndirectRecursive1);

#[derive(Categorize, Encode, Decode, Describe)]
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
        generate_full_schema_from_single_type::<UnitStruct, NoCustomTypeExtension>(); // The original type should be the first type in the schema
    assert!(matches!(type_ref, LocalTypeIndex::SchemaLocalIndex(0)));
    assert_eq!(schema.type_kinds.len(), 1);
    assert_eq!(schema.type_metadata.len(), 1);
    assert_eq!(
        GlobalTypeId::Novel(schema.type_metadata[0].type_hash),
        <UnitStruct as Describe<NoCustomTypeKind>>::TYPE_ID
    );
    assert_eq!(
        schema.type_metadata[0].type_metadata.type_name,
        "UnitStruct"
    );
    assert!(matches!(
        &schema.type_metadata[0].type_metadata.child_names,
        ChildNames::None
    ));
}

#[test]
fn create_basic_sample_schema_works_correctly() {
    let (root_type_ref, schema) =
        generate_full_schema_from_single_type::<BasicSample, NoCustomTypeExtension>(); // The original type should be the first type in the schema

    assert!(matches!(root_type_ref, LocalTypeIndex::SchemaLocalIndex(0)));
    assert_eq!(schema.type_kinds.len(), 2);
    assert_eq!(schema.type_metadata.len(), 2);

    // Test Root Type

    let type_data = schema.resolve(LocalTypeIndex::SchemaLocalIndex(0)).unwrap();
    assert_eq!(type_data.metadata.type_name, "BasicSample");
    assert!(
        matches!(&type_data.metadata.child_names, ChildNames::FieldNames(field_names) if matches!(field_names[..], [
            Cow::Borrowed("a"),
            Cow::Borrowed("b"),
        ]))
    );
    assert!(
        matches!(type_data.kind.into_owned(), TypeKind::Tuple { field_types } if matches!(field_types[..], [
            LocalTypeIndex::WellKnown(basic_well_known_types::UNIT_ID),
            LocalTypeIndex::SchemaLocalIndex(1),
        ]))
    );

    // Test Further Types

    let type_data = schema.resolve(LocalTypeIndex::SchemaLocalIndex(1)).unwrap();
    assert_eq!(type_data.metadata.type_name, "UnitStruct");
    assert!(matches!(type_data.metadata.child_names, ChildNames::None));
    assert!(
        matches!(type_data.kind.into_owned(), TypeKind::Tuple { field_types } if matches!(field_types[..], []))
    );
}

#[test]
fn create_advanced_sample_schema_works_correctly() {
    let (type_ref, schema) = generate_full_schema_from_single_type::<
        AdvancedSample<UnitStruct, u128>,
        NoCustomTypeExtension,
    >();

    // The original type should be the first type in the schema
    assert!(matches!(type_ref, LocalTypeIndex::SchemaLocalIndex(0)));

    // We then check each type in turn is what we expect

    let type_data = schema.resolve(LocalTypeIndex::SchemaLocalIndex(0)).unwrap();
    assert_eq!(type_data.metadata.type_name, "AdvancedSample");
    assert!(
        matches!(&type_data.metadata.child_names, ChildNames::FieldNames(field_names) if matches!(field_names[..], [
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
    let TypeKind::Tuple { field_types } = type_data.kind.into_owned() else {
        panic!("Type was not a Tuple");
    };
    assert!(matches!(
        field_types[..],
        [
            LocalTypeIndex::WellKnown(basic_well_known_types::UNIT_ID),
            LocalTypeIndex::WellKnown(basic_well_known_types::U32_ID),
            LocalTypeIndex::SchemaLocalIndex(1), // Registers (u8, Vec<T>) which also registers SchemaLocal(2) as Vec<T>
            LocalTypeIndex::WellKnown(basic_well_known_types::STRING_ID),
            LocalTypeIndex::WellKnown(basic_well_known_types::U128_ID),
            LocalTypeIndex::WellKnown(basic_well_known_types::U128_ID), // S resolves to U128
            LocalTypeIndex::SchemaLocalIndex(3),                        // T resolves to UnitStruct
            LocalTypeIndex::WellKnown(basic_well_known_types::BYTES_ID),
            LocalTypeIndex::SchemaLocalIndex(4), // Vec<S> = Vec<u128>, a non-well-known type
            LocalTypeIndex::SchemaLocalIndex(3), // T resolves to UnitStruct - at the same schema index as before
            LocalTypeIndex::SchemaLocalIndex(5), // HashMap<[u8; 3], BTreeMap<i64, BTreeSet<i32>>>
        ]
    ));
}

#[test]
fn creating_schema_from_multiple_types_works_correctly() {
    let mut aggregator = TypeAggregator::<NoCustomTypeKind>::new();
    let unit_struct_type_ref = aggregator.add_child_type_and_descendents::<UnitStruct>();
    let advanced_sample_type_ref =
        aggregator.add_child_type_and_descendents::<AdvancedSample<UnitStruct, u128>>();
    let i64_type_ref = aggregator.add_child_type_and_descendents::<i64>();
    let unit_struct_type_ref_2 = aggregator.add_child_type_and_descendents::<UnitStruct>();

    // Check when adding a type that's already known, we return the existing index
    assert!(matches!(
        unit_struct_type_ref,
        LocalTypeIndex::SchemaLocalIndex(0)
    ));
    assert!(matches!(
        advanced_sample_type_ref,
        LocalTypeIndex::SchemaLocalIndex(1)
    ));
    assert!(matches!(
        i64_type_ref,
        LocalTypeIndex::WellKnown(basic_well_known_types::I64_ID)
    ));
    assert!(matches!(
        unit_struct_type_ref_2,
        LocalTypeIndex::SchemaLocalIndex(0)
    )); // Repeats the first one

    let schema = generate_full_schema(aggregator);

    // Check that the AdvancedSample references UnitStruct at the correct index
    let type_data = schema.resolve(advanced_sample_type_ref).unwrap();
    let TypeKind::Tuple { field_types } = type_data.kind.into_owned() else {
        panic!("Type was not a Tuple");
    };
    assert_eq!(field_types[6], unit_struct_type_ref); // T = UnitStruct is the 7th field in AdvancedSample<UnitStruct, u128>
}

#[test]
fn create_recursive_schema_works_correctly() {
    // Most of this test is checking that such recursive schemas can: (A) happily compile and (B) don't panic when a schema is generated
    let (type_ref, schema) =
        generate_full_schema_from_single_type::<IndirectRecursive1, NoCustomTypeExtension>();

    // The original type should be the first type in the schema
    assert!(matches!(type_ref, LocalTypeIndex::SchemaLocalIndex(0)));

    let type_data = schema.resolve(LocalTypeIndex::SchemaLocalIndex(0)).unwrap();
    assert_eq!(type_data.metadata.type_name, "IndirectRecursive1");
}
