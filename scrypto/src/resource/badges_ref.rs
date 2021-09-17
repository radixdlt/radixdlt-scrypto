use sbor::{describe::Type, *};

use crate::constants::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// A reference to a `Badges` bucket.
#[derive(Debug, Encode, Decode)]
pub struct BadgesRef {
    rid: RID,
}

impl From<RID> for BadgesRef {
    fn from(rid: RID) -> Self {
        Self { rid }
    }
}

impl From<BadgesRef> for RID {
    fn from(a: BadgesRef) -> RID {
        a.rid
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

    pub fn drop(self) {
        self.rid.drop()
    }
}

impl Describe for BadgesRef {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_BADGES_REF.to_owned(),
        }
    }
}
