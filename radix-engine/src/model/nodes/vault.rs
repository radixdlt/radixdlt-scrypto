use crate::engine::{HeapRENode, SystemApi};
use crate::fee::{FeeReserve, FeeReserveError};
use crate::model::{Bucket, InvokeError, ProofError, ResourceContainerId, ResourceOperationError};
use crate::types::*;
use crate::wasm::*;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum VaultError {
    InvalidRequestData(DecodeError),
    ResourceOperationError(ResourceOperationError),
    CouldNotCreateBucket,
    CouldNotTakeBucket,
    ProofError(ProofError),
    CouldNotCreateProof,
    LockFeeNotRadixToken,
    LockFeeInsufficientBalance,
    LockFeeRepayFailure(FeeReserveError),
}

pub struct Vault;

impl Vault {
    pub fn main<'s, Y, W, I, R>(
        vault_id: VaultId,
        method: VaultMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<VaultError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let node_id = RENodeId::Vault(vault_id);
        let offset = SubstateOffset::Vault(VaultOffset::Vault);
        let (mutable, write_through) = method.lock_type();
        let vault_handle = system_api.lock_substate(node_id, offset, mutable, write_through)?;

        let rtn = match method {
            VaultMethod::Put => {
                let input: VaultPutInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;
                let bucket = system_api
                    .node_drop(RENodeId::Bucket(input.bucket.0))?
                    .into();

                let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
                let mut raw_mut = substate_mut.get_raw_mut();
                let vault = raw_mut.vault();
                vault
                    .put(bucket)
                    .map_err(|e| InvokeError::Error(VaultError::ResourceOperationError(e)))?;
                ScryptoValue::from_typed(&())
            }
            VaultMethod::Take => {
                let input: VaultTakeInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;

                let container = {
                    let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let vault = raw_mut.vault();
                    vault.take(input.amount)?
                };

                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))?
                    .into();
                ScryptoValue::from_typed(&scrypto::resource::Bucket(bucket_id))
            }
            VaultMethod::LockFee | VaultMethod::LockContingentFee => {
                let input: VaultLockFeeInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;

                let fee = {
                    let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let vault = raw_mut.vault();

                    // Check resource and take amount
                    if vault.resource_address() != RADIX_TOKEN {
                        return Err(InvokeError::Error(VaultError::LockFeeNotRadixToken));
                    }

                    // Take fee from the vault
                    vault
                        .take(input.amount)
                        .map_err(|_| InvokeError::Error(VaultError::LockFeeInsufficientBalance))?
                };

                // Refill fee reserve
                let changes = system_api.lock_fee(
                    vault_id,
                    fee,
                    matches!(method, VaultMethod::LockContingentFee),
                )?;

                // Return changes
                {
                    let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let vault = raw_mut.vault();
                    vault
                        .borrow_resource_mut()
                        .put(changes)
                        .expect("Failed to return fee changes to a locking-fee vault");
                }

                ScryptoValue::from_typed(&())
            }
            VaultMethod::TakeNonFungibles => {
                let input: VaultTakeNonFungiblesInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;

                let container = {
                    let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let vault = raw_mut.vault();
                    vault.take_non_fungibles(&input.non_fungible_ids)?
                };

                let bucket_id = system_api
                    .node_create(HeapRENode::Bucket(Bucket::new(container)))?
                    .into();
                ScryptoValue::from_typed(&scrypto::resource::Bucket(bucket_id))
            }
            VaultMethod::GetAmount => {
                let _: VaultGetAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;
                let substate_ref = system_api.get_ref(vault_handle)?;
                let vault = substate_ref.vault();
                let amount = vault.total_amount();
                ScryptoValue::from_typed(&amount)
            }
            VaultMethod::GetResourceAddress => {
                let _: VaultGetResourceAddressInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;
                let substate_ref = system_api.get_ref(vault_handle)?;
                let vault = substate_ref.vault();
                let resource_address = vault.resource_address();
                ScryptoValue::from_typed(&resource_address)
            }
            VaultMethod::GetNonFungibleIds => {
                let _: VaultGetNonFungibleIdsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;
                let substate_ref = system_api.get_ref(vault_handle)?;
                let vault = substate_ref.vault();
                let ids = vault
                    .total_ids()
                    .map_err(|e| InvokeError::Error(VaultError::ResourceOperationError(e)))?;
                ScryptoValue::from_typed(&ids)
            }
            VaultMethod::CreateProof => {
                let _: VaultCreateProofInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;

                let proof = {
                    let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let vault = raw_mut.vault();
                    vault
                        .create_proof(ResourceContainerId::Vault(vault_id))
                        .map_err(|e| InvokeError::Error(VaultError::ProofError(e)))?
                };
                let proof_id = system_api.node_create(HeapRENode::Proof(proof))?.into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
            VaultMethod::CreateProofByAmount => {
                let input: VaultCreateProofByAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;

                let proof = {
                    let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let vault = raw_mut.vault();
                    vault
                        .create_proof_by_amount(input.amount, ResourceContainerId::Vault(vault_id))
                        .map_err(|e| InvokeError::Error(VaultError::ProofError(e)))?
                };

                let proof_id = system_api.node_create(HeapRENode::Proof(proof))?.into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
            VaultMethod::CreateProofByIds => {
                let input: VaultCreateProofByIdsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(VaultError::InvalidRequestData(e)))?;

                let proof = {
                    let mut substate_mut = system_api.get_ref_mut(vault_handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let vault = raw_mut.vault();
                    vault
                        .create_proof_by_ids(&input.ids, ResourceContainerId::Vault(vault_id))
                        .map_err(|e| InvokeError::Error(VaultError::ProofError(e)))?
                };

                let proof_id = system_api.node_create(HeapRENode::Proof(proof))?.into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
        };

        Ok(rtn)
    }
}
