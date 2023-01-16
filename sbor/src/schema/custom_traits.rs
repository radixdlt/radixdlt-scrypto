use super::*;
use crate::rust::collections::*;
use crate::rust::fmt::Debug;
use crate::CustomValueKind;

pub trait CustomTypeKind<L: SchemaTypeLink>: Clone + PartialEq + Eq {
    type CustomValueKind: CustomValueKind;
    type CustomTypeExtension: CustomTypeExtension<
        CustomValueKind = Self::CustomValueKind,
        CustomTypeKind<L> = Self,
    >;
}

pub trait CustomTypeValidation: Debug + Clone + PartialEq + Eq {}

pub trait CustomTypeExtension {
    type CustomValueKind: CustomValueKind;
    type CustomTypeKind<L: SchemaTypeLink>: CustomTypeKind<
        L,
        CustomValueKind = Self::CustomValueKind,
        CustomTypeExtension = Self,
    >;
    type CustomTypeValidation: CustomTypeValidation;

    fn linearize_type_kind(
        type_kind: Self::CustomTypeKind<GlobalTypeId>,
        type_indices: &BTreeMap<TypeHash, usize>,
    ) -> Self::CustomTypeKind<LocalTypeIndex>;

    fn resolve_custom_well_known_type(
        well_known_index: u8,
    ) -> Option<TypeData<Self::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>>;
}
