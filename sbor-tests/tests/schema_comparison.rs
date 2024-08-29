use sbor::prelude::*;
use sbor::schema::*;
use sbor::{BasicTypeAggregator, BasicValue, NoCustomSchema, NoCustomTypeKind};

//=====================
// HELPER CODE / TRAITS
//=====================
trait DerivableTypeSchema: Describe<NoCustomTypeKind> {
    fn single_type_schema_version() -> SingleTypeSchema<NoCustomSchema> {
        SingleTypeSchema::for_type::<Self>()
    }
}

impl<T: Describe<NoCustomTypeKind>> DerivableTypeSchema for T {}

fn assert_equality<T1: DerivableTypeSchema, T2: DerivableTypeSchema>() {
    let settings = SchemaComparisonSettings::require_equality();
    compare_single_type_schemas::<NoCustomSchema>(
        &settings,
        &T1::single_type_schema_version(),
        &T2::single_type_schema_version(),
    )
    .assert_valid("base", "compared")
}

fn assert_equality_ignoring_name_changes<T1: DerivableTypeSchema, T2: DerivableTypeSchema>() {
    let settings = SchemaComparisonSettings::require_equality().allow_all_name_changes();
    compare_single_type_schemas::<NoCustomSchema>(
        &settings,
        &T1::single_type_schema_version(),
        &T2::single_type_schema_version(),
    )
    .assert_valid("base", "compared")
}

fn assert_extension<T1: DerivableTypeSchema, T2: DerivableTypeSchema>() {
    let settings = SchemaComparisonSettings::allow_extension();
    compare_single_type_schemas::<NoCustomSchema>(
        &settings,
        &T1::single_type_schema_version(),
        &T2::single_type_schema_version(),
    )
    .assert_valid("base", "compared")
}

fn assert_extension_ignoring_name_changes<T1: DerivableTypeSchema, T2: DerivableTypeSchema>() {
    let settings = SchemaComparisonSettings::allow_extension().allow_all_name_changes();
    compare_single_type_schemas::<NoCustomSchema>(
        &settings,
        &T1::single_type_schema_version(),
        &T2::single_type_schema_version(),
    )
    .assert_valid("base", "compared")
}

fn assert_type_collection_equality(
    base: TypeCollectionSchema<NoCustomSchema>,
    compared: TypeCollectionSchema<NoCustomSchema>,
) {
    compare_type_collection_schemas(
        &SchemaComparisonSettings::require_equality(),
        &base,
        &compared,
    )
    .assert_valid("base", "compared");
}

fn assert_type_collection_extension(
    base: TypeCollectionSchema<NoCustomSchema>,
    compared: TypeCollectionSchema<NoCustomSchema>,
) {
    compare_type_collection_schemas(
        &SchemaComparisonSettings::allow_extension(),
        &base,
        &compared,
    )
    .assert_valid("base", "compared");
}

//=============
// HELPER TYPES
//=============

#[derive(Sbor)]
#[sbor(type_name = "MyStruct")]
struct MyStruct {
    val: u8,
}

#[derive(Sbor)]
#[sbor(type_name = "MyStruct")]
struct MyStructFieldRenamed {
    val_renamed: u8,
}

#[derive(Sbor)]
#[sbor(type_name = "MyStructTypeRenamed")]
struct MyStructTypeRenamed {
    val: u8,
}

#[derive(Sbor)]
#[sbor(type_name = "MyStruct")]
struct MyStructNewField {
    val: u8,
    field_2: u8,
}

type MyNamefreeNestedTuple = (u8, (u16, u16));
type NamelessAny = BasicValue;

#[derive(BasicDescribe)]
#[sbor(transparent)]
pub struct MyAny(pub BasicValue);

#[derive(Sbor)]
#[sbor(type_name = "MyEnum")]
enum MyEnum {
    Variant1,
    Variant2,
    Variant3(u8, u16),
    Variant4 { my_val: i32, my_struct: MyStruct },
}

