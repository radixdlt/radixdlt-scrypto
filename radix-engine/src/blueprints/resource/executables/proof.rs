use crate::errors::{InterpreterError, RuntimeError};
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::KernelNodeApi;
use crate::system::node::RENodeInit;
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{ProofOffset, RENodeId, SubstateOffset};
use radix_engine_interface::api::{ClientApi, ClientNativeInvokeApi, ClientSubstateApi};
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
        let _input: ProofCloneInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.lock_substate(
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

    pub(crate) fn get_amount<Y>(
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
        let _input: ProofGetAmountInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.lock_substate(
            RENodeId::Proof(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.get_ref(handle)?;
        let proof = substate_ref.proof();
        Ok(IndexedScryptoValue::from_typed(&proof.total_amount()))
    }

    pub(crate) fn get_non_fungible_local_ids<Y>(
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
        let _input: ProofGetNonFungibleLocalIdsInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.lock_substate(
            RENodeId::Proof(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.get_ref(handle)?;
        let proof = substate_ref.proof();
        let ids = proof.total_ids()?;
        Ok(IndexedScryptoValue::from_typed(&ids))
    }

    pub(crate) fn get_resource_address<Y>(
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
        let _input: ProofGetResourceAddressInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.lock_substate(
            RENodeId::Proof(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.get_ref(handle)?;
        let proof = substate_ref.proof();
        Ok(IndexedScryptoValue::from_typed(&proof.resource_address))
    }
}
