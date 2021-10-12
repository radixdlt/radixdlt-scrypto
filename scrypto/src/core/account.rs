use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::vec;
use crate::types::*;
use crate::utils::*;

/// An account is a component that holds resources.
#[derive(Debug, PartialEq, Eq)]
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
    pub fn new() -> Account {
        let rtn = call_function(ACCOUNT_PACKAGE, "Account", "new", vec![]);
        unwrap_light(scrypto_decode(&rtn))
    }

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

//========
// SBOR
//========

impl TypeId for Account {
    fn type_id() -> u8 {
        Address::type_id()
    }
}

impl Encode for Account {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.address.encode_value(encoder);
    }
}

impl Decode for Account {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Address::decode_value(decoder).map(Into::into)
    }
}

impl Describe for Account {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_ACCOUNT.to_owned(),
            generics: vec![],
        }
    }
}
