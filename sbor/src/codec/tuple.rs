use crate::type_id::*;
use crate::*;
use sbor::rust::vec;

macro_rules! encode_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<X: CustomTypeId, E: Encoder<X>, $($name: Encode<X, E>),+> Encode<X, E> for ($($name,)+) {
            #[inline]
            fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
                encoder.write_type_id(Self::type_id())
            }

            #[inline]
            fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
                encoder.write_size($n)?;
                $(encoder.encode(&self.$idx)?;)+
                Ok(())
            }
        }
    };
}

encode_tuple! { 1 0 T0 }
encode_tuple! { 2 0 T0 1 T1 }
encode_tuple! { 3 0 T0 1 T1 2 T2 }
encode_tuple! { 4 0 T0 1 T1 2 T2 3 T3 }
encode_tuple! { 5 0 T0 1 T1 2 T2 3 T3 4 T4 }
encode_tuple! { 6 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 }
encode_tuple! { 7 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 }
encode_tuple! { 8 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 }
encode_tuple! { 9 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 }
encode_tuple! { 10 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 }

macro_rules! decode_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<X: CustomTypeId, D: Decoder<X>, $($name: Decode<X, D>),+> Decode<X, D> for ($($name,)+) {
            #[inline]
            fn decode_body_with_type_id(decoder: &mut D, type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
                decoder.check_preloaded_type_id(type_id, Self::type_id())?;
                decoder.read_and_check_size($n)?;

                Ok(($(decoder.decode::<$name>()?,)+))
            }
        }
    };
}

decode_tuple! { 1 0 T0 }
decode_tuple! { 2 0 T0 1 T1 }
decode_tuple! { 3 0 T0 1 T1 2 T2 }
decode_tuple! { 4 0 T0 1 T1 2 T2 3 T3 }
decode_tuple! { 5 0 T0 1 T1 2 T2 3 T3 4 T4 }
decode_tuple! { 6 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 }
decode_tuple! { 7 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 }
decode_tuple! { 8 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 }
decode_tuple! { 9 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 }
decode_tuple! { 10 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 }

#[cfg(feature = "schema")]
pub use schema::*;

#[cfg(feature = "schema")]
mod schema {
    use super::*;
    macro_rules! tuple_schema {
        ($n:tt $($idx:tt $name:ident)+) => {
            impl<C: CustomTypeSchema, $($name: Schema<C>),+> Schema<C> for ($($name,)+) {
                const SCHEMA_TYPE_REF: GlobalTypeRef = GlobalTypeRef::complex("Tuple", &[$($name::SCHEMA_TYPE_REF, )+]);

                fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
                    Some(LocalTypeData {
                        schema: TypeSchema::Tuple {
                            field_types: vec![
                                $($name::SCHEMA_TYPE_REF,)+
                            ],
                        },
                        naming: TypeNaming::named_no_child_names("Tuple"),
                    })
                }

                fn add_all_dependencies(aggregator: &mut SchemaAggregator<C>) {
                    $(aggregator.add_child_type_and_descendents::<$name>();)+
                }
            }
        };
    }

    tuple_schema! { 1 0 T0 }
    tuple_schema! { 2 0 T0 1 T1 }
    tuple_schema! { 3 0 T0 1 T1 2 T2 }
    tuple_schema! { 4 0 T0 1 T1 2 T2 3 T3 }
    tuple_schema! { 5 0 T0 1 T1 2 T2 3 T3 4 T4 }
    tuple_schema! { 6 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 }
    tuple_schema! { 7 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 }
    tuple_schema! { 8 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 }
    tuple_schema! { 9 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 }
    tuple_schema! { 10 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 }
}
