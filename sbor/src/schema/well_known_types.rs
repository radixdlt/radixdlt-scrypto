use super::*;

use well_known_basic_types::*;

pub const CUSTOM_WELL_KNOWN_TYPE_START: u8 = 0x80;

pub mod well_known_basic_types {
    use sbor::*;

    // These ids must be usable in a const context
    pub const ANY_ID: u8 = 0x40;

    pub const UNIT_ID: u8 = TYPE_UNIT;
    pub const BOOL_ID: u8 = TYPE_BOOL;

    pub const I8_ID: u8 = TYPE_I8;
    pub const I16_ID: u8 = TYPE_I16;
    pub const I32_ID: u8 = TYPE_I32;
    pub const I64_ID: u8 = TYPE_I64;
    pub const I128_ID: u8 = TYPE_I128;

    pub const U8_ID: u8 = TYPE_U8;
    pub const U16_ID: u8 = TYPE_U16;
    pub const U32_ID: u8 = TYPE_U32;
    pub const U64_ID: u8 = TYPE_U64;
    pub const U128_ID: u8 = TYPE_U128;

    pub const STRING_ID: u8 = TYPE_STRING;

    pub const BYTES_ID: u8 = 0x41;
}

pub fn resolve_well_known_type<E: CustomTypeExtension>(
    well_known_index: u8,
) -> Option<TypeData<E::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>> {
    let type_data = match well_known_index {
        ANY_ID => TypeData::named_no_child_names("Any", TypeKind::Any),

        UNIT_ID => TypeData::named_no_child_names("-", TypeKind::Unit),
        BOOL_ID => TypeData::named_no_child_names("Bool", TypeKind::Bool),

        I8_ID => TypeData::named_no_child_names(
            "I8",
            TypeKind::I8 {
                validation: NumericValidation::none(),
            },
        ),
        I16_ID => TypeData::named_no_child_names(
            "I16",
            TypeKind::I16 {
                validation: NumericValidation::none(),
            },
        ),
        I32_ID => TypeData::named_no_child_names(
            "I32",
            TypeKind::I32 {
                validation: NumericValidation::none(),
            },
        ),
        I64_ID => TypeData::named_no_child_names(
            "I64",
            TypeKind::I64 {
                validation: NumericValidation::none(),
            },
        ),
        I128_ID => TypeData::named_no_child_names(
            "I128",
            TypeKind::I128 {
                validation: NumericValidation::none(),
            },
        ),

        U8_ID => TypeData::named_no_child_names(
            "U8",
            TypeKind::U8 {
                validation: NumericValidation::none(),
            },
        ),
        U16_ID => TypeData::named_no_child_names(
            "U16",
            TypeKind::U16 {
                validation: NumericValidation::none(),
            },
        ),
        U32_ID => TypeData::named_no_child_names(
            "U32",
            TypeKind::U32 {
                validation: NumericValidation::none(),
            },
        ),
        U64_ID => TypeData::named_no_child_names(
            "U64",
            TypeKind::U64 {
                validation: NumericValidation::none(),
            },
        ),
        U128_ID => TypeData::named_no_child_names(
            "U128",
            TypeKind::U128 {
                validation: NumericValidation::none(),
            },
        ),

        STRING_ID => TypeData::named_no_child_names(
            "String",
            TypeKind::String {
                length_validation: LengthValidation::none(),
            },
        ),

        BYTES_ID => TypeData::named_no_child_names(
            "Bytes",
            TypeKind::Array {
                element_type: LocalTypeIndex::WellKnown(U8_ID),
                length_validation: LengthValidation::none(),
            },
        ),
        index if index >= CUSTOM_WELL_KNOWN_TYPE_START => {
            return E::resolve_custom_well_known_type(index)
        }
        _ => return None,
    };
    Some(type_data)
}
