use sbor::{model::Type, *};

use uint::construct_uint;

use crate::rust::borrow::ToOwned;
use crate::types::*;

construct_uint! {
    pub struct U256(4);
}

impl Encode for U256 {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        let mut bytes = [0u8; 32];
        self.to_little_endian(&mut bytes);
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }

    #[inline]
    fn sbor_type() -> u8 {
        SCRYPTO_TYPE_U256
    }
}

impl Decode for U256 {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        if len == 32 {
            Ok(U256::from_little_endian(slice))
        } else {
            Err(DecodeError::InvalidCustomData(SCRYPTO_TYPE_U256))
        }
    }

    #[inline]
    fn sbor_type() -> u8 {
        SCRYPTO_TYPE_U256
    }
}

impl Describe for U256 {
    fn describe() -> Type {
        Type::Custom {
            name: "U256".to_owned(),
        }
    }
}
