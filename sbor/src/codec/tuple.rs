use crate::type_id::*;
use crate::*;

macro_rules! encode_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<X: CustomTypeId, $($name: Encode<X>),+> Encode<X> for ($($name,)+) {
            #[inline]
            fn encode_type_id(&self, encoder: &mut Encoder<X>) {
                encoder.write_type_id(Self::type_id());
            }
            #[inline]
            fn encode_body(&self, encoder: &mut Encoder<X>) {
                encoder.write_size($n);

                $(self.$idx.encode(encoder);)+
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
        impl<X: CustomTypeId, $($name: Decode<X>),+> Decode<X> for ($($name,)+) {
            fn decode_with_type_id(decoder: &mut Decoder<X>, type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
                decoder.check_preloaded_type_id(type_id, Self::type_id())?;
                decoder.check_size($n)?;

                Ok(($($name::decode(decoder)?),+))
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
