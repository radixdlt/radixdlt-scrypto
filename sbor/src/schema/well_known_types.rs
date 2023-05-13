pub const CUSTOM_WELL_KNOWN_TYPE_START: u8 = 0x80;

pub mod basic_well_known_types {
    use sbor::rust::prelude::*;
    use sbor::*;

    pub const BOOL_ID: u8 = VALUE_KIND_BOOL;
    pub fn bool_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::Bool)
    }

    pub const I8_ID: u8 = VALUE_KIND_I8;
    pub fn i8_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::I8)
    }

    pub const I16_ID: u8 = VALUE_KIND_I16;
    pub fn i16_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::I16)
    }

    pub const I32_ID: u8 = VALUE_KIND_I32;
    pub fn i32_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::I32)
    }

    pub const I64_ID: u8 = VALUE_KIND_I64;
    pub fn i64_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::I64)
    }

    pub const I128_ID: u8 = VALUE_KIND_I128;
    pub fn i128_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::I128)
    }

    pub const U8_ID: u8 = VALUE_KIND_U8;
    pub fn u8_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::U8)
    }

    pub const U16_ID: u8 = VALUE_KIND_U16;
    pub fn u16_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::U16)
    }

    pub const U32_ID: u8 = VALUE_KIND_U32;
    pub fn u32_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::U32)
    }

    pub const U64_ID: u8 = VALUE_KIND_U64;
    pub fn u64_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::U64)
    }

    pub const U128_ID: u8 = VALUE_KIND_U128;
    pub fn u128_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::U128)
    }

    pub const STRING_ID: u8 = VALUE_KIND_STRING;
    pub fn string_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::String)
    }

    pub const ANY_ID: u8 = 0x40; // Any type
    pub fn any_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::Any)
    }

    pub const BYTES_ID: u8 = 0x41; // `Vec<u8>`
    pub fn bytes_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::Array {
            element_type: WellKnownTypeIndex(U8_ID).into(),
        })
    }

    pub const UNIT_ID: u8 = 0x42; // `()`
    pub fn unit_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::Tuple {
            field_types: vec![],
        })
    }
}

#[macro_export]
macro_rules! create_well_known_lookup {
    ($lookup_name: ident, $custom_type_kind: ty, [$(($id: path, $type_data: expr),)*]) => {
        paste::paste! {
            const [<$lookup_name:upper _INIT>]: Option<TypeData<$custom_type_kind, LocalTypeIndex>> = None;

            lazy_static::lazy_static! {
                static ref $lookup_name: [Option<TypeData<$custom_type_kind, LocalTypeIndex>>; 256] = {
                    use sbor::basic_well_known_types::*;

                    let mut lookup: [Option<TypeData<$custom_type_kind, LocalTypeIndex>>; 256] = [ [<$lookup_name:upper _INIT>]; 256 ];

                    // Now add in the basic types
                    lookup[BOOL_ID as usize] = Some(bool_type_data());
                    lookup[I8_ID as usize] = Some(i8_type_data());
                    lookup[I16_ID as usize] = Some(i16_type_data());
                    lookup[I32_ID as usize] = Some(i32_type_data());
                    lookup[I64_ID as usize] = Some(i64_type_data());
                    lookup[I128_ID as usize] = Some(i128_type_data());
                    lookup[U8_ID as usize] = Some(u8_type_data());
                    lookup[U16_ID as usize] = Some(u16_type_data());
                    lookup[U32_ID as usize] = Some(u32_type_data());
                    lookup[U64_ID as usize] = Some(u64_type_data());
                    lookup[U128_ID as usize] = Some(u128_type_data());
                    lookup[STRING_ID as usize] = Some(string_type_data());
                    lookup[ANY_ID as usize] = Some(any_type_data());
                    lookup[BYTES_ID as usize] = Some(bytes_type_data());
                    lookup[UNIT_ID as usize] = Some(unit_type_data());
                    // And now add in the custom types
                    $(lookup[$id as usize] = Some($type_data);)*

                    // And return the lookup
                    lookup
                };

            }
        }
    };
}
