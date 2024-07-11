use super::*;
use crate::rust::fmt::*;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullLocation<'s, E: CustomExtension> {
    pub start_offset: usize,
    pub end_offset: usize,
    pub ancestor_path: Vec<(AncestorState<E::CustomTraversal>, ContainerType<'s>)>,
    pub current_value_info: Option<CurrentValueInfo<E>>,
}

impl<'s, 'a, E: CustomExtension> PathAnnotate
    for (&'a FullLocation<'s, E>, &'a Schema<E::CustomSchema>)
{
    fn iter_ancestor_path(&self) -> Box<dyn Iterator<Item = AnnotatedSborAncestor<'_>> + '_> {
        let (full_location, schema) = self;
        let schema = *schema;
        let iterator =
            full_location
                .ancestor_path
                .iter()
                .map(|(container_state, container_type)| {
                    let type_id = container_type.self_type();
                    let metadata = schema.resolve_type_metadata(type_id);
                    let name = metadata
                        .and_then(|m| m.get_name())
                        .unwrap_or_else(|| container_state.container_header.value_kind_name());
                    let header = container_state.container_header;

                    let current_child_index = container_state.current_child_index;

                    let container = match header {
                        ContainerHeader::EnumVariant(variant_header) => {
                            let discriminator = variant_header.variant;
                            let variant_data =
                                metadata.and_then(|m| m.get_enum_variant_data(discriminator));
                            let variant_name =
                                variant_data.and_then(|d| d.get_name()).map(Cow::Borrowed);
                            let field_index = current_child_index;
                            let field_name = variant_data
                                .and_then(|d| d.get_field_name(field_index))
                                .map(Cow::Borrowed);
                            AnnotatedSborAncestorContainer::EnumVariant {
                                discriminator,
                                variant_name,
                                field_index,
                                field_name,
                            }
                        }
                        ContainerHeader::Tuple(_) => {
                            let field_index = current_child_index;
                            let field_name = metadata
                                .and_then(|d| d.get_field_name(field_index))
                                .map(Cow::Borrowed);
                            AnnotatedSborAncestorContainer::Tuple {
                                field_index,
                                field_name,
                            }
                        }
                        ContainerHeader::Array(_) => {
                            let index = Some(current_child_index);
                            AnnotatedSborAncestorContainer::Array { index }
                        }
                        ContainerHeader::Map(_) => {
                            let index = Some(current_child_index / 2);
                            let entry_part = if current_child_index % 2 == 0 {
                                MapEntryPart::Key
                            } else {
                                MapEntryPart::Value
                            };
                            AnnotatedSborAncestorContainer::Map { index, entry_part }
                        }
                    };

                    AnnotatedSborAncestor {
                        name: Cow::Borrowed(name),
                        container,
                    }
                });
        Box::new(iterator)
    }

    fn annotated_leaf(&self) -> Option<AnnotatedSborPartialLeaf<'_>> {
        let current_value_info = self.0.current_value_info.as_ref()?;
        let schema = self.1;

        let metadata = schema.resolve_type_metadata(current_value_info.type_id);
        let name = metadata
            .and_then(|m| m.get_name())
            .map(Cow::Borrowed)
            // We should consider falling back to the TypeKind's name before falling back to the value kind
            .unwrap_or_else(|| Cow::Owned(current_value_info.value_kind.to_string()));
        let partial_kinded_data = match current_value_info.value_kind {
            ValueKind::Enum => {
                if let Some(variant_discriminator) = current_value_info.variant {
                    let variant_data =
                        metadata.and_then(|v| v.get_enum_variant_data(variant_discriminator));
                    let variant_name = variant_data.and_then(|d| d.get_name()).map(Cow::Borrowed);

                    Some(AnnotatedSborPartialLeafLocator::EnumVariant {
                        variant_discriminator: Some(variant_discriminator),
                        variant_name,
                        field_index: None,
                        field_name: None,
                    })
                } else {
                    None
                }
            }
            _ => None,
        };

        Some(AnnotatedSborPartialLeaf {
            name,
            partial_leaf_locator: partial_kinded_data,
        })
    }
}

impl<'s, E: CustomExtension> FullLocation<'s, E> {
    /// This enables a full path to be provided in an error message, which can have a debug such as:
    /// EG: `MyStruct.hello[0]->MyEnum::Option2{1}.inner[0]->MyEnum::Option1{0}.[0]->Map[0].Value->Array[0]->Tuple.[0]->Enum::{6}.[0]->Tuple.[1]->Map[0].Key`
    ///
    /// As much information is extracted from the Type as possible, falling back to data from the value model
    /// if the Type is Any.
    pub fn path_to_string(&self, schema: &Schema<E::CustomSchema>) -> String {
        (self, schema).format_path()
    }
}
