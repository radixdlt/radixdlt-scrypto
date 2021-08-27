use sbor::model::*;
use sbor::{Decode, Describe, Encode};

use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// A reference to a `Tokens` bucket.
#[derive(Debug, Encode, Decode)]
pub struct TokensRef {
    rid: RID,
}

impl Describe for TokensRef {
    fn describe() -> Type {
        Type::SystemType {
            name: "::scrypto::resource::TokensRef".to_owned(),
        }
    }
}

impl From<RID> for TokensRef {
    fn from(rid: RID) -> Self {
        Self { rid }
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
