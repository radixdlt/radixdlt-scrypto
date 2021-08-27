use sbor::model::*;
use sbor::{Decode, Describe, Encode};

use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// A bucket that holds badges.
#[derive(Debug, Encode, Decode)]
pub struct Badges {
    bid: BID,
}

impl Describe for Badges {
    fn describe() -> Type {
        Type::SystemType {
            name: "::scrypto::resource::Badges".to_owned(),
        }
    }
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

    pub fn put(&mut self, other: Self) {
        self.bid.put(other.bid);
    }

    pub fn take(&mut self, amount: U256) -> Self {
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
