use sbor::{describe::Type, *};

use crate::constants::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// A bucket that holds badges.
#[derive(Debug)]
pub struct Badges {
    bid: BID,
}

impl From<BID> for Badges {
    fn from(bid: BID) -> Self {
        Self { bid }
    }
}

impl Into<BID> for Badges {
    fn into(self) -> BID {
        self.bid
    }
}

impl Badges {
    pub fn check(&self, resource: Address) {
        assert!(self.resource() == resource && self.amount() >= 1.into());
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

    pub fn borrow(&self) -> BadgesRef {
        self.bid.borrow().into()
    }

    pub fn amount(&self) -> U256 {
        self.bid.amount()
    }

    pub fn resource(&self) -> Address {
        self.bid.resource()
    }
}

impl Encode for Badges {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        self.bid.encode_value(encoder);
    }

    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_BADGES
    }
}

impl Decode for Badges {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        let bid = BID::decode_value(decoder)?;
        Ok(bid.into())
    }

    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_BADGES
    }
}

impl Describe for Badges {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_BADGES.to_owned(),
        }
    }
}
