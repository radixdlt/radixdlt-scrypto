use crate::engine::{LockFlags, RENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{BucketSubstate, InvokeError, ProofError, ResourceOperationError};
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
#[custom_type_id(ScryptoCustomTypeId)]
pub enum BucketError {
    InvalidDivisibility,
    InvalidRequestData(DecodeError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ResourceOperationError(ResourceOperationError),
    ProofError(ProofError),
    CouldNotCreateProof,
    MethodNotFound(BucketMethod),
}

pub struct Bucket;

impl Bucket {
    pub fn method_locks(method: BucketMethod) -> LockFlags {
        match method {
            BucketMethod::Take => LockFlags::MUTABLE,
            BucketMethod::TakeNonFungibles => LockFlags::MUTABLE,
            BucketMethod::Put => LockFlags::MUTABLE,
            BucketMethod::GetNonFungibleIds => LockFlags::read_only(),
            BucketMethod::GetAmount => LockFlags::read_only(),
            BucketMethod::GetResourceAddress => LockFlags::read_only(),
            BucketMethod::CreateProof => LockFlags::MUTABLE,
        }
    }

    pub fn main<'s, Y, R>(
        bucket_id: BucketId,
        method: BucketMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<BucketError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Bucket(bucket_id);
        let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
        let bucket_handle =
            system_api.lock_substate(node_id, offset, Self::method_locks(method))?;

        let rtn = match method {
            BucketMethod::Take => {
                let input: BucketTakeInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
                let bucket = substate_mut.bucket();
                let container = bucket
                    .take(input.amount)
                    .map_err(|e| InvokeError::Error(BucketError::ResourceOperationError(e)))?;
                let bucket_id = system_api
                    .create_node(RENode::Bucket(BucketSubstate::new(container)))?
                    .into();
                Ok::<ScryptoValue, InvokeError<BucketError>>(ScryptoValue::from_typed(
                    &scrypto::resource::Bucket(bucket_id),
                ))
            }
            BucketMethod::TakeNonFungibles => {
                let input: BucketTakeNonFungiblesInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
                let bucket = substate_mut.bucket();
                let container = bucket
                    .take_non_fungibles(&input.ids)
                    .map_err(|e| InvokeError::Error(BucketError::ResourceOperationError(e)))?;
                let bucket_id = system_api
                    .create_node(RENode::Bucket(BucketSubstate::new(container)))?
                    .into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Bucket(
                    bucket_id,
                )))
            }
            BucketMethod::GetNonFungibleIds => {
                let _: BucketGetNonFungibleIdsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let substate_ref = system_api.get_ref(bucket_handle)?;
                let bucket = substate_ref.bucket();
                let ids = bucket
                    .total_ids()
                    .map_err(|e| InvokeError::Error(BucketError::ResourceOperationError(e)))?;
                Ok(ScryptoValue::from_typed(&ids))
            }
            BucketMethod::Put => {
                let input: BucketPutInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let other_bucket = system_api
                    .drop_node(RENodeId::Bucket(input.bucket.0))?
                    .into();
                let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
                let bucket = substate_mut.bucket();
                bucket
                    .put(other_bucket)
                    .map_err(|e| InvokeError::Error(BucketError::ResourceOperationError(e)))?;
                Ok(ScryptoValue::from_typed(&()))
            }
            BucketMethod::GetAmount => {
                let _: BucketGetAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let substate = system_api.get_ref(bucket_handle)?;
                let bucket = substate.bucket();
                Ok(ScryptoValue::from_typed(&bucket.total_amount()))
            }
            BucketMethod::GetResourceAddress => {
                let _: BucketGetResourceAddressInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;
                let substate = system_api.get_ref(bucket_handle)?;
                let bucket = substate.bucket();
                Ok(ScryptoValue::from_typed(&bucket.resource_address()))
            }
            BucketMethod::CreateProof => {
                let _: BucketCreateProofInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(BucketError::InvalidRequestData(e)))?;

                let mut substate_mut = system_api.get_ref_mut(bucket_handle)?;
                let bucket = substate_mut.bucket();
                let proof = bucket
                    .create_proof(bucket_id)
                    .map_err(|e| InvokeError::Error(BucketError::ProofError(e)))?;

                let proof_id = system_api.create_node(RENode::Proof(proof))?.into();
                Ok(ScryptoValue::from_typed(&scrypto::resource::Proof(
                    proof_id,
                )))
            }
        }?;

        Ok(rtn)
    }
}
