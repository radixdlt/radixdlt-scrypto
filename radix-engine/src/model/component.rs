use sbor::rust::borrow::ToOwned;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::component::*;
use scrypto::engine::types::*;
use scrypto::resource::AccessRules;
use scrypto::values::*;

use crate::engine::SystemApi;
use crate::fee::CostUnitCounter;
use crate::fee::CostUnitCounterError;
use crate::model::{convert, MethodAuthorization};
use crate::wasm::{WasmEngine, WasmInstance};

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentError {
    InvalidRequestData(DecodeError),
    BlueprintFunctionDoesNotExist(String),
    MethodNotFound,
    CostingError(CostUnitCounterError),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct ComponentState {
    state: Vec<u8>,
}

impl ComponentState {
    pub fn new(state: Vec<u8>) -> Self {
        ComponentState { state }
    }

    pub fn state(&self) -> &[u8] {
        &self.state
    }

    pub fn set_state(&mut self, new_state: Vec<u8>) {
        self.state = new_state;
    }
}

/// A component is an instance of blueprint.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Component {
    package_address: PackageAddress,
    blueprint_name: String,
    access_rules: Vec<AccessRules>,
}

impl Component {
    pub fn new(
        package_address: PackageAddress,
        blueprint_name: String,
        access_rules: Vec<AccessRules>,
    ) -> Self {
        Self {
            package_address,
            blueprint_name,
            access_rules,
        }
    }

    pub fn method_authorization(
        &self,
        component_state: &ComponentState,
        schema: &Type,
        method_name: &str,
    ) -> Vec<MethodAuthorization> {
        let data = ScryptoValue::from_slice(&component_state.state).unwrap();

        let mut authorizations = Vec::new();
        for auth in &self.access_rules {
            let method_auth = auth.get(method_name);
            let authorization = convert(schema, &data.dom, method_auth);
            authorizations.push(authorization);
        }

        authorizations
    }

    pub fn info(&self) -> (PackageAddress, String) {
        (self.package_address.clone(), self.blueprint_name.clone())
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

    pub fn main<
        'p,
        's,
        Y: SystemApi<'p, 's, W, I, C>,
        W: WasmEngine<I>,
        I: WasmInstance,
        C: CostUnitCounter,
    >(
        component_address: ComponentAddress,
        fn_ident: &str,
        arg: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, ComponentError> {
        let substate_id = SubstateId::ComponentInfo(component_address);
        let node_id = RENodeId::Component(component_address);

        let rtn = match fn_ident {
            "add_access_check" => {
                let input: ComponentAddAccessCheckInput =
                    scrypto_decode(&arg.raw).map_err(|e| ComponentError::InvalidRequestData(e))?;

                // Abi checks
                {
                    let component_ref = system_api
                        .borrow_node(&node_id)
                        .map_err(ComponentError::CostingError)?;
                    let component = component_ref.component();
                    let component_name = component.blueprint_name().to_owned();
                    let package_id = RENodeId::Package(component.package_address.clone());
                    drop(component);
                    drop(component_ref);
                    let package_ref = system_api
                        .borrow_node(&package_id)
                        .map_err(ComponentError::CostingError)?;
                    let package = package_ref.package();
                    let blueprint_abi = package.blueprint_abi(&component_name).unwrap();
                    for (func_name, _) in input.access_rules.iter() {
                        if !blueprint_abi.contains_fn(func_name.as_str()) {
                            return Err(ComponentError::BlueprintFunctionDoesNotExist(
                                func_name.to_string(),
                            ));
                        }
                    }
                }

                let mut ref_mut = system_api
                    .substate_borrow_mut(&substate_id)
                    .map_err(ComponentError::CostingError)?;
                let component = ref_mut.component();
                component.access_rules.push(input.access_rules);
                system_api
                    .substate_return_mut(ref_mut)
                    .map_err(ComponentError::CostingError)?;

                Ok(ScryptoValue::from_typed(&()))
            }
            _ => Err(ComponentError::MethodNotFound),
        }?;

        Ok(rtn)
    }
}
