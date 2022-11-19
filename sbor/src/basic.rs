use crate::*;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // See https://serde.rs/enum-representations.html
)]
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum NoCustomTypeId {}

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // See https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoCustomValue {}

pub type BasicEncoder<'a> = Encoder<'a, NoCustomTypeId>;
pub type BasicDecoder<'a> = DefaultVecDecoder<'a, NoCustomTypeId>;
pub type BasicSborValue = SborValue<NoCustomTypeId, NoCustomValue>;
pub type BasicSborTypeId = SborTypeId<NoCustomTypeId>;

impl CustomTypeId for NoCustomTypeId {
    fn as_u8(&self) -> u8 {
        panic!("No custom type")
    }

    fn from_u8(_id: u8) -> Option<Self> {
        panic!("No custom type")
    }
}

impl<X: CustomTypeId> Encode<X> for NoCustomValue {
    fn encode_type_id(&self, _encoder: &mut Encoder<X>) {
        panic!("No custom value")
    }

    fn encode_body(&self, _encoder: &mut Encoder<X>) {
        panic!("No custom value")
    }
}

impl<X: CustomTypeId, D: Decoder<X>> Decode<X, D> for NoCustomValue {
    fn decode_body_with_type_id(
        _decoder: &mut D,
        _type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError>
    where
        Self: Sized,
    {
        panic!("No custom value")
    }
}
