use crate::rust::prelude::*;
use crate::traversal::*;
use crate::typed_traversal::*;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadValidationError<E: CustomTypeExtension> {
    TraversalError(TypedTraversalError<E::CustomValueKind>),
    TypeValidationError(TypeValidationError),
}

impl<E: CustomTypeExtension> From<TypeValidationError> for PayloadValidationError<E> {
    fn from(value: TypeValidationError) -> Self {
        Self::TypeValidationError(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeValidationError {
    LengthValidationError {
        required: LengthValidation,
        actual: usize,
    },
    I8ValidationError {
        required: NumericValidation<i8>,
        actual: i8,
    },
    I16ValidationError {
        required: NumericValidation<i16>,
        actual: i16,
    },
    I32ValidationError {
        required: NumericValidation<i32>,
        actual: i32,
    },
    I64ValidationError {
        required: NumericValidation<i64>,
        actual: i64,
    },
    I128ValidationError {
        required: NumericValidation<i128>,
        actual: i128,
    },
    U8ValidationError {
        required: NumericValidation<u8>,
        actual: u8,
    },
    U16ValidationError {
        required: NumericValidation<u16>,
        actual: u16,
    },
    U32ValidationError {
        required: NumericValidation<u32>,
        actual: u32,
    },
    U64ValidationError {
        required: NumericValidation<u64>,
        actual: u64,
    },
    U128ValidationError {
        required: NumericValidation<u128>,
        actual: u128,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedValidationError<'s, E: CustomTypeExtension> {
    error: PayloadValidationError<E>,
    location: FullLocation<'s, E::CustomTraversal>,
}

impl<'s, E: CustomTypeExtension> LocatedValidationError<'s, E> {
    pub fn error_message(&self, schema: &Schema<E>) -> String {
        format!(
            "{:?} occurred at byte offset {}-{} and value path {}",
            self.error,
            self.location.start_offset,
            self.location.end_offset,
            self.location.path_to_string(schema)
        )
    }
}

// TODO: need better representation for validated scheme to eliminate panic below

#[macro_export]
macro_rules! failed_to_resolve_type_validation {
    () => {
        panic!("Failed to resolve type validation, possibly caused by a bug in schema or payload validation!")
    };
}

#[macro_export]
macro_rules! type_validation_meets_unexpected_value {
    () => {
        panic!("Type validation meets unexpected value, possibly caused by a bug in schema or payload validation!")
    };
}

#[macro_export]
macro_rules! numeric_validation_match {
    ($numeric_validation: ident, $value: expr, $type: ident, $error_type: ident) => {{
        {
            let TerminalValueRef::$type(value) = *$value else { type_validation_meets_unexpected_value!() };
            if !$numeric_validation.is_valid(value) {
                return Err(TypeValidationError::$error_type {
                    required: *$numeric_validation,
                    actual: value,
                }.into());
            }
        }
    }};
}

pub fn validate_payload_against_schema<'s, E: CustomTypeExtension>(
    payload: &[u8],
    schema: &'s Schema<E>,
    index: LocalTypeIndex,
) -> Result<(), LocatedValidationError<'s, E>> {
    let mut traverser = traverse_payload_with_types::<E>(payload, &schema, index);
    loop {
        let typed_event = traverser.next_event();
        if validate_event_with_type::<E>(&schema, &typed_event.event).map_err(|error| {
            LocatedValidationError {
                error,
                location: typed_event.full_location(),
            }
        })? {
            return Ok(());
        }
    }
}

fn validate_event_with_type<E: CustomTypeExtension>(
    schema: &Schema<E>,
    event: &TypedTraversalEvent<E::CustomTraversal>,
) -> Result<bool, PayloadValidationError<E>> {
    match event {
        TypedTraversalEvent::PayloadPrefix => Ok(false),
        TypedTraversalEvent::ContainerStart(type_index, header) => {
            validate_container::<E>(schema, header, *type_index).map(|_| false)
        }
        TypedTraversalEvent::ContainerEnd(_, _) => Ok(false), // Validation already handled at Container Start
        TypedTraversalEvent::TerminalValue(type_index, value_ref) => {
            validate_terminal_value::<E>(schema, value_ref, *type_index).map(|_| false)
        }
        TypedTraversalEvent::TerminalValueBatch(type_index, value_batch_ref) => {
            validate_terminal_value_batch::<E>(schema, value_batch_ref, *type_index).map(|_| false)
        }
        TypedTraversalEvent::End => Ok(true),
        TypedTraversalEvent::Error(error) => {
            Err(PayloadValidationError::TraversalError(error.clone()))
        }
    }
}

pub fn validate_container<E: CustomTypeExtension>(
    schema: &Schema<E>,
    header: &ContainerHeader<E::CustomTraversal>,
    type_index: LocalTypeIndex,
) -> Result<(), PayloadValidationError<E>> {
    match schema
        .resolve_type_validation(type_index)
        .unwrap_or_else(|| failed_to_resolve_type_validation!())
    {
        TypeValidation::None => {}
        TypeValidation::Array(length_validation) => {
            let ContainerHeader::Array(ArrayHeader { length, .. }) = header else {
                type_validation_meets_unexpected_value!()
            };
            if !length_validation.is_valid(*length) {
                return Err(TypeValidationError::LengthValidationError {
                    required: *length_validation,
                    actual: *length,
                }
                .into());
            }
        }
        TypeValidation::Map(length_validation) => {
            let ContainerHeader::Map(MapHeader { length, .. }) = header else {
                type_validation_meets_unexpected_value!()
            };
            if !length_validation.is_valid(*length) {
                return Err(TypeValidationError::LengthValidationError {
                    required: *length_validation,
                    actual: *length,
                }
                .into());
            }
        }
        _ => type_validation_meets_unexpected_value!(),
    }
    Ok(())
}

pub fn validate_terminal_value<'de, E: CustomTypeExtension>(
    schema: &Schema<E>,
    value: &TerminalValueRef<'de, E::CustomTraversal>,
    type_index: LocalTypeIndex,
) -> Result<(), PayloadValidationError<E>> {
    match schema
        .resolve_type_validation(type_index)
        .unwrap_or_else(|| failed_to_resolve_type_validation!())
    {
        TypeValidation::None => {}
        TypeValidation::I8(x) => {
            numeric_validation_match!(x, value, I8, I8ValidationError)
        }
        TypeValidation::I16(x) => {
            numeric_validation_match!(x, value, I16, I16ValidationError)
        }
        TypeValidation::I32(x) => {
            numeric_validation_match!(x, value, I32, I32ValidationError)
        }
        TypeValidation::I64(x) => {
            numeric_validation_match!(x, value, I64, I64ValidationError)
        }
        TypeValidation::I128(x) => {
            numeric_validation_match!(x, value, I128, I128ValidationError)
        }
        TypeValidation::U8(x) => {
            numeric_validation_match!(x, value, U8, U8ValidationError)
        }
        TypeValidation::U16(x) => {
            numeric_validation_match!(x, value, U16, U16ValidationError)
        }
        TypeValidation::U32(x) => {
            numeric_validation_match!(x, value, U32, U32ValidationError)
        }
        TypeValidation::U64(x) => {
            numeric_validation_match!(x, value, U64, U64ValidationError)
        }
        TypeValidation::U128(x) => {
            numeric_validation_match!(x, value, U128, U128ValidationError)
        }
        TypeValidation::String(length_validation) => {
            let TerminalValueRef::String(x) = value else {
                type_validation_meets_unexpected_value!()
            };
            if !length_validation.is_valid(x.len()) {
                return Err(TypeValidationError::LengthValidationError {
                    required: *length_validation,
                    actual: x.len(),
                }
                .into());
            }
        }
        TypeValidation::Custom(_) => {
            todo!("Add support for custom type validation")
        }
        _ => type_validation_meets_unexpected_value!(),
    }
    Ok(())
}

pub fn validate_terminal_value_batch<'de, E: CustomTypeExtension>(
    schema: &Schema<E>,
    value_batch: &TerminalValueBatchRef<'de>,
    type_index: LocalTypeIndex,
) -> Result<(), PayloadValidationError<E>> {
    match schema
        .resolve_type_validation(type_index)
        .unwrap_or_else(|| failed_to_resolve_type_validation!())
    {
        TypeValidation::None => {}
        TypeValidation::U8(numeric_validation) => {
            // This is for `Vec<u8<min, max>>`
            let TerminalValueBatchRef::U8(value_batch) = value_batch;
            for byte in value_batch.iter() {
                if !numeric_validation.is_valid(*byte) {
                    return Err(TypeValidationError::U8ValidationError {
                        required: *numeric_validation,
                        actual: *byte,
                    }
                    .into());
                }
            }
        }
        _ => type_validation_meets_unexpected_value!(),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::LocatedValidationError;
    use crate::{rust::prelude::*, *};

    #[derive(Sbor)]
    struct TestStructArray {
        x: [u8; 16],
    }

    #[derive(Sbor)]
    struct TestStructVec {
        x: Vec<u8>,
    }

    #[test]
    pub fn identical_length_vec_and_array_are_interchangeable() {
        let (type_index, schema) =
            generate_full_schema_from_single_type::<TestStructArray, NoCustomTypeExtension>();
        let payload = basic_encode(&TestStructVec {
            x: Vec::from([0; 16]),
        })
        .unwrap();

        let result = validate_payload_against_schema(&payload, &schema, type_index);
        assert!(result.is_ok())
    }

    #[test]
    pub fn longer_length_vec_is_not_interchangeable_with_array() {
        let (type_index, schema) =
            generate_full_schema_from_single_type::<TestStructArray, NoCustomTypeExtension>();
        let payload = basic_encode(&TestStructVec {
            x: Vec::from([0; 17]),
        })
        .unwrap();

        let result = validate_payload_against_schema(&payload, &schema, type_index);
        assert!(matches!(
            result,
            Err(LocatedValidationError {
                error: PayloadValidationError::TypeValidationError(
                    TypeValidationError::LengthValidationError {
                        required: LengthValidation {
                            min: Some(16),
                            max: Some(16)
                        },
                        actual: 17,
                    }
                ),
                ..
            })
        ))
    }

    #[derive(Debug, Clone, Sbor)]
    pub enum SimpleEnum {
        Unit,
        Unnamed(String),
        Named { x: u8, y: u32 },
    }

    #[derive(Debug, Clone, Sbor)]
    pub struct SimpleStruct {
        pub unit: (),
        pub boolean: bool,
        pub u8: u8,
        pub u16: u16,
        pub u32: u32,
        pub u64: u64,
        pub u128: u128,
        pub i8: i8,
        pub i16: i16,
        pub i32: i32,
        pub i64: i64,
        pub i128: i128,
        pub string: String,
        pub enumeration: (SimpleEnum, SimpleEnum, SimpleEnum),
        pub recursive_struct: Option<Box<SimpleStruct>>,
        pub vector: Vec<u16>,
        pub set: HashSet<String>,
        pub map: BTreeMap<String, String>,
    }

    #[test]
    pub fn test_basic_payload_validation() {
        let mut x = SimpleStruct {
            unit: (),
            boolean: true,
            u8: 1,
            u16: 2,
            u32: 3,
            u64: 4,
            u128: 5,
            i8: 6,
            i16: 7,
            i32: 8,
            i64: 9,
            i128: 10,
            string: "String".to_owned(),
            enumeration: (
                SimpleEnum::Unit,
                SimpleEnum::Named { x: 1, y: 2 },
                SimpleEnum::Unnamed("a".to_string()),
            ),
            recursive_struct: None,
            vector: vec![1, 2],
            set: hashset!("a".to_string(), "b".to_string()),
            map: btreemap!("c".to_string() => "d".to_string()),
        };
        x.recursive_struct = Some(Box::new(x.clone()));

        let bytes = basic_encode(&x).unwrap();
        let (type_index, schema) =
            generate_full_schema_from_single_type::<SimpleStruct, NoCustomTypeExtension>();
        let result = validate_payload_against_schema(&bytes, &schema, type_index);
        assert!(result.is_ok())
    }

    #[test]
    pub fn test_vec_u8_with_min_max() {
        let t0 = BasicTypeData {
            kind: BasicTypeKind::Array {
                element_type: LocalTypeIndex::SchemaLocalIndex(1),
            },
            metadata: TypeMetadata::unnamed(),
            validation: TypeValidation::Array(LengthValidation {
                min: 0.into(),
                max: 1.into(),
            }),
        };
        let t1 = BasicTypeData {
            kind: BasicTypeKind::U8,
            metadata: TypeMetadata::unnamed(),
            validation: TypeValidation::U8(NumericValidation {
                min: 5.into(),
                max: 6.into(),
            }),
        };
        let schema = BasicSchema {
            type_kinds: vec![t0.kind, t1.kind],
            type_metadata: vec![t0.metadata, t1.metadata],
            type_validations: vec![t0.validation, t1.validation],
        };

        assert_eq!(
            validate_payload_against_schema(
                &basic_encode(&vec![5u8]).unwrap(),
                &schema,
                LocalTypeIndex::SchemaLocalIndex(0),
            ),
            Ok(())
        );

        assert_eq!(
            validate_payload_against_schema(
                &basic_encode(&vec![8u8]).unwrap(),
                &schema,
                LocalTypeIndex::SchemaLocalIndex(0),
            )
            .map_err(|e| e.error),
            Err(PayloadValidationError::TypeValidationError(
                TypeValidationError::U8ValidationError {
                    required: NumericValidation {
                        min: Some(5),
                        max: Some(6)
                    },
                    actual: 8
                }
            ))
        );

        assert_eq!(
            validate_payload_against_schema(
                &basic_encode(&vec![5u8, 5u8]).unwrap(),
                &schema,
                LocalTypeIndex::SchemaLocalIndex(0),
            )
            .map_err(|e| e.error),
            Err(PayloadValidationError::TypeValidationError(
                TypeValidationError::LengthValidationError {
                    required: LengthValidation {
                        min: Some(0),
                        max: Some(1)
                    },
                    actual: 2
                }
            ))
        );
    }

    #[derive(Sbor)]
    #[sbor(custom_value_kind = "NoCustomValueKind")]
    struct MyStruct {
        hello: MyEnum,
    }

    #[derive(Sbor)]
    #[sbor(custom_value_kind = "NoCustomValueKind")]
    enum MyEnum {
        Option1(HashMap<String, Vec<(BasicValue,)>>),
        Option2 { inner: Box<MyEnum> },
    }

    #[test]
    pub fn full_location_path_is_readable() {
        let value = MyStruct {
            hello: MyEnum::Option2 {
                inner: Box::new(MyEnum::Option1(hashmap!(
                    "test".to_string() => vec![
                        (BasicValue::Enum {
                            discriminator: 6,
                            fields: vec![
                                BasicValue::Tuple {
                                    fields: vec![
                                        BasicValue::U8 { value: 1 },
                                        BasicValue::Map {
                                            key_value_kind: BasicValueKind::U8,
                                            value_value_kind: BasicValueKind::U8,
                                            entries: vec![
                                                (BasicValue::U8 { value: 7 }, BasicValue::U8 { value: 21 },),
                                            ]
                                        }
                                    ]
                                },
                            ]
                        },),

                    ]
                ))),
            },
        };
        let payload = basic_encode(&value).unwrap();
        // We cut off the payload to get a decode error near the end of the encoding!
        let cut_off_payload = &payload[0..payload.len() - 2];

        let (type_index, schema) =
            generate_full_schema_from_single_type::<MyStruct, NoCustomTypeExtension>();

        let Err(error) = validate_payload_against_schema(
            &cut_off_payload,
            &schema,
            type_index,
        ) else {
            panic!("Validation did not error with too short a payload");
        };
        let path_message = error.location.path_to_string(&schema);

        assert_eq!(
            path_message,
            "MyStruct.hello[0]->MyEnum::Option2{1}.inner[0]->MyEnum::Option1{0}.[0]->Map[0].Value->Array[0]->Tuple.[0]->Enum::{6}.[0]->Tuple.[1]->Map[0].Key"
        );
    }
}
