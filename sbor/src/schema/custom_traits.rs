use super::*;
use crate::rust::collections::IndexSet;
use crate::CustomTypeId;

pub trait CustomTypeKind<L: SchemaTypeLink>: Clone + PartialEq + Eq {
    type CustomTypeId: CustomTypeId;
    type CustomTypeExtension: CustomTypeExtension<
        CustomTypeId = Self::CustomTypeId,
        CustomTypeKind<L> = Self,
    >;
}

pub trait CustomTypeExtension {
    type CustomTypeId: CustomTypeId;
    type CustomTypeKind<L: SchemaTypeLink>: CustomTypeKind<
        L,
        CustomTypeId = Self::CustomTypeId,
        CustomTypeExtension = Self,
    >;

    fn linearize_type_kind(
        type_kind: Self::CustomTypeKind<GlobalTypeId>,
        schemas: &IndexSet<TypeHash>,
    ) -> Self::CustomTypeKind<LocalTypeIndex>;
    fn resolve_custom_well_known_type(
        well_known_index: u8,
    ) -> Option<TypeData<Self::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>>;
}
