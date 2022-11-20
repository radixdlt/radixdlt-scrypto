use crate::type_id::*;
use crate::*;

macro_rules! encode_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<X: CustomTypeId, Enc: Encoder<X>, $($name: Encode<X, Enc>),+> Encode<X, Enc> for ($($name,)+) {
            #[inline]
            fn encode_type_id(&self, encoder: &mut Enc) -> Result<(), EncodeError> {
                encoder.write_type_id(Self::type_id())
            }

            #[inline]
            fn encode_body(&self, encoder: &mut Enc) -> Result<(), EncodeError> {
                encoder.write_size($n)?;
                $(encoder.encode(&self.$idx)?;)+
                Ok(())
            }
        }
    };
}

encode_tuple! { 2 0 A 1 B }
encode_tuple! { 3 0 A 1 B 2 C }
encode_tuple! { 4 0 A 1 B 2 C 3 D }
encode_tuple! { 5 0 A 1 B 2 C 3 D 4 E }
encode_tuple! { 6 0 A 1 B 2 C 3 D 4 E 5 F }
encode_tuple! { 7 0 A 1 B 2 C 3 D 4 E 5 F 6 G }
encode_tuple! { 8 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H }
encode_tuple! { 9 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I }
encode_tuple! { 10 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J }

macro_rules! decode_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<X: CustomTypeId, Dec: Decoder<X>, $($name: Decode<X, Dec>),+> Decode<X, Dec> for ($($name,)+) {
            #[inline]
            fn decode_body_with_type_id(decoder: &mut Dec, type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
                decoder.check_preloaded_type_id(type_id, Self::type_id())?;
                decoder.read_and_check_size($n)?;

                Ok(($(decoder.decode::<$name>()?),+))
            }
        }
    };
}

decode_tuple! { 2 0 A 1 B }
decode_tuple! { 3 0 A 1 B 2 C }
decode_tuple! { 4 0 A 1 B 2 C 3 D }
decode_tuple! { 5 0 A 1 B 2 C 3 D 4 E }
decode_tuple! { 6 0 A 1 B 2 C 3 D 4 E 5 F }
decode_tuple! { 7 0 A 1 B 2 C 3 D 4 E 5 F 6 G }
decode_tuple! { 8 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H }
decode_tuple! { 9 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I }
decode_tuple! { 10 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J }
