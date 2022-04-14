use sbor::*;
use scrypto::engine::types::*;
use scrypto::resource::Authorization;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::values::*;

use crate::model::{convert, MethodAuthorization};

/// A component is an instance of blueprint.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct Component {
    package_address: PackageAddress,
    blueprint_name: String,
    auths: Vec<Authorization>,
    state: Vec<u8>,
}

impl Component {
    pub fn new(
        package_address: PackageAddress,
        blueprint_name: String,
        method_auth: Vec<Authorization>,
        state: Vec<u8>,
    ) -> Self {
        Self {
            package_address,
            blueprint_name,
            auths: method_auth,
            state,
        }
    }

    pub fn method_authorization(
        &self,
        schema: &Type,
        method_name: &str,
    ) -> (ScryptoValue, Vec<MethodAuthorization>) {
        let data = ScryptoValue::from_slice(&self.state).unwrap();

        let mut authorizations = Vec::new();
        for auth in &self.auths {
            let method_auth = auth.get(method_name);
            let authorization = convert(schema, &data.dom, method_auth);
            authorizations.push(authorization);
        }

        (data, authorizations)
    }

    pub fn authorization(&self) -> &[Authorization] {
        &self.auths
    }

    pub fn package_address(&self) -> PackageAddress {
        self.package_address.clone()
    }

    pub fn blueprint_name(&self) -> &str {
        &self.blueprint_name
    }

    pub fn state(&self) -> &[u8] {
        &self.state
    }

    pub fn set_state(&mut self, new_state: Vec<u8>) {
        self.state = new_state;
    }
}
