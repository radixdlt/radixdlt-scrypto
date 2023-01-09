use crate::type_id::*;
use crate::*;

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
    macro_rules! describe_tuple {
        ($n:tt $($idx:tt $name:ident)+) => {
            impl<C: CustomTypeKind<GlobalTypeId>, $($name: Describe<C>),+> Describe<C> for ($($name,)+) {
                const TYPE_ID: GlobalTypeId = GlobalTypeId::complex("Tuple", &[$($name::TYPE_ID, )+]);

                fn type_data() -> Option<TypeData<C, GlobalTypeId>> {
                    Some(TypeData::named_tuple("Tuple", vec![
                        $($name::TYPE_ID,)+
                    ]))
                }

                fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
                    $(aggregator.add_child_type_and_descendents::<$name>();)+
                }
            }
        };
    }

    describe_tuple! { 1 0 T0 }
    describe_tuple! { 2 0 T0 1 T1 }
    describe_tuple! { 3 0 T0 1 T1 2 T2 }
    describe_tuple! { 4 0 T0 1 T1 2 T2 3 T3 }
    describe_tuple! { 5 0 T0 1 T1 2 T2 3 T3 4 T4 }
    describe_tuple! { 6 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 }
    describe_tuple! { 7 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 }
    describe_tuple! { 8 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 }
    describe_tuple! { 9 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 }
    describe_tuple! { 10 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 }
}
