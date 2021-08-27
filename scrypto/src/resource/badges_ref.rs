use sbor::model::*;
use sbor::{Decode, Describe, Encode};

use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// A reference to a `Badges` bucket.
#[derive(Debug, Encode, Decode)]
pub struct BadgesRef {
    rid: RID,
}

impl Describe for BadgesRef {
    fn describe() -> Type {
        Type::SystemType {
            name: "::scrypto::resource::BadgesRef".to_owned(),
        }
    }
}

impl From<RID> for BadgesRef {
    fn from(rid: RID) -> Self {
        Self { rid }
    }
}

impl BadgesRef {
    pub fn check(&self, resource: Address) {
        assert!(self.resource() == resource && self.amount() >= 1.into());
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
