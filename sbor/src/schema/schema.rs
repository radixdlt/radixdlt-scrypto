use crate::rust::borrow::Cow;
use crate::rust::vec::Vec;
use crate::*;

/// An array of custom types, and associated extra information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema<C: CustomTypeExtension> {
    pub type_kinds:
        Vec<TypeKind<C::CustomTypeId, C::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>>,
    pub type_naming: Vec<TypeMetadata>,
}

// TODO: Could get rid of the Cow by using some per-custom type once_cell to cache basic well-known-types,
//       and return references to the static cached values
pub struct ResolvedTypeData<'a, C: CustomTypeExtension> {
    pub kind: Cow<'a, TypeKind<C::CustomTypeId, C::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>>,
    pub naming: Cow<'a, TypeMetadata>,
}

impl<E: CustomTypeExtension> Schema<E> {
    pub fn resolve<'a>(&'a self, type_ref: LocalTypeIndex) -> Option<ResolvedTypeData<'a, E>> {
        match type_ref {
            LocalTypeIndex::WellKnown(index) => {
                resolve_well_known_type::<E>(index).map(|local_type_data| ResolvedTypeData {
                    kind: Cow::Owned(local_type_data.kind),
                    naming: Cow::Owned(local_type_data.metadata),
                })
            }
            LocalTypeIndex::SchemaLocalIndex(index) => {
                match (self.type_kinds.get(index), self.type_naming.get(index)) {
                    (Some(schema), Some(naming)) => Some(ResolvedTypeData {
                        kind: Cow::Borrowed(schema),
                        naming: Cow::Borrowed(naming),
                    }),
                    (None, None) => None,
                    _ => panic!("Index existed in exactly one of schema and naming"),
                }
            }
        }
    }
}
