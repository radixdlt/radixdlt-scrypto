use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::constructs::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::vec;
use crate::types::*;

/// An account is a component that holds user's resource.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct Account {
    address: Address,
}

impl From<Address> for Account {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl From<Account> for Address {
    fn from(a: Account) -> Address {
        a.address
    }
}

impl Account {
    pub fn deposit(&self, bucket: Bucket) {
        let component = Component::from(self.address());
        let args = vec![scrypto_encode(&bucket)];
        component.call::<()>("deposit", args);
    }

    pub fn address(&self) -> Address {
        self.address
    }
}

impl Describe for Account {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_ACCOUNT.to_owned(),
        }
    }
}
