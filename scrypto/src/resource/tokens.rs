use sbor::{describe::Type, *};

use crate::constants::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// A bucket that holds tokens.
#[derive(Debug)]
pub struct Tokens {
    bid: BID,
}

impl From<BID> for Tokens {
    fn from(bid: BID) -> Self {
        Self { bid }
    }
}

impl Into<BID> for Tokens {
    fn into(self) -> BID {
        self.bid
    }
}

impl Tokens {
    pub fn check(&self, resource: Address) {
        assert!(self.resource() == resource);
    }

    pub fn new_empty(resource: Address) -> Self {
        BID::new_empty(resource).into()
    }

    pub fn put(&self, other: Self) {
        self.bid.put(other.bid);
    }

    pub fn take(&self, amount: U256) -> Self {
        self.bid.take(amount).into()
    }

    pub fn borrow(&self) -> TokensRef {
        self.bid.borrow().into()
    }

    pub fn amount(&self) -> U256 {
        self.bid.amount()
    }

    pub fn resource(&self) -> Address {
        self.bid.resource()
    }
}

impl Encode for Tokens {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.bid.encode_value(encoder);
    }

    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_TOKENS
    }
}

impl Decode for Tokens {
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        let bid = BID::decode_value(decoder)?;
        Ok(bid.into())
    }

    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_TOKENS
    }
}

impl Describe for Tokens {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_TOKENS.to_owned(),
        }
    }
}
