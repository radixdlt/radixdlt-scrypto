use crate::blueprints::resource::{LocalRef, ProofError, ProofMoveableSubstate};
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::system_callback::SystemLockData;
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::{ClientApi, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, Clone, ScryptoSbor)]
pub struct FungibleProofSubstate {
    pub total_locked: Decimal,
    /// The supporting containers.
    pub evidence: BTreeMap<LocalRef, Decimal>,
}

impl FungibleProofSubstate {
    pub fn new(
        total_locked: Decimal,
        evidence: BTreeMap<LocalRef, Decimal>,
    ) -> Result<FungibleProofSubstate, ProofError> {
        if total_locked.is_zero() {
            return Err(ProofError::EmptyProofNotAllowed);
        }

        Ok(Self {
            total_locked,
            evidence,
        })
    }

    pub fn clone_proof<Y: ClientApi<RuntimeError>>(
        &self,
        api: &mut Y,
    ) -> Result<Self, RuntimeError> {
        for (container, locked_amount) in &self.evidence {
            api.call_method(
                container.as_node_id(),
                match container {
                    LocalRef::Bucket(_) => FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT,
                    LocalRef::Vault(_) => FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT,
                },
                scrypto_args!(locked_amount),
            )?;
        }
        Ok(Self {
            total_locked: self.total_locked.clone(),
            evidence: self.evidence.clone(),
        })
    }

    pub fn drop_proof<Y: ClientApi<RuntimeError>>(self, api: &mut Y) -> Result<(), RuntimeError> {
        for (container, locked_amount) in &self.evidence {
            api.call_method(
                container.as_node_id(),
                match container {
                    LocalRef::Bucket(_) => FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT,
                    LocalRef::Vault(_) => FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT,
                },
                scrypto_args!(locked_amount),
            )?;
        }
        Ok(())
    }

    pub fn amount(&self) -> Decimal {
        self.total_locked
    }
}

pub struct FungibleProofBlueprint;

impl FungibleProofBlueprint {
    pub(crate) fn clone<Y>(api: &mut Y) -> Result<Proof, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let moveable = {
            let handle = api.actor_open_field(
                OBJECT_HANDLE_SELF,
                FungibleProofField::Moveable.into(),
                LockFlags::read_only(),
            )?;
            let substate_ref: ProofMoveableSubstate = api.field_lock_read_typed(handle)?;
            let moveable = substate_ref.clone();
            api.field_lock_release(handle)?;
            moveable
        };

        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            FungibleProofField::ProofRefs.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: FungibleProofSubstate = api.field_lock_read_typed(handle)?;
        let proof = substate_ref.clone();
        let clone = proof.clone_proof(api)?;

        let proof_id = api.new_simple_object(
            FUNGIBLE_PROOF_BLUEPRINT,
            vec![
                scrypto_encode(&moveable).unwrap(),
                scrypto_encode(&clone).unwrap(),
            ],
        )?;

        // Drop after object creation to keep the reference alive
        api.field_lock_release(handle)?;

        Ok(Proof(Own(proof_id)))
    }

    pub(crate) fn get_amount<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            FungibleProofField::ProofRefs.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: FungibleProofSubstate = api.field_lock_read_typed(handle)?;
        let amount = substate_ref.amount();
        api.field_lock_release(handle)?;
        Ok(amount)
    }

    pub(crate) fn get_resource_address<Y>(api: &mut Y) -> Result<ResourceAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let address =
            ResourceAddress::new_or_panic(api.actor_get_info()?.get_outer_object().into());
        Ok(address)
    }

    pub(crate) fn drop<Y>(proof: Proof, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        // TODO: add `drop` callback for drop atomicity, which will remove the necessity of kernel api.

        // Notify underlying buckets/vaults
        let handle = api.kernel_open_substate(
            proof.0.as_node_id(),
            MAIN_BASE_PARTITION,
            &NonFungibleProofField::ProofRefs.into(),
            LockFlags::read_only(),
            SystemLockData::Default,
        )?;
        let proof_substate: FungibleProofSubstate =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        proof_substate.drop_proof(api)?;
        api.kernel_close_substate(handle)?;

        // Drop self
        api.drop_object(proof.0.as_node_id())?;

        Ok(())
    }
}
