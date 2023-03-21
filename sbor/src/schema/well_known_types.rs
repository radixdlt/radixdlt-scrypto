pub const CUSTOM_WELL_KNOWN_TYPE_START: u8 = 0x80;

pub mod basic_well_known_types {
    use sbor::*;

    // These ids must be usable in a const context
    pub const BOOL_ID: u8 = VALUE_KIND_BOOL;
    pub const I8_ID: u8 = VALUE_KIND_I8;
    pub const I16_ID: u8 = VALUE_KIND_I16;
    pub const I32_ID: u8 = VALUE_KIND_I32;
    pub const I64_ID: u8 = VALUE_KIND_I64;
    pub const I128_ID: u8 = VALUE_KIND_I128;
    pub const U8_ID: u8 = VALUE_KIND_U8;
    pub const U16_ID: u8 = VALUE_KIND_U16;
    pub const U32_ID: u8 = VALUE_KIND_U32;
    pub const U64_ID: u8 = VALUE_KIND_U64;
    pub const U128_ID: u8 = VALUE_KIND_U128;
    pub const STRING_ID: u8 = VALUE_KIND_STRING;
    pub const ANY_ID: u8 = 0x40; // Any type
    pub const BYTES_ID: u8 = 0x41; // `Vec<u8>`
    pub const UNIT_ID: u8 = 0x42; // `()`
}

#[macro_export]
macro_rules! create_well_known_lookup {
    ($lookup_name: ident, $custom_type_kind: ty, [$(($id: path, $type_data: expr),)*]) => {
        lazy_static::lazy_static! {
            static ref $lookup_name: [Option<TypeData<$custom_type_kind, LocalTypeIndex>>; 256] = {
                let mut lookup = {
                    // Initialize the array with None, following the example here:
                    // https://github.com/rust-lang/rust/issues/54542#issuecomment-505789992
                    let mut lookup: [sbor::rust::mem::MaybeUninit<Option<TypeData<$custom_type_kind, LocalTypeIndex>>>; 256] = unsafe {
                        sbor::rust::mem::MaybeUninit::uninit().assume_init()
                    };

                    for elem in &mut lookup[..] {
                        unsafe { sbor::rust::ptr::write(elem.as_mut_ptr(), None); }
                    }

                    unsafe { sbor::rust::mem::transmute::<_, [Option<TypeData<$custom_type_kind, LocalTypeIndex>>; 256]>(lookup) }
                };
                // Now add in the basic types
                lookup[sbor::basic_well_known_types::BOOL_ID as usize] = Some(TypeData::unnamed(TypeKind::Bool));
                lookup[sbor::basic_well_known_types::I8_ID as usize] = Some(TypeData::unnamed(TypeKind::I8));
                lookup[sbor::basic_well_known_types::I16_ID as usize] = Some(TypeData::unnamed(TypeKind::I16));
                lookup[sbor::basic_well_known_types::I32_ID as usize] = Some(TypeData::unnamed(TypeKind::I32));
                lookup[sbor::basic_well_known_types::I64_ID as usize] = Some(TypeData::unnamed(TypeKind::I64));
                lookup[sbor::basic_well_known_types::I128_ID as usize] = Some(TypeData::unnamed(TypeKind::I128));
                lookup[sbor::basic_well_known_types::U8_ID as usize] = Some(TypeData::unnamed(TypeKind::U8));
                lookup[sbor::basic_well_known_types::U16_ID as usize] = Some(TypeData::unnamed(TypeKind::U16));
                lookup[sbor::basic_well_known_types::U32_ID as usize] = Some(TypeData::unnamed(TypeKind::U32));
                lookup[sbor::basic_well_known_types::U64_ID as usize] = Some(TypeData::unnamed(TypeKind::U64));
                lookup[sbor::basic_well_known_types::U128_ID as usize] = Some(TypeData::unnamed(TypeKind::U128));
                lookup[sbor::basic_well_known_types::STRING_ID as usize] = Some(TypeData::unnamed(TypeKind::String));
                lookup[sbor::basic_well_known_types::ANY_ID as usize] = Some(TypeData::unnamed(TypeKind::Any));
                lookup[sbor::basic_well_known_types::UNIT_ID as usize] = Some(TypeData::no_child_names(
                    TypeKind::Tuple {
                        field_types: sbor::rust::prelude::vec![],
                    },
                    "None"
                ));
                lookup[sbor::basic_well_known_types::BYTES_ID as usize] = Some(TypeData::no_child_names(
                    TypeKind::Array {
                        element_type: LocalTypeIndex::WellKnown(sbor::basic_well_known_types::U8_ID),
                    },
                    "Bytes"
                ));
                // And now add in the custom types
                $(lookup[$id as usize] = Some($type_data);)*

                // And return the lookup
                lookup
            };

        }
    };
}
