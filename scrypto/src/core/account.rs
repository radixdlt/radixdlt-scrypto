use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::vec;
use crate::types::*;
use crate::utils::*;

/// An account is a component that holds resources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    component: Component,
}

impl<A: Into<Component>> From<A> for Account {
    fn from(a: A) -> Self {
        Self {
            component: a.into(),
        }
    }
}

impl From<Account> for Address {
    fn from(a: Account) -> Address {
        a.component.into()
    }
}

impl Account {
    /// Creates a new account.
    pub fn new() -> Account {
        let rtn = call_function(ACCOUNT_PACKAGE, "Account", "new", vec![]);
        scrypto_unwrap(scrypto_decode(&rtn))
    }

    /// Withdraws resource fromm this account.
    pub fn withdraw<A: Into<ResourceDef>>(&mut self, amount: Decimal, resource_def: A) {
        let args = vec![
            scrypto_encode(&amount),
            scrypto_encode(&resource_def.into()),
        ];
        call_method(self.address(), "withdraw", args);
    }

    /// Deposits resource to this account.
    pub fn deposit(&mut self, bucket: Bucket) {
        let args = vec![scrypto_encode(&bucket)];
        call_method(self.address(), "deposit", args);
    }

    /// Returns the address of this account.
    pub fn address(&self) -> Address {
        self.component.address()
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
        self.component.encode_value(encoder);
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
