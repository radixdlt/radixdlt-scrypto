use sbor::{describe::Type, *};

use crate::constants::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// A reference to a `Tokens` bucket.
#[derive(Debug, Encode, Decode)]
pub struct TokensRef {
    rid: RID,
}

impl From<RID> for TokensRef {
    fn from(rid: RID) -> Self {
        Self { rid }
    }
}

impl From<TokensRef> for RID {
    fn from(a: TokensRef) -> RID {
        a.rid
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

impl Describe for TokensRef {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_TOKENS_REF.to_owned(),
        }
    }
}
