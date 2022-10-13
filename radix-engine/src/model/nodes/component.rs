use crate::engine::{LockFlags, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{ComponentInfoSubstate, ComponentStateSubstate, InvokeError};
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance};

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum ComponentError {
    InvalidRequestData(DecodeError),
    BlueprintFunctionNotFound(String),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Component {
    pub info: ComponentInfoSubstate,
    pub state: Option<ComponentStateSubstate>,
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
            state: Some(ComponentStateSubstate { raw: state }),
        }
    }

    pub fn get_state(&self) -> Option<&ComponentStateSubstate> {
        self.state.as_ref()
    }

    pub fn put_state(&mut self, state: ComponentStateSubstate) {
        self.state = Some(state);
    }

    fn method_lock_flags(method: ComponentMethod) -> LockFlags {
        match method {
            ComponentMethod::AddAccessCheck => LockFlags::MUTABLE,
        }
    }

    pub fn main<'s, Y, W, I, R>(
        component_id: ComponentId,
        method: ComponentMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<ComponentError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let node_id = RENodeId::Component(component_id);
        let offset = SubstateOffset::Component(ComponentOffset::Info);
        let handle = system_api.lock_substate(node_id, offset, Self::method_lock_flags(method))?;

        let rtn = match method {
            ComponentMethod::AddAccessCheck => {
                let input: ComponentAddAccessCheckInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ComponentError::InvalidRequestData(e)))?;

                // Abi checks
                {
                    let (package_id, blueprint_name) = {
                        let substate_ref = system_api.get_ref(handle)?;
                        let component_info = substate_ref.component_info();
                        let package_address = component_info.package_address;
                        let blueprint_name = component_info.blueprint_name.to_owned();
                        (
                            RENodeId::Global(GlobalAddress::Package(package_address)),
                            blueprint_name,
                        )
                    };

                    let package_offset = SubstateOffset::Package(PackageOffset::Package);
                    let handle = system_api.lock_substate(
                        package_id,
                        package_offset,
                        LockFlags::read_only(),
                    )?;
                    let substate_ref = system_api.get_ref(handle)?;
                    let package = substate_ref.package();
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

                let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
                let mut raw_mut = substate_ref_mut.get_raw_mut();
                raw_mut
                    .component_info()
                    .access_rules
                    .push(input.access_rules);
                substate_ref_mut.flush()?;

                ScryptoValue::from_typed(&())
            }
        };

        Ok(rtn)
    }
}
