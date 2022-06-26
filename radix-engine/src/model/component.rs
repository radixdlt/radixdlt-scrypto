use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::component::ComponentAddAccessCheckInput;
use scrypto::engine::types::*;
use scrypto::resource::AccessRules;
use scrypto::values::*;
use crate::engine::SystemApi;

use crate::model::{convert, MethodAuthorization};
use crate::wasm::{WasmEngine, WasmInstance};

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentError {
    InvalidRequestData(DecodeError),
    MethodNotFound,
}

/// A component is an instance of blueprint.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Component {
    package_address: PackageAddress,
    blueprint_name: String,
    access_rules: Vec<AccessRules>,
    state: Vec<u8>,
}

impl Component {
    pub fn new(
        package_address: PackageAddress,
        blueprint_name: String,
        access_rules: Vec<AccessRules>,
        state: Vec<u8>,
    ) -> Self {
        Self {
            package_address,
            blueprint_name,
            access_rules,
            state,
        }
    }

    pub fn method_authorization(
        &self,
        schema: &Type,
        method_name: &str,
    ) -> Vec<MethodAuthorization> {
        let data = ScryptoValue::from_slice(&self.state).unwrap();

        let mut authorizations = Vec::new();
        for auth in &self.access_rules {
            let method_auth = auth.get(method_name);
            let authorization = convert(schema, &data.dom, method_auth);
            authorizations.push(authorization);
        }

        authorizations
    }

    pub fn authorization(&self) -> &[AccessRules] {
        &self.access_rules
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

    pub fn main<'borrowed, S: SystemApi<'borrowed, W, I>, W: WasmEngine<I>, I: WasmInstance>(
        value_id: ValueId,
        fn_ident: &str,
        arg: ScryptoValue,
        system_api: &mut S,
    ) -> Result<ScryptoValue, ComponentError> {
        let mut ref_mut = system_api.borrow_native_value(&value_id);
        let component = ref_mut.component();

        let rtn = match fn_ident {
            "add_access_check" => {
                let input: ComponentAddAccessCheckInput =
                    scrypto_decode(&arg.raw).map_err(|e| ComponentError::InvalidRequestData(e))?;

                // Abi checks
                /*
                let package = self
                    .track
                    .borrow_global_value(component.package_address())
                    .unwrap()
                    .package();
                let blueprint_abi = package.blueprint_abi(component.blueprint_name()).unwrap();
                for access_rules in &access_rules_list {
                    for (func_name, _) in access_rules.iter() {
                        if !blueprint_abi.contains_fn(func_name.as_str()) {
                            return Err(BlueprintFunctionDoesNotExist(func_name.to_string()));
                        }
                    }
                }
                 */

                component.access_rules.push(input.access_rules);
                Ok(ScryptoValue::from_typed(&()))
            }
            _ => Err(ComponentError::MethodNotFound),
        }?;

        system_api.return_native_value(value_id, ref_mut);
        Ok(rtn)
    }
}
