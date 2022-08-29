use crate::engine::SystemApi;
use crate::fee::FeeReserve;
use crate::model::{convert, InvokeError, MethodAuthorization};
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance};

#[derive(Debug, TypeId, Encode, Decode)]
pub enum ComponentError {
    InvalidRequestData(DecodeError),
    BlueprintFunctionNotFound(String),
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
pub struct ComponentInfo {
    package_address: PackageAddress,
    blueprint_name: String,
    access_rules: Vec<AccessRules>,
}

impl ComponentInfo {
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
        let data = ScryptoValue::from_slice(&component_state.state)
            .expect("Failed to decode component state");

        let mut authorizations = Vec::new();
        for auth in &self.access_rules {
            let method_auth = auth.get(method_name);
            let authorization = convert(schema, &data, method_auth);
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

    pub fn main<'s, Y, W, I, R>(
        component_address: ComponentAddress,
        component_fn: ComponentFnIdentifier,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<ComponentError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let substate_id = SubstateId::ComponentInfo(component_address);
        let node_id = RENodeId::Component(component_address);

        let rtn = match component_fn {
            ComponentFnIdentifier::AddAccessCheck => {
                let input: ComponentAddAccessCheckInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ComponentError::InvalidRequestData(e)))?;

                // Abi checks
                {
                    let (package_id, blueprint_name) = {
                        let component_ref = system_api
                            .borrow_node(&node_id)
                            .map_err(InvokeError::Downstream)?;
                        let component = component_ref.component_info();
                        let blueprint_name = component.blueprint_name().to_owned();
                        (
                            RENodeId::Package(component.package_address.clone()),
                            blueprint_name,
                        )
                    };

                    let package_ref = system_api
                        .borrow_node(&package_id)
                        .map_err(InvokeError::Downstream)?;
                    let package = package_ref.package();
                    let blueprint_abi = package.blueprint_abi(&blueprint_name).expect(&format!(
                        "Blueprint {} is not found in package node {:?}",
                        blueprint_name, package_id
                    ));
                    for (func_name, _) in input.access_rules.iter() {
                        if !blueprint_abi.contains_fn(func_name.as_str()) {
                            return Err(InvokeError::Error(
                                ComponentError::BlueprintFunctionNotFound(func_name.to_string()),
                            ));
                        }
                    }
                }

                let mut ref_mut = system_api
                    .substate_borrow_mut(&substate_id)
                    .map_err(InvokeError::Downstream)?;
                let component_info = ref_mut.component_info();
                component_info.access_rules.push(input.access_rules);
                system_api
                    .substate_return_mut(ref_mut)
                    .map_err(InvokeError::Downstream)?;

                Ok(ScryptoValue::from_typed(&()))
            }
        }?;

        Ok(rtn)
    }
}
