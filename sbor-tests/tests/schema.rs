#![cfg_attr(not(feature = "std"), no_std)]

use sbor::prelude::*;
use sbor::*;

#[derive(Sbor)]
pub struct UnitStruct;

#[derive(Sbor)]
#[sbor(type_name = "UnitStructRenamed2")]
pub struct UnitStructRenamed;

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
    let (type_id, schema) = generate_full_schema_from_single_type::<UnitStruct, NoCustomSchema>(); // The original type should be the first type in the schema
    assert_matches!(type_id, LocalTypeId::SchemaLocalIndex(0));
    assert_eq!(schema.v1().type_kinds.len(), 1);
    assert_eq!(schema.v1().type_metadata.len(), 1);
    assert_eq!(
        schema.v1().type_metadata[0].get_name().unwrap(),
        "UnitStruct"
    );
    assert_matches!(&schema.v1().type_metadata[0].child_names, None);
    assert!(schema.v1().validate().is_ok());
}

#[test]
fn create_basic_sample_schema_works_correctly() {
    let (root_type_id, schema) =
        generate_full_schema_from_single_type::<BasicSample, NoCustomSchema>(); // The original type should be the first type in the schema

    assert_matches!(root_type_id, LocalTypeId::SchemaLocalIndex(0));
    assert_eq!(schema.v1().type_kinds.len(), 2);
    assert_eq!(schema.v1().type_metadata.len(), 2);

    // Test Root Type
    let kind = schema
        .v1()
        .resolve_type_kind(LocalTypeId::SchemaLocalIndex(0))
        .unwrap();
    let metadata = schema
        .v1()
        .resolve_type_metadata(LocalTypeId::SchemaLocalIndex(0))
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
            LocalTypeId::WellKnown(basic_well_known_types::UNIT_TYPE),
            LocalTypeId::SchemaLocalIndex(1),
        ]))
    );

    // Test Further Types
    let kind = schema
        .v1()
        .resolve_type_kind(LocalTypeId::SchemaLocalIndex(1))
        .unwrap();
    let metadata = schema
        .v1()
        .resolve_type_metadata(LocalTypeId::SchemaLocalIndex(1))
        .unwrap();
    assert_eq!(metadata.get_name().unwrap(), "UnitStruct");
    assert_matches!(metadata.child_names, None);
    assert_matches!(kind, TypeKind::Tuple { field_types } if matches!(field_types[..], []));
    assert!(schema.v1().validate().is_ok());
}

#[test]
fn create_advanced_sample_schema_works_correctly() {
    let (type_id, schema) =
        generate_full_schema_from_single_type::<AdvancedSample<UnitStruct, u128>, NoCustomSchema>();

    // The original type should be the first type in the schema
    assert_matches!(type_id, LocalTypeId::SchemaLocalIndex(0));

    // We then check each type in turn is what we expect
    let kind = schema
        .v1()
        .resolve_type_kind(LocalTypeId::SchemaLocalIndex(0))
        .unwrap();
    let metadata = schema
        .v1()
        .resolve_type_metadata(LocalTypeId::SchemaLocalIndex(0))
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
    let TypeKind::Tuple { field_types } = kind else {
        panic!("Type was not a Tuple");
    };
    assert_matches!(
        field_types[..],
        [
            LocalTypeId::WellKnown(basic_well_known_types::UNIT_TYPE),
            LocalTypeId::WellKnown(basic_well_known_types::U32_TYPE),
            LocalTypeId::SchemaLocalIndex(1), // Registers (u8, Vec<T>) which also registers SchemaLocal(2) as Vec<T>
            LocalTypeId::WellKnown(basic_well_known_types::STRING_TYPE),
            LocalTypeId::WellKnown(basic_well_known_types::U128_TYPE),
            LocalTypeId::WellKnown(basic_well_known_types::U128_TYPE), // S resolves to U128
            LocalTypeId::SchemaLocalIndex(3),                          // T resolves to UnitStruct
            LocalTypeId::WellKnown(basic_well_known_types::BYTES_TYPE),
            LocalTypeId::SchemaLocalIndex(4), // Vec<S> = Vec<u128>, a non-well-known type
            LocalTypeId::SchemaLocalIndex(3), // T resolves to UnitStruct - at the same schema index as before
            LocalTypeId::SchemaLocalIndex(5), // HashMap<[u8; 3], BTreeMap<i64, BTreeSet<i32>>>
        ]
    );
    assert!(schema.v1().validate().is_ok());
}

#[test]
fn creating_schema_from_multiple_types_works_correctly() {
    let mut aggregator = TypeAggregator::<NoCustomTypeKind>::new();
    let unit_struct_type_id = aggregator.add_child_type_and_descendents::<UnitStruct>();
    let advanced_sample_type_id =
        aggregator.add_child_type_and_descendents::<AdvancedSample<UnitStruct, u128>>();
    let i64_type_id = aggregator.add_child_type_and_descendents::<i64>();
    let unit_struct_type_id_2 = aggregator.add_child_type_and_descendents::<UnitStruct>();

    // Check when adding a type that's already known, we return the existing index
    assert_matches!(unit_struct_type_id, LocalTypeId::SchemaLocalIndex(0));
    assert_matches!(advanced_sample_type_id, LocalTypeId::SchemaLocalIndex(1));
    assert_matches!(
        i64_type_id,
        LocalTypeId::WellKnown(basic_well_known_types::I64_TYPE)
    );
    assert_matches!(unit_struct_type_id_2, LocalTypeId::SchemaLocalIndex(0)); // Repeats the first one

    let schema = generate_full_schema::<NoCustomSchema>(aggregator);

    // Check that the AdvancedSample references UnitStruct at the correct index
    let kind = schema
        .v1()
        .resolve_type_kind(advanced_sample_type_id)
        .unwrap();
    let TypeKind::Tuple { field_types } = kind else {
        panic!("Type was not a Tuple");
    };
    assert_eq!(field_types[6], unit_struct_type_id); // T = UnitStruct is the 7th field in AdvancedSample<UnitStruct, u128>
    assert!(schema.v1().validate().is_ok());
}

#[test]
fn create_recursive_schema_works_correctly() {
    // Most of this test is checking that such recursive schemas can: (A) happily compile and (B) don't panic when a schema is generated
    let (type_id, schema) =
        generate_full_schema_from_single_type::<IndirectRecursive1, NoCustomSchema>();

    // The original type should be the first type in the schema
    assert_matches!(type_id, LocalTypeId::SchemaLocalIndex(0));
    let metadata = schema
        .v1()
        .resolve_type_metadata(LocalTypeId::SchemaLocalIndex(0))
        .unwrap();
    assert_eq!(metadata.get_name().unwrap(), "IndirectRecursive1");
    assert!(schema.v1().validate().is_ok());
}

#[test]
fn test_type_name_works_correctly() {
    // Most of this test is checking that such recursive schemas can: (A) happily compile and (B) don't panic when a schema is generated
    let (type_id, schema) =
        generate_full_schema_from_single_type::<UnitStructRenamed, NoCustomSchema>();

    // The original type should be the first type in the schema
    assert_matches!(type_id, LocalTypeId::SchemaLocalIndex(0));
    let metadata = schema.v1().resolve_type_metadata(type_id).unwrap();
    assert_eq!(metadata.get_name().unwrap(), "UnitStructRenamed2");
    assert!(schema.v1().validate().is_ok());
}
