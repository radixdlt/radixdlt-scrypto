use crate::errors::{InterpreterError, RuntimeError};
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::KernelNodeApi;
use crate::kernel::{
    CallFrameUpdate, ExecutableInvocation, Executor, ResolvedActor, ResolvedReceiver,
};
use crate::system::node::RENodeInit;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{
    GlobalAddress, NativeFn, ProofFn, ProofOffset, RENodeId, SubstateOffset,
};
use radix_engine_interface::api::{ClientApi, ClientDerefApi, ClientNativeInvokeApi, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ProofError {
    /// Error produced by a resource container.
    ResourceOperationError(ResourceOperationError),
    /// Can't generate zero-amount or empty non-fungible set proofs.
    EmptyProofNotAllowed,
    /// The base proofs are not enough to cover the requested amount or non-fungible ids.
    InsufficientBaseProofs,
    /// Can't apply a non-fungible operation on fungible proofs.
    NonFungibleOperationNotAllowed,
    /// Can't apply a fungible operation on non-fungible proofs.
    FungibleOperationNotAllowed,
    CouldNotCreateProof,
    InvalidRequestData(DecodeError),
}

pub struct ProofBlueprint;

impl ProofBlueprint {
    pub(crate) fn clone<Y>(
        receiver: ProofId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: ProofCloneInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle =
            api.lock_substate(
                RENodeId::Proof(receiver),
                NodeModuleId::SELF,
                SubstateOffset::Proof(ProofOffset::Proof),
                LockFlags::read_only(),
            )?;
        let substate_ref = api.get_ref(handle)?;
        let proof = substate_ref.proof();
        let cloned_proof = proof.clone();

        let node_id = api.allocate_node_id(RENodeType::Proof)?;
        api.create_node(node_id, RENodeInit::Proof(cloned_proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }
}

impl ExecutableInvocation for ProofGetAmountInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Proof(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Proof(ProofFn::GetAmount),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ProofGetAmountInvocation {
    type Output = Decimal;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Decimal, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let node_id = RENodeId::Proof(self.receiver);
        let offset = SubstateOffset::Proof(ProofOffset::Proof);
        let handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
        let substate_ref = api.get_ref(handle)?;
        let proof = substate_ref.proof();

        Ok((proof.total_amount(), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ProofGetNonFungibleLocalIdsInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Proof(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Proof(ProofFn::GetNonFungibleLocalIds),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ProofGetNonFungibleLocalIdsInvocation {
    type Output = BTreeSet<NonFungibleLocalId>;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(BTreeSet<NonFungibleLocalId>, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let node_id = RENodeId::Proof(self.receiver);
        let offset = SubstateOffset::Proof(ProofOffset::Proof);
        let handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
        let substate_ref = api.get_ref(handle)?;
        let proof = substate_ref.proof();
        let ids = proof.total_ids()?;

        Ok((ids, CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for ProofGetResourceAddressInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let receiver = RENodeId::Proof(self.receiver);
        let call_frame_update = CallFrameUpdate::copy_ref(receiver);
        let actor = ResolvedActor::method(
            NativeFn::Proof(ProofFn::GetResourceAddress),
            ResolvedReceiver::new(receiver),
        );
        Ok((actor, call_frame_update, self))
    }
}

impl Executor for ProofGetResourceAddressInvocation {
    type Output = ResourceAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(ResourceAddress, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi,
    {
        let node_id = RENodeId::Proof(self.receiver);
        let offset = SubstateOffset::Proof(ProofOffset::Proof);
        let handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
        let substate_ref = api.get_ref(handle)?;
        let proof = substate_ref.proof();

        Ok((
            proof.resource_address,
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Resource(
                proof.resource_address,
            ))),
        ))
    }
}