#[derive(Sbor)]
#[sbor(type_name = "MyEnum")]
enum MyEnumVariantRenamed {
    Variant1,
    Variant2V2,
    Variant3(u8, u16),
    Variant4 { my_val: i32, my_struct: MyStruct },
}

#[derive(Sbor)]
#[sbor(type_name = "MyEnum")]
enum MyEnumVariantFieldRenamed {
    Variant1,
    Variant2,
    Variant3(u8, u16),
    Variant4 {
        my_val_renamed: i32,
        my_struct: MyStruct,
    },
}

#[derive(Sbor)]
#[sbor(type_name = "MyEnum")]
enum MyEnumNewVariant {
    Variant1,
    Variant2,
    Variant3(u8, u16),
    Variant4 { my_val: i32, my_struct: MyStruct },
    Variant5,
}

#[derive(Sbor)]
#[sbor(type_name = "MyEnum")]
enum MyEnumVariantFieldAdded {
    Variant1,
    Variant2,
    Variant3(u8, u16, u32),
    Variant4 { my_val: i32, my_struct: MyStruct },
}

#[derive(BasicDescribe)]
pub struct MyTupleOf<T1, T2, T3>(pub T1, pub T2, pub T3);

// Form1 and Form2 are equivalent types (ignoring naming of the first field)
// But to verify equivalency of A = Form1 and B = Form2 it has to verify:
// * Via the Opposite variant: (A.Form2, B.Form1),
// * Via the Me variant: (A.Form1, B.Form2)
// * Via the Form1 variant: (A.Form1, B.Form1)
// * If a Form2 variant existed, (A.Form2, B.Form2)
// This demonstrates that verifying equivalency of schemas is O(N^2).
#[derive(Sbor)]
enum MyMultiRecursiveTypeForm1 {
    Nothing,
    Opposite(Option<Box<MyMultiRecursiveTypeForm2>>),
    Me(Option<Box<MyMultiRecursiveTypeForm1>>),
    Form1(Option<Box<MyMultiRecursiveTypeForm1>>),
}

#[derive(Sbor)]
enum MyMultiRecursiveTypeForm2 {
    None,
    Opposite(Option<Box<MyMultiRecursiveTypeForm1>>),
    Me(Option<Box<MyMultiRecursiveTypeForm2>>),
    Form1(Option<Box<MyMultiRecursiveTypeForm1>>),
}

//============
// BASIC TESTS
//============
#[test]
#[should_panic]
fn asserting_backwards_compatibility_requires_a_named_schema() {
    assert_type_backwards_compatibility::<NoCustomSchema, MyStructFieldRenamed>(|v| v)
}

#[test]
fn asserting_backwards_compatibility_with_a_single_latest_schema_version_succeeds() {
    assert_type_backwards_compatibility::<NoCustomSchema, MyStruct>(|v| {
        v.register_version("latest", MyStruct::single_type_schema_version())
    })
}

#[test]
#[should_panic]
fn asserting_backwards_compatibility_with_incorrect_latest_schema_version_succeeds() {
    assert_type_backwards_compatibility::<NoCustomSchema, MyStruct>(|v| {
        v.register_version("latest", MyStructFieldRenamed::single_type_schema_version())
    })
}

#[test]
fn asserting_backwards_compatibility_with_two_identical_schema_versions_succeeds() {
    assert_extension::<MyStruct, MyStruct>();
}

#[test]
fn recursive_types_work() {
    assert_extension::<MyMultiRecursiveTypeForm1, MyMultiRecursiveTypeForm1>();
    assert_extension::<MyMultiRecursiveTypeForm2, MyMultiRecursiveTypeForm2>();
    // Note that, ignoring names, A and B are equivalent types, so this should work!
    assert_equality_ignoring_name_changes::<MyMultiRecursiveTypeForm1, MyMultiRecursiveTypeForm2>();
    assert_equality_ignoring_name_changes::<MyMultiRecursiveTypeForm2, MyMultiRecursiveTypeForm1>();
}

