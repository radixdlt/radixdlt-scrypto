#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::prelude::*;
use sbor::*;

#[derive(Sbor, PartialEq, Eq, Debug)]
#[sbor(
    as_type = "u32",
    as_ref = "&self.state",
    from_value = "Self { state: value }"
)]
pub struct TestStructNamed {
    pub state: u32,
}

#[derive(Sbor, PartialEq, Eq, Debug)]
#[sbor(
    as_type = "u32",
    as_ref = "&self.state",
    from_value = "Self { state: value }"
)]
#[sbor(transparent_name)]
pub struct TestStructTransparentNamed {
    pub state: u32,
}

#[derive(Sbor, PartialEq, Eq, Debug)]
#[sbor(
    as_type = "u32",
    as_ref = "&self.state",
    from_value = "Self { state: value }"
)]
#[sbor(type_name = "HEY")]
pub struct TestStructRenamed {
    pub state: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
#[sbor(as_type = "GenericModelVersions < T >")]
struct VersionedGenericModel<T> {
    inner: Option<GenericModelVersions<T>>,
}

impl<T> VersionedGenericModel<T> {
    pub fn new(inner: GenericModelVersions<T>) -> Self {
        Self { inner: Some(inner) }
    }
}

impl<T> AsRef<GenericModelVersions<T>> for VersionedGenericModel<T> {
    fn as_ref(&self) -> &GenericModelVersions<T> {
        self.inner.as_ref().unwrap()
    }
}
impl<T> From<GenericModelVersions<T>> for VersionedGenericModel<T> {
    fn from(value: GenericModelVersions<T>) -> Self {
        Self::new(value)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
enum GenericModelVersions<T> {
    V1(T),
}

#[test]
fn categorize_is_correct() {
    // With inner u32
    assert_eq!(get_value_kind::<TestStructNamed>(), ValueKind::U32);
    assert_eq!(
        get_value_kind::<VersionedGenericModel<u32>>(),
        ValueKind::Enum
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
    // With generic enum using AsRef
    assert_eq!(
        basic_encode(&VersionedGenericModel::new(GenericModelVersions::V1(45u32))).unwrap(),
        basic_encode(&GenericModelVersions::V1(45u32)).unwrap()
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
    // With generic enum using AsRef
    let payload = basic_encode(&GenericModelVersions::V1(45u32)).unwrap();
    assert_eq!(
        basic_decode::<VersionedGenericModel::<u32>>(&payload).unwrap(),
        VersionedGenericModel::new(GenericModelVersions::V1(45u32))
    );
}

#[test]
fn describe_is_correct() {
    check_identical_types::<TestStructNamed, u32>(Some("TestStructNamed"));
    check_identical_types::<TestStructTransparentNamed, u32>(None);
    check_identical_types::<TestStructRenamed, u32>(Some("HEY"));
    check_identical_types::<VersionedGenericModel<u64>, GenericModelVersions<u64>>(Some(
        "VersionedGenericModel",
    ));
}

fn check_identical_types<T1: Describe<NoCustomTypeKind>, T2: Describe<NoCustomTypeKind>>(
    name: Option<&'static str>,
) {
    let (type_id1, schema1) = generate_full_schema_from_single_type::<T1, NoCustomSchema>();
    let (type_id2, schema2) = generate_full_schema_from_single_type::<T2, NoCustomSchema>();

    assert_eq!(
        schema1.v1().resolve_type_kind(type_id1),
        schema2.v1().resolve_type_kind(type_id2)
    );
    assert_eq!(
        schema1
            .v1()
            .resolve_type_metadata(type_id1)
            .unwrap()
            .clone(),
        schema2
            .v1()
            .resolve_type_metadata(type_id2)
            .unwrap()
            .clone()
            .with_name(name.map(|name| Cow::Borrowed(name)))
    );
    assert_eq!(
        schema1.v1().resolve_type_validation(type_id1),
        schema2.v1().resolve_type_validation(type_id2)
    );
}
