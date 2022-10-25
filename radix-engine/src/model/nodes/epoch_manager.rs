use crate::engine::{AuthModule, LockFlags, RENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{
    EpochManagerSubstate, GlobalAddressSubstate, HardAuthRule, HardProofRule,
    HardResourceOrNonFungible, InvokeError, MethodAuthorization,
};
use crate::types::*;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum EpochManagerError {
    InvalidRequestData(DecodeError),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct EpochManager {
    pub info: EpochManagerSubstate,
}

impl EpochManager {
    pub fn function_auth(func: &EpochManagerFunction) -> Vec<MethodAuthorization> {
        match func {
            EpochManagerFunction::Create => {
                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                    HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                        NonFungibleAddress::new(SYSTEM_TOKEN, AuthModule::system_id()),
                    )),
                ))]
            }
        }
    }

    pub fn method_auth(method: &EpochManagerMethod) -> Vec<MethodAuthorization> {
        match method {
            EpochManagerMethod::SetEpoch => {
                vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                    HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                        NonFungibleAddress::new(SYSTEM_TOKEN, AuthModule::supervisor_id()),
                    )),
                ))]
            }
            _ => vec![],
        }
    }

    pub fn static_main<'s, Y, R>(
        func: EpochManagerFunction,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<EpochManagerError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        match func {
            EpochManagerFunction::Create => {
                let _: EpochManagerCreateInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(EpochManagerError::InvalidRequestData(e)))?;

                let node_id = system_api
                    .create_node(RENode::EpochManager(EpochManagerSubstate { epoch: 0 }))?;

                let global_node_id = system_api.create_node(RENode::Global(
                    GlobalAddressSubstate::System(node_id.into()),
                ))?;

                let system_address: SystemAddress = global_node_id.into();

                Ok(ScryptoValue::from_typed(&system_address))
            }
        }
    }

    fn method_lock_flags(method: EpochManagerMethod) -> LockFlags {
        match method {
            EpochManagerMethod::SetEpoch => LockFlags::MUTABLE,
            EpochManagerMethod::GetCurrentEpoch => LockFlags::read_only(),
        }
    }

    pub fn main<'s, Y, R>(
        component_id: ComponentId,
        method: EpochManagerMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<EpochManagerError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let node_id = RENodeId::EpochManager(component_id);
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::System);
        let handle = system_api.lock_substate(node_id, offset, Self::method_lock_flags(method))?;

        match method {
            EpochManagerMethod::GetCurrentEpoch => {
                let _: EpochManagerGetCurrentEpochInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(EpochManagerError::InvalidRequestData(e)))?;

                let substate_ref = system_api.get_ref(handle)?;
                let system = substate_ref.epoch_manager();

                Ok(ScryptoValue::from_typed(&system.epoch))
            }
            EpochManagerMethod::SetEpoch => {
                let EpochManagerSetEpochInput { epoch } = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(EpochManagerError::InvalidRequestData(e)))?;

                let mut substate_mut = system_api
                    .get_ref_mut(handle)
                    .map_err(InvokeError::Downstream)?;
                substate_mut.epoch_manager().epoch = epoch;

                Ok(ScryptoValue::from_typed(&()))
            }
        }
    }
}