#[test]
fn generic_types_work() {
    assert_equality_ignoring_name_changes::<
        MyTupleOf<u8, MyStruct, MyMultiRecursiveTypeForm1>,
        (u8, MyStruct, MyMultiRecursiveTypeForm2),
    >();
}

//========================================
// TYPE INCOMPATIBILITY - STRUCTURAL TESTS
//========================================
#[test]
#[should_panic(expected = "TypeKindMismatch")]
fn changing_type_fails() {
    assert_equality::<MyStruct, MyEnum>();
}

#[test]
#[should_panic(expected = "TupleFieldCountMismatch")]
fn adding_tuple_field_fails() {
    assert_equality::<MyStruct, MyStructNewField>();
}

#[test]
#[should_panic(expected = "EnumVariantFieldCountMismatch")]
fn adding_enum_variant_field_fails() {
    assert_equality::<MyEnum, MyEnumVariantFieldAdded>();
}

#[test]
fn adding_variant_succeeds() {
    assert_extension::<MyEnum, MyEnumNewVariant>();
}

#[test]
#[should_panic(expected = "EnumSupportedVariantsMismatch")]
fn adding_variant_fails_if_equality_is_required() {
    assert_equality::<MyEnum, MyEnumNewVariant>();
}

#[test]
#[should_panic(expected = "TypeKindMismatch")]
fn internal_type_change_fails() {
    assert_equality_ignoring_name_changes::<
        MyTupleOf<u8, u16, MyMultiRecursiveTypeForm1>,
        (u8, MyStruct, MyMultiRecursiveTypeForm2),
    >();
}

#[test]
fn replacing_with_any_succeeds() {
    // Note that extension requires that names are preserved,
    // so using just assert_extension in the first case would fail
    assert_extension_ignoring_name_changes::<MyEnum, MyAny>();
    // But if the base type has no names, it is fine to replace with a nameless Any
    assert_extension::<MyNamefreeNestedTuple, NamelessAny>();
}

//========================================
// TYPE INCOMPATIBILITY - METADATA TESTS
//========================================
#[test]
#[should_panic(expected = "FieldNameChangeError")]
fn updating_struct_field_name_fails() {
    assert_extension::<MyStruct, MyStructFieldRenamed>();
}

#[test]
#[should_panic(expected = "TypeNameChangeError")]
fn updating_type_name_fails() {
    assert_extension::<MyStruct, MyStructTypeRenamed>();
}

#[test]
#[should_panic(expected = "EnumVariantNameChangeError")]
fn updating_variant_name_fails() {
    assert_extension::<MyEnum, MyEnumVariantRenamed>();
}

#[test]
#[should_panic(expected = "EnumVariantFieldNameChangeError")]
fn updating_variant_field_name_fails() {
    assert_extension::<MyEnum, MyEnumVariantFieldRenamed>();
}

#[test]
fn all_name_changes_allowed_succeeds() {
    assert_equality_ignoring_name_changes::<MyStruct, MyStructFieldRenamed>();
    assert_equality_ignoring_name_changes::<MyStruct, MyStructTypeRenamed>();
    assert_equality_ignoring_name_changes::<MyEnum, MyEnumVariantRenamed>();
    assert_equality_ignoring_name_changes::<MyEnum, MyEnumVariantFieldRenamed>();
    assert_equality_ignoring_name_changes::<
        (MyStruct, MyStruct, MyEnum, MyEnum),
        (
            MyStructFieldRenamed,
            MyStructTypeRenamed,
            MyEnumVariantRenamed,
            MyEnumVariantFieldRenamed,
        ),
    >();
}

//========================================
// TYPE INCOMPATIBILITY - VALIDATION TESTS
//========================================
#[test]
#[should_panic(expected = "TypeValidationChangeError")]
fn changing_length_fails() {
    assert_extension::<[u8; 5], [u8; 10]>();
}

#[test]
fn extension_removing_length_validation_succeeds() {
    assert_extension::<[u8; 5], Vec<u8>>();
}

