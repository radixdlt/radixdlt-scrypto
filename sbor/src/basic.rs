use crate::*;

pub type BasicSborValue = SborValue<NoCustomValue>;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // For JSON readability, see https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoCustomValue {}

impl CustomValue for NoCustomValue {
    fn encode_type_id(&self, _encoder: &mut Encoder) {
        panic!("No custom value")
    }

    fn encode_value(&self, _encoder: &mut Encoder) {
        panic!("No custom value")
    }

    fn decode(_decoder: &mut Decoder, type_id: SborTypeId) -> Result<Self, DecodeError>
    where
        Self: Sized,
    {
        Err(DecodeError::UnknownTypeId(type_id.id()))
    }
}
