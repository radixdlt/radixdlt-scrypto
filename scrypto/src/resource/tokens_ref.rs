use sbor::{describe::Type, *};

use crate::constants::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// A reference to a `Tokens` bucket.
#[derive(Debug)]
pub struct TokensRef {
    rid: RID,
}

impl From<RID> for TokensRef {
    fn from(rid: RID) -> Self {
        Self { rid }
    }
}

impl Into<RID> for TokensRef {
    fn into(self) -> RID {
        self.rid
    }
}

impl TokensRef {
    pub fn check(&self, resource: Address) {
        assert!(self.resource() == resource);
    }

    pub fn amount(&self) -> U256 {
        self.rid.amount()
    }

    pub fn resource(&self) -> Address {
        self.rid.resource()
    }

    pub fn destroy(self) {
        self.rid.destroy()
    }
}

impl Encode for TokensRef {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        self.rid.encode_value(encoder);
    }

    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_TOKENS_REF
    }
}

impl Decode for TokensRef {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        let rid = RID::decode_value(decoder)?;
        Ok(rid.into())
    }

    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_TOKENS_REF
    }
}

impl Describe for TokensRef {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_TOKENS_REF.to_owned(),
        }
    }
}
