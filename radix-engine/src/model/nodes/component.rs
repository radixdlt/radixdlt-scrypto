use crate::engine::SystemApi;
use crate::fee::FeeReserve;
use crate::model::{ComponentInfoSubstate, ComponentStateSubstate, InvokeError};
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance};

use super::TryIntoSubstates;

#[derive(Debug, TypeId, Encode, Decode)]
pub enum ComponentError {
    InvalidRequestData(DecodeError),
    BlueprintFunctionNotFound(String),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Component {
    pub info: ComponentInfoSubstate,
    pub state: ComponentStateSubstate, // TODO: lazy loading of component state
}

impl TryIntoSubstates for Component {
    type Error = ();

    fn try_into_substates(self) -> Result<Vec<crate::model::Substate>, Self::Error> {
        Ok(vec![self.info.into(), self.state.into()])
    }
}

impl Component {
    pub fn new(
        package_address: PackageAddress,
        blueprint_name: String,
        access_rules: Vec<AccessRules>,
        state: Vec<u8>,
    ) -> Self {
        Self {
            info: ComponentInfoSubstate {
                package_address,
                blueprint_name,
                access_rules,
            },
            state: ComponentStateSubstate { state },
        }
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
        let node_id = RENodeId::Component(component_address);

        let rtn = match component_fn {
            ComponentFnIdentifier::AddAccessCheck => {
                let input: ComponentAddAccessCheckInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ComponentError::InvalidRequestData(e)))?;

                // Abi checks
                {
                    let (package_id, blueprint_name) = {
                        let mut component_ref = system_api
                            .borrow_node(&node_id)
                            .map_err(InvokeError::Downstream)?;
                        let component = component_ref.component();
                        let blueprint_name = component.info.blueprint_name.to_owned();
                        (
                            RENodeId::Package(component.info.package_address),
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

                let mut node = system_api
                    .borrow_node_mut(&node_id)
                    .map_err(InvokeError::Downstream)?;
                let component = node.component_mut();
                component.info.access_rules.push(input.access_rules);

                Ok(ScryptoValue::from_typed(&()))
            }
        }?;

        Ok(rtn)
    }
}
