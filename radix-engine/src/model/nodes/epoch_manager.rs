use crate::engine::{AuthModule, CallFrameUpdate, Invokable, InvokableNativeFunction, LockFlags, NativeFuncInvocation, NativeFunctionExecutor, RENode, RuntimeError, SystemApi};
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

impl NativeFuncInvocation for EpochManagerCreateInput {
    type NativeOutput = SystemAddress;

    fn prepare(&self) -> (NativeFunction, CallFrameUpdate) {
        (
            NativeFunction::EpochManager(EpochManagerFunction::Create),
            CallFrameUpdate::empty(),
        )
    }

    fn execute<'s, 'a, Y, R>(
        self,
        system_api: &mut Y,
    ) -> Result<(SystemAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation>
            + InvokableNativeFunction<'a>
            + Invokable<NativeFunctionInvocation>
            + Invokable<NativeMethodInvocation>,
        R: FeeReserve,
    {
        let node_id =
            system_api.create_node(RENode::EpochManager(EpochManagerSubstate { epoch: 0 }))?;

        let global_node_id = system_api.create_node(RENode::Global(
            GlobalAddressSubstate::System(node_id.into()),
        ))?;

        let system_address: SystemAddress = global_node_id.into();
        let mut node_refs_to_copy = HashSet::new();
        node_refs_to_copy.insert(global_node_id);

        let update = CallFrameUpdate {
            node_refs_to_copy,
            nodes_to_move: vec![],
        };

        Ok((system_address, update))
    }
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
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
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
