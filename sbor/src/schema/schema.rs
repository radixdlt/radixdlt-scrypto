use crate::rust::borrow::Cow;
use crate::rust::vec::Vec;
use crate::*;

/// An array of custom type kinds, and associated extra information which can attach to the type kinds
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema<C: CustomTypeExtension> {
    pub type_kinds:
        Vec<TypeKind<C::CustomValueKind, C::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>>,
    pub type_metadata: Vec<NovelTypeMetadata>,
}

// TODO: Could get rid of the Cow by using some per-custom type once_cell to cache basic well-known-types,
//       and return references to the static cached values
pub struct ResolvedTypeData<'a, C: CustomTypeExtension> {
    pub kind:
        Cow<'a, TypeKind<C::CustomValueKind, C::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>>,
    pub metadata: Cow<'a, TypeMetadata>,
}

impl<E: CustomTypeExtension> Schema<E> {
    pub fn resolve<'a>(&'a self, type_ref: LocalTypeIndex) -> Option<ResolvedTypeData<'a, E>> {
        match type_ref {
            LocalTypeIndex::WellKnown(index) => {
                resolve_well_known_type::<E>(index).map(|local_type_data| ResolvedTypeData {
                    kind: Cow::Owned(local_type_data.kind),
                    metadata: Cow::Owned(local_type_data.metadata),
                })
            }
            LocalTypeIndex::SchemaLocalIndex(index) => {
                match (self.type_kinds.get(index), self.type_metadata.get(index)) {
                    (Some(schema), Some(novel_metadata)) => Some(ResolvedTypeData {
                        kind: Cow::Borrowed(schema),
                        metadata: Cow::Borrowed(&novel_metadata.type_metadata),
                    }),
                    (None, None) => None,
                    _ => panic!("Index existed in exactly one of schema and naming"),
                }
            }
        }
    }
}
