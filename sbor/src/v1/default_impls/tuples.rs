use super::super::*;

macro_rules! encode_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<$($name: Interpretation),+> Interpretation for ($($name,)+) {
            const INTERPRETATION: u8 = DefaultInterpretations::TUPLE;
        }

        impl<Enc: Encoder, $($name: Encode<Enc> + Interpretation),+> Encode<Enc> for ($($name,)+) {
            fn encode_value(&self, encoder: &mut Enc) -> Result<(), EncodeError> {
                encoder.write_product_type_header_u8_length($n)?;
                $(encoder.encode(&self.$idx)?;)+
                Ok(())
            }
        }

        impl <Dec: Decoder, $($name: Decode<Dec> + Interpretation),+> Decode<Dec> for ($($name,)+) {
            fn decode_value(decoder: &mut Dec) -> Result<Self, DecodeError> {
                decoder.read_product_type_header_u8_length($n)?;
                Ok(($(decoder.decode::<$name>()?,)+))
            }
        }
    };
}

encode_tuple! { 1 0 A }
encode_tuple! { 2 0 A 1 B }
encode_tuple! { 3 0 A 1 B 2 C }
encode_tuple! { 4 0 A 1 B 2 C 3 D }
encode_tuple! { 5 0 A 1 B 2 C 3 D 4 E }
encode_tuple! { 6 0 A 1 B 2 C 3 D 4 E 5 F }
encode_tuple! { 7 0 A 1 B 2 C 3 D 4 E 5 F 6 G }
encode_tuple! { 8 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H }
encode_tuple! { 9 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I }
encode_tuple! { 10 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J }
encode_tuple! { 11 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K }
encode_tuple! { 12 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J 10 K 11 L }