use super::*;
use crate::rust::fmt::*;
use crate::rust::format;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullLocation<'s, C: CustomTraversal> {
    pub start_offset: usize,
    pub end_offset: usize,
    pub ancestor_path: Vec<(ContainerState<C>, ContainerType<'s>)>,
    pub current_value_info: Option<CurrentValueInfo<C::CustomValueKind>>,
}

impl<'s, C: CustomTraversal> FullLocation<'s, C> {
    /// This enables a full path to be provided in an error message, which can have a debug such as:
    /// EG: `MyStruct.hello[0]->MyEnum::Option2{1}.inner[0]->MyEnum::Option1{0}.[0]->Map[0].Value->Array[0]->Tuple.[0]->Enum::{6}.[0]->Tuple.[1]->Map[0].Key`
    ///
    /// As much information is extracted from the Type as possible, falling back to data from the value model
    /// if the Type is Any.
    pub fn path_to_string<E: CustomTypeExtension>(&self, schema: &Schema<E>) -> String {
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
                        .map(|d| {
                            format!(
                                "::{}{{{}}}",
                                d.get_name().expect("Enum variants require names"),
                                variant_header.variant
                            )
                        })
                        .unwrap_or_else(|| format!("::{{{}}}", variant_header.variant));
                    let index = variant_data.and_then(|d| match &d.child_names {
                        Some(ChildNames::NamedFields(fields)) => {
                            fields.get(container_state.current_child_index())
                        }
                        _ => None,
                    });
                    let field_part = index
                        .map(|i| format!(".{}[{}]", i, current_index))
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
                        .map(|i| format!(".{}[{}]", i, current_index))
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
                    .map(|d| {
                        format!(
                            "::{}{{{}}}",
                            d.get_name().expect("Enum variants require names"),
                            variant
                        )
                    })
                    .unwrap_or_else(|| format!("::{{{}}}", variant));
                write!(buf, "{}{}", type_name, variant_part).unwrap();
            } else {
                write!(buf, "{}", type_name).unwrap();
            }
        }
        buf
    }
}
