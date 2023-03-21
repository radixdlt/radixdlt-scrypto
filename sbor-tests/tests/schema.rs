#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::borrow::Cow;
use sbor::rust::boxed::Box;
use sbor::rust::collections::*;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

#[derive(Sbor)]
pub struct UnitStruct;

#[derive(Sbor)]
pub struct BasicSample {
    pub a: (),
    pub b: UnitStruct,
}

#[derive(Sbor)]
#[sbor(categorize_types = "S, T")]
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

#[derive(Sbor)]
pub struct Recursive<T> {
    pub hello: Option<Box<Recursive<T>>>,
    pub what: T,
}

#[derive(Sbor)]
pub struct IndirectRecursive1(
    Vec<IndirectRecursive2<Recursive<u8>>>,
    Recursive<String>,
    Box<IndirectRecursiveEnum3>,
);

#[derive(Sbor)]
pub struct IndirectRecursive2<T>(Recursive<T>, IndirectRecursive1);

#[derive(Sbor)]
pub enum IndirectRecursiveEnum3 {
    Variant1,
    Variant2(Box<IndirectRecursive1>),
    Variant3 {
        x: Box<IndirectRecursive2<IndirectRecursive1>>,
    },
}

#[test]
fn create_unit_struct_schema_works_correctly() {
    let (type_index, schema) =
        generate_full_schema_from_single_type::<UnitStruct, NoCustomTypeExtension>(); // The original type should be the first type in the schema
    assert!(matches!(type_index, LocalTypeIndex::SchemaLocalIndex(0)));
    assert_eq!(schema.type_kinds.len(), 1);
    assert_eq!(schema.type_metadata.len(), 1);
    assert_eq!(schema.type_metadata[0].get_name().unwrap(), "UnitStruct");
    assert!(matches!(&schema.type_metadata[0].child_names, None));
    assert!(schema.validate().is_ok());
}

#[test]
fn create_basic_sample_schema_works_correctly() {
    let (root_type_index, schema) =
        generate_full_schema_from_single_type::<BasicSample, NoCustomTypeExtension>(); // The original type should be the first type in the schema

    assert!(matches!(
        root_type_index,
        LocalTypeIndex::SchemaLocalIndex(0)
    ));
    assert_eq!(schema.type_kinds.len(), 2);
    assert_eq!(schema.type_metadata.len(), 2);

    // Test Root Type
    let kind = schema
        .resolve_type_kind(LocalTypeIndex::SchemaLocalIndex(0))
        .unwrap();
    let metadata = schema
        .resolve_type_metadata(LocalTypeIndex::SchemaLocalIndex(0))
        .unwrap();
    assert_eq!(metadata.get_name().unwrap(), "BasicSample");
    assert!(
        matches!(&metadata.child_names, Some(ChildNames::NamedFields(field_names)) if matches!(field_names[..], [
            Cow::Borrowed("a") ,
            Cow::Borrowed("b")
        ]))
    );
    assert!(
        matches!(kind, TypeKind::Tuple { field_types } if matches!(field_types[..], [
            LocalTypeIndex::WellKnown(basic_well_known_types::UNIT_ID),
            LocalTypeIndex::SchemaLocalIndex(1),
        ]))
    );

    // Test Further Types
    let kind = schema
        .resolve_type_kind(LocalTypeIndex::SchemaLocalIndex(1))
        .unwrap();
    let metadata = schema
        .resolve_type_metadata(LocalTypeIndex::SchemaLocalIndex(1))
        .unwrap();
    assert_eq!(metadata.get_name().unwrap(), "UnitStruct");
    assert!(matches!(metadata.child_names, None));
    assert!(matches!(kind, TypeKind::Tuple { field_types } if matches!(field_types[..], [])));
    assert!(schema.validate().is_ok());
}

#[test]
fn create_advanced_sample_schema_works_correctly() {
    let (type_index, schema) = generate_full_schema_from_single_type::<
        AdvancedSample<UnitStruct, u128>,
        NoCustomTypeExtension,
    >();

    // The original type should be the first type in the schema
    assert!(matches!(type_index, LocalTypeIndex::SchemaLocalIndex(0)));

    // We then check each type in turn is what we expect
    let kind = schema
        .resolve_type_kind(LocalTypeIndex::SchemaLocalIndex(0))
        .unwrap();
    let metadata = schema
        .resolve_type_metadata(LocalTypeIndex::SchemaLocalIndex(0))
        .unwrap();
    assert_eq!(metadata.get_name().unwrap(), "AdvancedSample");
    assert!(
        matches!(&metadata.child_names, Some(ChildNames::NamedFields(field_names)) if matches!(field_names[..], [
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
            Cow::Borrowed("k")
        ]))
    );
    let TypeKind::Tuple { field_types } =  kind else {
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
    assert!(schema.validate().is_ok());
}

#[test]
fn creating_schema_from_multiple_types_works_correctly() {
    let mut aggregator = TypeAggregator::<NoCustomTypeKind>::new();
    let unit_struct_type_index = aggregator.add_child_type_and_descendents::<UnitStruct>();
    let advanced_sample_type_index =
        aggregator.add_child_type_and_descendents::<AdvancedSample<UnitStruct, u128>>();
    let i64_type_index = aggregator.add_child_type_and_descendents::<i64>();
    let unit_struct_type_index_2 = aggregator.add_child_type_and_descendents::<UnitStruct>();

    // Check when adding a type that's already known, we return the existing index
    assert!(matches!(
        unit_struct_type_index,
        LocalTypeIndex::SchemaLocalIndex(0)
    ));
    assert!(matches!(
        advanced_sample_type_index,
        LocalTypeIndex::SchemaLocalIndex(1)
    ));
    assert!(matches!(
        i64_type_index,
        LocalTypeIndex::WellKnown(basic_well_known_types::I64_ID)
    ));
    assert!(matches!(
        unit_struct_type_index_2,
        LocalTypeIndex::SchemaLocalIndex(0)
    )); // Repeats the first one

    let schema = generate_full_schema(aggregator);

    // Check that the AdvancedSample references UnitStruct at the correct index
    let kind = schema
        .resolve_type_kind(advanced_sample_type_index)
        .unwrap();
    let TypeKind::Tuple { field_types } =  kind else {
        panic!("Type was not a Tuple");
    };
    assert_eq!(field_types[6], unit_struct_type_index); // T = UnitStruct is the 7th field in AdvancedSample<UnitStruct, u128>
    assert!(schema.validate().is_ok());
}

#[test]
fn create_recursive_schema_works_correctly() {
    // Most of this test is checking that such recursive schemas can: (A) happily compile and (B) don't panic when a schema is generated
    let (type_index, schema) =
        generate_full_schema_from_single_type::<IndirectRecursive1, NoCustomTypeExtension>();

    // The original type should be the first type in the schema
    assert!(matches!(type_index, LocalTypeIndex::SchemaLocalIndex(0)));
    let metadata = schema
        .resolve_type_metadata(LocalTypeIndex::SchemaLocalIndex(0))
        .unwrap();
    assert_eq!(metadata.get_name().unwrap(), "IndirectRecursive1");
    assert!(schema.validate().is_ok());
}
