use crate::engine::{AuthModule, CallFrameUpdate, Invokable, InvokableNative, LockFlags, NativeExecutable, NativeInvocation, NativeInvocationInfo, REActor, RENode, ResolvedReceiver, RuntimeError, SystemApi};
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

impl NativeExecutable for EpochManagerCreateInput {
    type Output = SystemAddress;

    fn execute<'s, 'a, Y, R>(
        _invocation: Self,
        system_api: &mut Y,
    ) -> Result<(SystemAddress, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi<'s, R>
            + Invokable<ScryptoInvocation>
            + InvokableNative<'a>
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

impl NativeInvocation for EpochManagerCreateInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Function(
            NativeFunction::EpochManager(EpochManagerFunction::Create),
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for EpochManagerGetCurrentEpochInput {
    type Output = u64;

    fn execute<'s, 'a, Y, R>(
        _input: Self,
        system_api: &mut Y,
    ) -> Result<(u64, CallFrameUpdate), RuntimeError>
        where
            Y: SystemApi<'s, R> + InvokableNative<'a>,
            R: FeeReserve,
    {
        // TODO: Remove this hack and get resolved receiver in a better way
        let node_id = match system_api.get_actor() {
            REActor::Method(_, ResolvedReceiver { receiver, .. }) => *receiver,
            _ => panic!("Unexpected"),
        };
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

        let substate_ref = system_api.get_ref(handle)?;
        let system = substate_ref.epoch_manager();

        Ok((system.epoch, CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for EpochManagerGetCurrentEpochInput {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::EpochManager(EpochManagerMethod::GetCurrentEpoch),
            RENodeId::Global(GlobalAddress::System(self.system_address)),
            CallFrameUpdate::empty(),
        )
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
                panic!("Unexpected")
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
