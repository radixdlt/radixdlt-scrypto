use crate::encoder::*;
use crate::type_id::*;

/// A data structure that can be serialized into a byte array using SBOR.
pub trait Encode<X: CustomTypeId, E: Encoder<X>> {
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError>;
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError>;
}
