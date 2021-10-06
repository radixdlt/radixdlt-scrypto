use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::vec;
use crate::types::*;

/// An account is a component that holds resources.
#[derive(Debug, PartialEq, Eq, TypeId, Encode, Decode)]
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
    pub fn withdraw<A: Into<Address>>(&self, amount: Amount, resource_def: A) {
        let args = vec![
            scrypto_encode(&amount),
            scrypto_encode(&resource_def.into()),
        ];
        call_method(self.address(), "withdraw", args);
    }

    pub fn deposit(&self, bucket: Bucket) {
        let args = vec![scrypto_encode(&bucket)];
        call_method(self.address(), "deposit", args);
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
