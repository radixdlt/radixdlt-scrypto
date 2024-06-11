pub mod basic_well_known_types {
    use sbor::rust::prelude::*;
    use sbor::*;

    pub const BOOL_TYPE: WellKnownTypeId = WellKnownTypeId::of(VALUE_KIND_BOOL);
    pub fn bool_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::Bool)
    }

    pub const I8_TYPE: WellKnownTypeId = WellKnownTypeId::of(VALUE_KIND_I8);
    pub fn i8_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::I8)
    }

    pub const I16_TYPE: WellKnownTypeId = WellKnownTypeId::of(VALUE_KIND_I16);
    pub fn i16_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::I16)
    }

    pub const I32_TYPE: WellKnownTypeId = WellKnownTypeId::of(VALUE_KIND_I32);
    pub fn i32_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::I32)
    }

    pub const I64_TYPE: WellKnownTypeId = WellKnownTypeId::of(VALUE_KIND_I64);
    pub fn i64_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::I64)
    }

    pub const I128_TYPE: WellKnownTypeId = WellKnownTypeId::of(VALUE_KIND_I128);
    pub fn i128_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::I128)
    }

    pub const U8_TYPE: WellKnownTypeId = WellKnownTypeId::of(VALUE_KIND_U8);
    pub fn u8_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::U8)
    }

    pub const U16_TYPE: WellKnownTypeId = WellKnownTypeId::of(VALUE_KIND_U16);
    pub fn u16_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::U16)
    }

    pub const U32_TYPE: WellKnownTypeId = WellKnownTypeId::of(VALUE_KIND_U32);
    pub fn u32_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::U32)
    }

    pub const U64_TYPE: WellKnownTypeId = WellKnownTypeId::of(VALUE_KIND_U64);
    pub fn u64_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::U64)
    }

    pub const U128_TYPE: WellKnownTypeId = WellKnownTypeId::of(VALUE_KIND_U128);
    pub fn u128_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::U128)
    }

    pub const STRING_TYPE: WellKnownTypeId = WellKnownTypeId::of(VALUE_KIND_STRING);
    pub fn string_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::String)
    }

    pub const ANY_TYPE: WellKnownTypeId = WellKnownTypeId::of(0x40); // Any type
    pub fn any_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::Any)
    }

    pub const BYTES_TYPE: WellKnownTypeId = WellKnownTypeId::of(0x41); // `Vec<u8>`
    pub fn bytes_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::Array {
            element_type: U8_TYPE.into(),
        })
    }

    pub const UNIT_TYPE: WellKnownTypeId = WellKnownTypeId::of(0x42); // `()`
    pub fn unit_type_data<C: CustomTypeKind<L>, L: SchemaTypeLink>() -> TypeData<C, L> {
        TypeData::unnamed(TypeKind::Tuple {
            field_types: vec![],
        })
    }
}

#[macro_export]
macro_rules! create_well_known_lookup {
    (
        $lookup_name: ident,
        $constants_mod: ident,
        $custom_type_kind: ty,
        [
            $((
                $(#[$meta:meta])*
                $name: ident,
                $type_id: expr,
                $type_data: expr$(,)?
            )),*
            $(,)?
        ]
    ) => {
        paste::paste! {
            pub mod $constants_mod {
                #[allow(unused_imports)]
                use super::*;

                $(
                    $(#[$meta])*
                    pub const [<$name:upper _TYPE>]: WellKnownTypeId = WellKnownTypeId::of($type_id);

                    pub fn [<$name:lower _type_data>]<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
                        $type_data
                    }
                )*
            }

            const [<$lookup_name:upper _INIT>]: Option<TypeData<$custom_type_kind, LocalTypeId>> = None;

            lazy_static::lazy_static! {
                static ref $lookup_name: [Option<TypeData<$custom_type_kind, LocalTypeId>>; 256] = {
                    use sbor::basic_well_known_types::*;

                    // Note - whilst WellKnownTypeId is u16, we don't yet use any index > 256 - so this should fit all
                    // existing well known types.
                    // If we exceed 256, we'll get a panic when this loads and tests will fail.
                    let mut lookup: [Option<TypeData<$custom_type_kind, LocalTypeId>>; 256] = [ [<$lookup_name:upper _INIT>]; 256 ];

                    let mut add = |type_id: WellKnownTypeId, type_data: TypeData<$custom_type_kind, LocalTypeId>| {
                        let index = type_id.as_index();
                        let entry = lookup.get(index).expect("Well known type index larger than lookup size");
                        if entry.is_some() {
                            panic!("Duplicate well known type index: {:?}", type_id);
                        }
                        lookup[index] = Some(type_data);
                    };

                    // Now add in the basic types
                    add(BOOL_TYPE, bool_type_data());
                    add(I8_TYPE, i8_type_data());
                    add(I16_TYPE, i16_type_data());
                    add(I32_TYPE, i32_type_data());
                    add(I64_TYPE, i64_type_data());
                    add(I128_TYPE, i128_type_data());
                    add(U8_TYPE, u8_type_data());
                    add(U16_TYPE, u16_type_data());
                    add(U32_TYPE, u32_type_data());
                    add(U64_TYPE, u64_type_data());
                    add(U128_TYPE, u128_type_data());
                    add(STRING_TYPE, string_type_data());
                    add(ANY_TYPE, any_type_data());
                    add(BYTES_TYPE, bytes_type_data());
                    add(UNIT_TYPE, unit_type_data());

                    // And now add in the custom types
                    $(add($constants_mod::[<$name:upper _TYPE>],$constants_mod::[<$name:lower _type_data>]());)*

                    // And return the lookup
                    lookup
                };
            }
        }
    };
}