#[test]
#[should_panic(expected = "TypeValidationChangeError")]
fn equality_removing_length_validation_fails() {
    assert_equality::<[u8; 5], Vec<u8>>();
}

//===================================
// INCOMPLETENESS TESTS
//===================================

#[test]
#[should_panic(expected = "TypeUnreachableFromRootInBaseSchema")]
fn base_schema_not_covered_by_root_types_fails() {
    let mut base_schema = {
        let mut aggregator = BasicTypeAggregator::new();
        aggregator.add_root_type::<MyStruct>("struct");
        aggregator.add_root_type::<MyEnum>("enum");
        aggregator.generate_type_collection_schema()
    };
    let compared_schema = {
        let mut aggregator = BasicTypeAggregator::new();
        aggregator.add_root_type::<MyStruct>("struct");
        aggregator.generate_type_collection_schema()
    };
    // Forget about a root type - this leaves the schema not fully covered.
    base_schema.type_ids.swap_remove("enum");

    assert_type_collection_equality(base_schema, compared_schema);
}

#[test]
#[should_panic(expected = "TypeUnreachableFromRootInComparedSchema")]
fn compared_schema_not_covered_by_root_types_fails() {
    let base_schema = {
        let mut aggregator = BasicTypeAggregator::new();
        aggregator.add_root_type::<MyStruct>("struct");
        aggregator.generate_type_collection_schema()
    };
    let mut compared_schema = {
        let mut aggregator = BasicTypeAggregator::new();
        aggregator.add_root_type::<MyStruct>("struct");
        aggregator.add_root_type::<MyEnum>("enum");
        aggregator.generate_type_collection_schema()
    };
    // Forget about a root type - this leaves the schema not fully covered.
    compared_schema.type_ids.swap_remove("enum");

    // Note - the process starts by comparing "latest" with "current"
    // (i.e. compared_schema against itself) - so we get both a
    // TypeUnreachableFromRootInBaseSchema and a TypeUnreachableFromRootInComparedSchema
    assert_type_collection_equality(base_schema, compared_schema);
}

#[test]
#[should_panic(expected = "NamedRootTypeMissingInComparedSchema")]
fn removed_root_type_fails_comparison() {
    let base_schema = {
        let mut aggregator = BasicTypeAggregator::new();
        aggregator.add_root_type::<MyStruct>("struct");
        aggregator.add_root_type::<MyEnum>("enum");
        aggregator.generate_type_collection_schema()
    };
    let compared_schema = {
        let mut aggregator = BasicTypeAggregator::new();
        aggregator.add_root_type::<MyStruct>("struct");
        aggregator.generate_type_collection_schema()
    };

    assert_type_collection_extension(base_schema, compared_schema);
}

#[test]
fn under_extension_added_root_type_succeeds() {
    let base_schema = {
        let mut aggregator = BasicTypeAggregator::new();
        aggregator.add_root_type::<MyStruct>("struct");
        aggregator.generate_type_collection_schema()
    };
    let compared_schema = {
        let mut aggregator = BasicTypeAggregator::new();
        aggregator.add_root_type::<MyStruct>("struct");
        aggregator.add_root_type::<MyEnum>("enum");
        aggregator.generate_type_collection_schema()
    };

    assert_type_collection_extension(base_schema, compared_schema);
}

#[test]
#[should_panic(expected = "DisallowedNewRootTypeInComparedSchema")]
fn under_equality_added_root_type_fails() {
    let base_schema = {
        let mut aggregator = BasicTypeAggregator::new();
        aggregator.add_root_type::<MyStruct>("struct");
        aggregator.generate_type_collection_schema()
    };
    let compared_schema = {
        let mut aggregator = BasicTypeAggregator::new();
        aggregator.add_root_type::<MyStruct>("struct");
        aggregator.add_root_type::<MyEnum>("enum");
        aggregator.generate_type_collection_schema()
    };

    assert_type_collection_equality(base_schema, compared_schema);
}
