use crate::engine::SystemApi;
use crate::fee::FeeReserve;
use crate::model::{ComponentInfo, ComponentState, InvokeError};
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance};

#[derive(Debug, TypeId, Encode, Decode)]
pub enum ComponentError {
    InvalidRequestData(DecodeError),
    BlueprintFunctionNotFound(String),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Component {
    pub info: ComponentInfo,
    pub state: ComponentState, // TODO: lazily loaded substate
}

impl Component {
    pub fn new(
        package_address: PackageAddress,
        blueprint_name: String,
        access_rules: Vec<AccessRules>,
        state: Vec<u8>,
    ) -> Self {
        Self {
            info: ComponentInfo {
                package_address,
                blueprint_name,
                access_rules,
            },
            state: ComponentState { state },
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
        let substate_id = SubstateId::ComponentInfo(component_address);
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

                let mut ref_mut = system_api
                    .substate_borrow_mut(&substate_id)
                    .map_err(InvokeError::Downstream)?;
                let component = ref_mut.component_mut();
                component.info.access_rules.push(input.access_rules);
                system_api
                    .substate_return_mut(ref_mut)
                    .map_err(InvokeError::Downstream)?;

                Ok(ScryptoValue::from_typed(&()))
            }
        }?;

        Ok(rtn)
    }
}
