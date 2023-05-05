use super::*;
use crate::rust::fmt::*;
use crate::rust::format;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullLocation<'s, E: CustomExtension> {
    pub start_offset: usize,
    pub end_offset: usize,
    pub ancestor_path: Vec<(ContainerState<E::CustomTraversal>, ContainerType<'s>)>,
    pub current_value_info: Option<CurrentValueInfo<E>>,
    pub error: Option<TypedTraversalError<E>>,
}

impl<'s, E: CustomExtension> FullLocation<'s, E> {
    /// This enables a full path to be provided in an error message, which can have a debug such as:
    /// EG: `MyStruct.hello[0]->MyEnum::Option2{1}.inner[0]->MyEnum::Option1{0}.[0]->Map[0].Value->Array[0]->Tuple.[0]->Enum::{6}.[0]->Tuple.[1]->Map[0].Key`
    ///
    /// As much information is extracted from the Type as possible, falling back to data from the value model
    /// if the Type is Any.
    pub fn path_to_string(&self, schema: &Schema<E::CustomSchema>) -> String {
        let mut buf = String::new();
        let mut is_first = true;
        for (container_state, container_type) in self.ancestor_path.iter() {
            if is_first {
                is_first = false;
            } else {
                write!(buf, "->").unwrap();
            }
            let type_index = container_type.self_type();
            let metadata = schema.resolve_type_metadata(type_index);
            let type_name = metadata
                .and_then(|m| m.get_name())
                .unwrap_or_else(|| container_state.container_header.value_kind_name());
            let current_index = container_state.current_child_index();
            let header = container_state.container_header;
            match header {
                ContainerHeader::EnumVariant(variant_header) => {
                    let variant_data = metadata.and_then(|v| match &v.child_names {
                        Some(ChildNames::EnumVariants(variants)) => {
                            variants.get(&variant_header.variant)
                        }
                        _ => None,
                    });
                    let variant_part = variant_data
                        .and_then(|d| d.get_name())
                        .map(|variant_name| {
                            format!("::{{{}|{}}}", variant_header.variant, variant_name,)
                        })
                        .unwrap_or_else(|| format!("::{{{}}}", variant_header.variant));
                    let index = variant_data.and_then(|d| match &d.child_names {
                        Some(ChildNames::NamedFields(fields)) => {
                            fields.get(container_state.current_child_index())
                        }
                        _ => None,
                    });
                    let field_part = index
                        .map(|field_name| format!(".[{}|{}]", current_index, field_name))
                        .unwrap_or_else(|| format!(".[{}]", current_index));
                    write!(buf, "{}{}{}", type_name, variant_part, field_part).unwrap();
                }
                ContainerHeader::Tuple(_) => {
                    let index = metadata.and_then(|d| match &d.child_names {
                        Some(ChildNames::NamedFields(fields)) => {
                            fields.get(container_state.current_child_index())
                        }
                        _ => None,
                    });
                    let field_part = index
                        .map(|field_name| format!(".[{}|{}]", current_index, field_name))
                        .unwrap_or_else(|| format!(".[{}]", current_index));
                    write!(buf, "{}{}", type_name, field_part).unwrap();
                }
                ContainerHeader::Array(_) => {
                    write!(buf, "{}[{}]", type_name, current_index).unwrap();
                }
                ContainerHeader::Map(_) => {
                    let child_index = container_state.current_child_index() / 2;
                    let key_or_value = if container_state.current_child_index() % 2 == 0 {
                        "Key"
                    } else {
                        "Value"
                    };
                    write!(buf, "{}[{}].{}", type_name, child_index, key_or_value).unwrap();
                }
            }
        }
        if let Some(current_value_info) = &self.current_value_info {
            if !is_first {
                write!(buf, "->").unwrap();
            }
            let type_kind = schema
                .resolve_type_kind(current_value_info.type_index)
                .expect("Type index not found in given schema");
            let metadata = if !matches!(type_kind, TypeKind::Any) {
                schema.resolve_type_metadata(current_value_info.type_index)
            } else {
                None
            };
            let type_name = metadata
                .and_then(|m| m.get_name_string())
                .unwrap_or_else(|| current_value_info.value_kind.to_string());
            if let Some(variant) = current_value_info.variant {
                let variant_data = metadata.and_then(|v| match &v.child_names {
                    Some(ChildNames::EnumVariants(variants)) => variants.get(&variant),
                    _ => None,
                });
                let variant_part = variant_data
                    .and_then(|d| d.get_name())
                    .map(|variant_name| format!("::{{{}|{}}}", variant, variant_name,))
                    .unwrap_or_else(|| format!("::{{{}}}", variant));
                write!(buf, "{}{}", type_name, variant_part).unwrap();
            } else {
                write!(buf, "{}", type_name).unwrap();
            }
        }
        if let Some(error) = &self.error {
            if !is_first && !self.current_value_info.is_some() {
                write!(buf, "->").unwrap();
            }
            write!(buf, "[ERROR] ").unwrap();
            match error {
                TypedTraversalError::ValueMismatchWithType(
                    TypeMismatchError::MismatchingType {
                        expected_type_kind,
                        actual_value_kind,
                        ..
                    },
                ) => {
                    write!(
                        buf,
                        "{{ expected_type: {:?}, found: {:?} }}",
                        expected_type_kind, actual_value_kind
                    )
                    .unwrap();
                }
                TypedTraversalError::ValueMismatchWithType(
                    TypeMismatchError::MismatchingChildElementType {
                        expected_type_kind,
                        actual_value_kind,
                        ..
                    },
                ) => {
                    write!(
                        buf,
                        "{{ expected_child_type: {:?}, found: {:?} }}",
                        expected_type_kind, actual_value_kind
                    )
                    .unwrap();
                }
                TypedTraversalError::ValueMismatchWithType(
                    TypeMismatchError::MismatchingChildKeyType {
                        expected_type_kind,
                        actual_value_kind,
                        ..
                    },
                ) => {
                    write!(
                        buf,
                        "{{ expected_key_type: {:?}, found: {:?} }}",
                        expected_type_kind, actual_value_kind
                    )
                    .unwrap();
                }
                TypedTraversalError::ValueMismatchWithType(
                    TypeMismatchError::MismatchingChildValueType {
                        expected_type_kind,
                        actual_value_kind,
                        ..
                    },
                ) => {
                    write!(
                        buf,
                        "{{ expected_value_type: {:?}, found: {:?} }}",
                        expected_type_kind, actual_value_kind
                    )
                    .unwrap();
                }
                TypedTraversalError::ValueMismatchWithType(
                    TypeMismatchError::MismatchingTupleLength {
                        expected, actual, ..
                    },
                ) => {
                    write!(
                        buf,
                        "{{ expected_field_count: {:?}, found: {:?} }}",
                        expected, actual
                    )
                    .unwrap();
                }
                TypedTraversalError::ValueMismatchWithType(
                    TypeMismatchError::MismatchingEnumVariantLength {
                        expected, actual, ..
                    },
                ) => {
                    write!(
                        buf,
                        "{{ expected_field_count: {:?}, found: {:?} }}",
                        expected, actual
                    )
                    .unwrap();
                }
                TypedTraversalError::ValueMismatchWithType(
                    TypeMismatchError::UnknownEnumVariant { variant, .. },
                ) => {
                    write!(buf, "{{ unknown_variant_id: {:?} }}", variant).unwrap();
                }
                TypedTraversalError::TypeIndexNotFound(_) | TypedTraversalError::DecodeError(_) => {
                    write!(buf, "{:?}", error).unwrap();
                }
            }
        }
        buf
    }
}
