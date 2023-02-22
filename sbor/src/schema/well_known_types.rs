pub const CUSTOM_WELL_KNOWN_TYPE_START: u8 = 0x80;

pub mod basic_well_known_types {
    use sbor::*;

    // These ids must be usable in a const context
    pub const ANY_ID: u8 = 0x40;

    pub const UNIT_ID: u8 = 0x00;
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

    pub const BYTES_ID: u8 = 0x41;
}

#[macro_export]
macro_rules! well_known_type_mapping {
    ($well_known_index: expr, [$($id: path, $type_name: path),*]) => {{
        let type_data = match $well_known_index {
            sbor::basic_well_known_types::ANY_ID => ANY_TYPE,
            sbor::basic_well_known_types::UNIT_ID => UNIT_TYPE,
            sbor::basic_well_known_types::BOOL_ID => BOOL_TYPE,
            sbor::basic_well_known_types::I8_ID => I8_TYPE,
            sbor::basic_well_known_types::I16_ID => I16_TYPE,
            sbor::basic_well_known_types::I32_ID => I32_TYPE,
            sbor::basic_well_known_types::I64_ID => I64_TYPE,
            sbor::basic_well_known_types::I128_ID => I128_TYPE,
            sbor::basic_well_known_types::U8_ID => U8_TYPE,
            sbor::basic_well_known_types::U16_ID => U16_TYPE,
            sbor::basic_well_known_types::U32_ID => U32_TYPE,
            sbor::basic_well_known_types::U64_ID => U64_TYPE,
            sbor::basic_well_known_types::U128_ID => U128_TYPE,
            sbor::basic_well_known_types::STRING_ID => STRING_TYPE,
            sbor::basic_well_known_types::BYTES_ID => BYTES_TYPE,
            $($id => $type_name,)*
            _ => return None,
        };
        Some(&type_data)
    }};
}

#[macro_export]
macro_rules! create_well_known_lookup {
    ($lookup_name: ident, $custom_type_kind: ty, [$($id: path, $type_data: path),*]) => {
        static $lookup_name: once_cell::sync::Lazy<[Option<TypeData<$custom_type_kind, LocalTypeIndex>>; 255]> = once_cell::sync::Lazy::new(|| {
            let mut lookup = {
                // Initialize the array with None, following the example here:
                // https://github.com/rust-lang/rust/issues/54542#issuecomment-505789992
                let mut lookup: [sbor::rust::mem::MaybeUninit<Option<TypeData<$custom_type_kind, LocalTypeIndex>>>; 255] = unsafe {
                    sbor::rust::mem::MaybeUninit::uninit().assume_init()
                };
            
                for elem in &mut lookup[..] {
                    unsafe { sbor::rust::ptr::write(elem.as_mut_ptr(), None); }
                }
            
                unsafe { sbor::rust::mem::transmute::<_, [Option<TypeData<$custom_type_kind, LocalTypeIndex>>; 255]>(lookup) }
            };
            // Now add in the basic types
            lookup[sbor::basic_well_known_types::ANY_ID as usize] = Some(TypeData::named_no_child_names("Any", TypeKind::Any));
            lookup[sbor::basic_well_known_types::UNIT_ID as usize] = Some(TypeData::named_no_child_names(
                "-",
                TypeKind::Tuple {
                    field_types: sbor::rust::vec::Vec::new(),
                },
            ));
            lookup[sbor::basic_well_known_types::BOOL_ID as usize] = Some(TypeData::named_no_child_names("Bool", TypeKind::Bool));
            lookup[sbor::basic_well_known_types::I8_ID as usize] = Some(TypeData::named_no_child_names("I8", TypeKind::I8));
            lookup[sbor::basic_well_known_types::I16_ID as usize] = Some(TypeData::named_no_child_names("I16", TypeKind::I16));
            lookup[sbor::basic_well_known_types::I32_ID as usize] = Some(TypeData::named_no_child_names("I32", TypeKind::I32));
            lookup[sbor::basic_well_known_types::I64_ID as usize] = Some(TypeData::named_no_child_names("I64", TypeKind::I64));
            lookup[sbor::basic_well_known_types::I128_ID as usize] = Some(TypeData::named_no_child_names("I128", TypeKind::I128));
            lookup[sbor::basic_well_known_types::U8_ID as usize] = Some(TypeData::named_no_child_names("U8", TypeKind::U8));
            lookup[sbor::basic_well_known_types::U16_ID as usize] = Some(TypeData::named_no_child_names("U16", TypeKind::U16));
            lookup[sbor::basic_well_known_types::U32_ID as usize] = Some(TypeData::named_no_child_names("U32", TypeKind::U32));
            lookup[sbor::basic_well_known_types::U64_ID as usize] = Some(TypeData::named_no_child_names("U64", TypeKind::U64));
            lookup[sbor::basic_well_known_types::U128_ID as usize] = Some(TypeData::named_no_child_names("U128", TypeKind::U128));
            lookup[sbor::basic_well_known_types::STRING_ID as usize] = Some(TypeData::named_no_child_names("String", TypeKind::String));
            lookup[sbor::basic_well_known_types::BYTES_ID as usize] = Some(TypeData::named_no_child_names(
                "Bytes",
                TypeKind::Array {
                    element_type: LocalTypeIndex::WellKnown(sbor::basic_well_known_types::U8_ID),
                },
            ));
            // And now add in the custom types
            $(lookup[$id as usize] => Some($type_data);)*

            // And return the lookup
            lookup
        });
    };
}
