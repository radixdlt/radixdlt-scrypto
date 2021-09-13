use sbor::{describe::Type, *};

use uint::construct_uint;

use crate::constants::*;
use crate::rust::borrow::ToOwned;

construct_uint! {
    pub struct U256(4);
}

impl Encode for U256 {
    fn encode_value(&self, encoder: &mut Encoder) {
        let mut bytes = [0u8; 32];
        self.to_little_endian(&mut bytes);
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }

    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_U256
    }
}

impl Decode for U256 {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        if len == 32 {
            Ok(U256::from_little_endian(slice))
        } else {
            Err(DecodeError::InvalidCustomData(SCRYPTO_TYPE_U256))
        }
    }

    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_U256
    }
}

impl Describe for U256 {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_U256.to_owned(),
        }
    }
}
