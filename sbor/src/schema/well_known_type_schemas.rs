use super::*;

use well_known_basic_schemas::*;

pub const CUSTOM_WELL_KNOWN_TYPE_START: u8 = 0x80;

pub mod well_known_basic_schemas {
    use sbor::*;

    // These must be usable in a const context
    pub const ANY_INDEX: u8 = 0x40;

    pub const UNIT_INDEX: u8 = TYPE_UNIT;
    pub const BOOL_INDEX: u8 = TYPE_BOOL;

    pub const I8_INDEX: u8 = TYPE_I8;
    pub const I16_INDEX: u8 = TYPE_I16;
    pub const I32_INDEX: u8 = TYPE_I32;
    pub const I64_INDEX: u8 = TYPE_I64;
    pub const I128_INDEX: u8 = TYPE_I128;

    pub const U8_INDEX: u8 = TYPE_U8;
    pub const U16_INDEX: u8 = TYPE_U16;
    pub const U32_INDEX: u8 = TYPE_U32;
    pub const U64_INDEX: u8 = TYPE_U64;
    pub const U128_INDEX: u8 = TYPE_U128;

    pub const STRING_INDEX: u8 = TYPE_STRING;

    pub const BYTES_INDEX: u8 = 0x41;
}

pub enum WellKnownType<X: CustomWellKnownType> {
    // Any
    Any,
    // Basic, limitless
    Unit,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    String,
    // Common aliases
    Bytes,
    // Custom
    Custom(X),
}

pub fn resolve_well_known_type_data<W: CustomWellKnownType>(
    well_known_index: u8,
) -> Option<LocalTypeData<W::CustomTypeSchema, SchemaLocalTypeRef>> {
    let type_data = match well_known_index {
        ANY_INDEX => LocalTypeData::named("Any", TypeSchema::Any),

        UNIT_INDEX => LocalTypeData::named("-", TypeSchema::Unit),
        BOOL_INDEX => LocalTypeData::named("Bool", TypeSchema::Bool),

        I8_INDEX => LocalTypeData::named(
            "I8",
            TypeSchema::I8 {
                validation: NumericValidation::none(),
            },
        ),
        I16_INDEX => LocalTypeData::named(
            "I16",
            TypeSchema::I16 {
                validation: NumericValidation::none(),
            },
        ),
        I32_INDEX => LocalTypeData::named(
            "I32",
            TypeSchema::I32 {
                validation: NumericValidation::none(),
            },
        ),
        I64_INDEX => LocalTypeData::named(
            "I64",
            TypeSchema::I64 {
                validation: NumericValidation::none(),
            },
        ),
        I128_INDEX => LocalTypeData::named(
            "I128",
            TypeSchema::I128 {
                validation: NumericValidation::none(),
            },
        ),

        U8_INDEX => LocalTypeData::named(
            "U8",
            TypeSchema::U8 {
                validation: NumericValidation::none(),
            },
        ),
        U16_INDEX => LocalTypeData::named(
            "U16",
            TypeSchema::U16 {
                validation: NumericValidation::none(),
            },
        ),
        U32_INDEX => LocalTypeData::named(
            "U32",
            TypeSchema::U32 {
                validation: NumericValidation::none(),
            },
        ),
        U64_INDEX => LocalTypeData::named(
            "U64",
            TypeSchema::U64 {
                validation: NumericValidation::none(),
            },
        ),
        U128_INDEX => LocalTypeData::named(
            "U128",
            TypeSchema::U128 {
                validation: NumericValidation::none(),
            },
        ),

        STRING_INDEX => LocalTypeData::named(
            "String",
            TypeSchema::String {
                length_validation: LengthValidation::none(),
            },
        ),

        BYTES_INDEX => LocalTypeData::named(
            "Bytes",
            TypeSchema::Array {
                element_sbor_type_id: sbor::TYPE_U8,
                element_type: SchemaLocalTypeRef::WellKnown(U8_INDEX),
                length_validation: LengthValidation::none(),
            },
        ),
        index if index >= CUSTOM_WELL_KNOWN_TYPE_START => return W::from_well_known_index(index),
        _ => return None,
    };
    Some(type_data)
}

pub trait CustomWellKnownType {
    type CustomTypeSchema: CustomTypeSchema;

    fn from_well_known_index(
        well_known_index: u8,
    ) -> Option<LocalTypeData<Self::CustomTypeSchema, SchemaLocalTypeRef>>;
}
