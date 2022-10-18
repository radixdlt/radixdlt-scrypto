use crate::engine::{HeapRENode, LockFlags, SystemApi};
use crate::fee::FeeReserve;
use crate::model::ProofError::UnknownMethod;
use crate::model::{InvokeError, ProofSubstate, ResourceOperationError};
use crate::types::*;
use crate::wasm::*;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
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
    UnknownMethod,
}

pub struct Proof;

impl Proof {
    pub fn main<'s, Y, W, I, R>(
        proof_id: ProofId,
        method: ProofMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<ProofError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let node_id = RENodeId::Proof(proof_id);
        let offset = SubstateOffset::Proof(ProofOffset::Proof);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let proof = substate_ref.proof();

        let rtn = match method {
            ProofMethod::GetAmount => {
                let _: ProofGetAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ProofError::InvalidRequestData(e)))?;
                ScryptoValue::from_typed(&proof.total_amount())
            }
            ProofMethod::GetNonFungibleIds => {
                let _: ProofGetNonFungibleIdsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ProofError::InvalidRequestData(e)))?;
                ScryptoValue::from_typed(&proof.total_ids()?)
            }
            ProofMethod::GetResourceAddress => {
                let _: ProofGetResourceAddressInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ProofError::InvalidRequestData(e)))?;
                ScryptoValue::from_typed(&proof.resource_address)
            }
            ProofMethod::Clone => {
                let _: ProofCloneInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ProofError::InvalidRequestData(e)))?;
                let cloned_proof = proof.clone();
                let proof_id = system_api
                    .node_create(HeapRENode::Proof(cloned_proof))?
                    .into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
            _ => return Err(InvokeError::Error(ProofError::UnknownMethod)),
        };

        Ok(rtn)
    }

    pub fn main_consume<'s, Y, W, I, R>(
        node_id: RENodeId,
        method: ProofMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<ProofError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let proof: ProofSubstate = system_api.node_drop(node_id)?.into();
        match method {
            ProofMethod::Drop => {
                let _: ConsumingProofDropInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ProofError::InvalidRequestData(e)))?;
                proof.drop();
                Ok(ScryptoValue::from_typed(&()))
            }
            _ => Err(InvokeError::Error(UnknownMethod)),
        }
    }
}
