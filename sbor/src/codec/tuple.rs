use crate::value_kind::*;
use crate::*;

macro_rules! categorize_tuple {
    ($n:tt$( $idx:tt $name:ident)*) => {
        impl<X: CustomValueKind$(, $name)*> Categorize<X> for ($($name,)*) {
            #[inline]
            fn value_kind() -> ValueKind<X> {
                ValueKind::Tuple
            }
        }
    };
}

categorize_tuple! { 0 } // Unit
categorize_tuple! { 1 0 A }
categorize_tuple! { 2 0 A 1 B }
categorize_tuple! { 3 0 A 1 B 2 C }
categorize_tuple! { 4 0 A 1 B 2 C 3 D }
categorize_tuple! { 5 0 A 1 B 2 C 3 D 4 E }
categorize_tuple! { 6 0 A 1 B 2 C 3 D 4 E 5 F }
categorize_tuple! { 7 0 A 1 B 2 C 3 D 4 E 5 F 6 G }
categorize_tuple! { 8 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H }
categorize_tuple! { 9 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I }
categorize_tuple! { 10 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J }
categorize_tuple! { 11 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K  }
categorize_tuple! { 12 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L   }
categorize_tuple! { 13 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M  }
categorize_tuple! { 14 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N  }
categorize_tuple! { 15 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N 14 O  }
categorize_tuple! { 16 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N 14 O 15 P  }
categorize_tuple! { 17 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N 14 O 15 P 16 Q   }
categorize_tuple! { 18 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N 14 O 15 P 16 Q 17 R  }
categorize_tuple! { 19 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N 14 O 15 P 16 Q 17 R 18 S  }
categorize_tuple! { 20 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L 12 M 13 N 14 O 15 P 16 Q 17 R 18 S 19 T  }

macro_rules! encode_tuple {
    ($n:tt$( $idx:tt $name:ident)*) => {
        impl<X: CustomValueKind, E: Encoder<X>$(, $name: Encode<X, E>)*> Encode<X, E> for ($($name,)*) {
            #[inline]
            fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
                encoder.write_value_kind(Self::value_kind())
            }

            #[inline]
            fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
                encoder.write_size($n)?;
                $(encoder.encode(&self.$idx)?;)*
                Ok(())
            }
        }
    };
}

encode_tuple! { 0 } // Unit
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
encode_tuple! { 11 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 }
encode_tuple! { 12 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 }
encode_tuple! { 13 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 }
encode_tuple! { 14 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 }
encode_tuple! { 15 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 }
encode_tuple! { 16 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 }
encode_tuple! { 17 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 }
encode_tuple! { 18 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 }
encode_tuple! { 19 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 18 T18 }
encode_tuple! { 20 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 18 T18 19 T19 }

macro_rules! decode_tuple {
    ($n:tt$( $idx:tt $name:ident)*) => {
        impl<X: CustomValueKind, D: Decoder<X>$(, $name: Decode<X, D>)*> Decode<X, D> for ($($name,)*) {
            #[inline]
            fn decode_body_with_value_kind(decoder: &mut D, value_kind: ValueKind<X>) -> Result<Self, DecodeError> {
                decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
                decoder.read_and_check_size($n)?;

                Ok(($(decoder.decode::<$name>()?,)*))
            }
        }
    };
}

decode_tuple! { 0 } // Unit
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
decode_tuple! { 11 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 }
decode_tuple! { 12 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 }
decode_tuple! { 13 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 }
decode_tuple! { 14 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 }
decode_tuple! { 15 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 }
decode_tuple! { 16 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 }
decode_tuple! { 17 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 }
decode_tuple! { 18 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 }
decode_tuple! { 19 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 18 T18 }
decode_tuple! { 20 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 18 T18 19 T19 }

pub use schema::*;

mod schema {
    use super::*;
    macro_rules! describe_tuple {
        ($n:tt$( $idx:tt $name:ident)*) => {
            impl<C: CustomTypeKind<GlobalTypeId>$(, $name: Describe<C>)*> Describe<C> for ($($name,)*) {
                const TYPE_ID: GlobalTypeId = GlobalTypeId::novel("Tuple", &[$($name::TYPE_ID),*]);

                fn type_data() -> Option<TypeData<C, GlobalTypeId>> {
                    Some(TypeData::unnamed(
                        TypeKind::Tuple {
                            field_types: crate::rust::vec![
                                $($name::TYPE_ID,)*
                            ]
                        }
                    ))
                }

                #[allow(unused_variables)] // For the unit case
                fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {
                    $(aggregator.add_child_type_and_descendents::<$name>();)*
                }
            }
        };
    }

    describe_basic_well_known_type!((), UNIT_ID);
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
    describe_tuple! { 11 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 }
    describe_tuple! { 12 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 }
    describe_tuple! { 13 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 }
    describe_tuple! { 14 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 }
    describe_tuple! { 15 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 }
    describe_tuple! { 16 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 }
    describe_tuple! { 17 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 }
    describe_tuple! { 18 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 }
    describe_tuple! { 19 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 18 T18 }
    describe_tuple! { 20 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 18 T18 19 T19 }
}
