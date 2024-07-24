use crate::blueprints::resource::{LocalRef, ProofError, ProofMoveableSubstate};
use crate::errors::{ApplicationError, RuntimeError};
use crate::internal_prelude::*;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::{FieldValue, SystemApi, ACTOR_REF_OUTER, ACTOR_STATE_SELF};
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, Clone, ScryptoSbor)]
pub struct FungibleProofSubstate {
    pub total_locked: Decimal,
    /// The supporting containers.
    pub evidence: IndexMap<LocalRef, Decimal>,
}

impl FungibleProofSubstate {
    pub fn new(
        total_locked: Decimal,
        evidence: IndexMap<LocalRef, Decimal>,
    ) -> Result<FungibleProofSubstate, ProofError> {
        if total_locked.is_zero() {
            return Err(ProofError::EmptyProofNotAllowed);
        }

        Ok(Self {
            total_locked,
            evidence,
        })
    }

    pub fn clone_proof<Y: SystemApi<RuntimeError>>(
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

    pub fn teardown<Y: SystemApi<RuntimeError>>(self, api: &mut Y) -> Result<(), RuntimeError> {
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
    pub(crate) fn clone<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<Proof, RuntimeError> {
        let moveable = {
            let handle = api.actor_open_field(
                ACTOR_STATE_SELF,
                FungibleProofField::Moveable.into(),
                LockFlags::read_only(),
            )?;
            let substate_ref: ProofMoveableSubstate = api.field_read_typed(handle)?;
            let moveable = substate_ref.clone();
            api.field_close(handle)?;
            moveable
        };

        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleProofField::ProofRefs.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: FungibleProofSubstate = api.field_read_typed(handle)?;
        let proof = substate_ref.clone();
        let clone = proof.clone_proof(api)?;

        let proof_id = api.new_simple_object(
            FUNGIBLE_PROOF_BLUEPRINT,
            indexmap! {
                FungibleProofField::Moveable.field_index() => FieldValue::new(&moveable),
                FungibleProofField::ProofRefs.field_index() => FieldValue::new(&clone),
            },
        )?;

        // Drop after object creation to keep the reference alive
        api.field_close(handle)?;

        Ok(Proof(Own(proof_id)))
    }

    pub(crate) fn get_amount<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<Decimal, RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleProofField::ProofRefs.into(),
            LockFlags::read_only(),
        )?;
        let substate_ref: FungibleProofSubstate = api.field_read_typed(handle)?;
        let amount = substate_ref.amount();
        api.field_close(handle)?;
        Ok(amount)
    }

    pub(crate) fn get_resource_address<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError> {
        let address = ResourceAddress::new_or_panic(api.actor_get_node_id(ACTOR_REF_OUTER)?.into());
        Ok(address)
    }

    pub(crate) fn drop<Y: SystemApi<RuntimeError>>(
        proof: Proof,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        api.drop_object(proof.0.as_node_id())?;

        Ok(())
    }

    pub(crate) fn on_drop<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            FungibleProofField::ProofRefs.into(),
            LockFlags::MUTABLE,
        )?;
        let proof_substate: FungibleProofSubstate = api.field_read_typed(handle)?;
        proof_substate.teardown(api)?;
        api.field_close(handle)?;

        Ok(())
    }

    pub(crate) fn on_move<Y: SystemApi<RuntimeError>>(
        is_moving_down: bool,
        is_to_barrier: bool,
        destination_blueprint_id: Option<BlueprintId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        if is_moving_down {
            let is_to_self = destination_blueprint_id.eq(&Some(BlueprintId::new(
                &RESOURCE_PACKAGE,
                FUNGIBLE_PROOF_BLUEPRINT,
            )));
            let is_to_auth_zone = destination_blueprint_id.eq(&Some(BlueprintId::new(
                &RESOURCE_PACKAGE,
                AUTH_ZONE_BLUEPRINT,
            )));
            if !is_to_self && (is_to_barrier || is_to_auth_zone) {
                let handle = api.actor_open_field(
                    ACTOR_STATE_SELF,
                    FungibleProofField::Moveable.into(),
                    LockFlags::MUTABLE,
                )?;
                let mut proof: ProofMoveableSubstate = api.field_read_typed(handle)?;

                // Check if the proof is restricted
                if proof.restricted {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::PanicMessage(
                            "Moving restricted proof downstream".to_owned(),
                        ),
                    ));
                }

                // Update restricted flag
                if is_to_barrier {
                    proof.change_to_restricted();
                }

                api.field_write_typed(handle, &proof)?;
                api.field_close(handle)?;
                Ok(())
            } else {
                // Proofs can move freely as long as it's not to a barrier or auth zone.
                Ok(())
            }
        } else {
            // No restriction for moving up
            Ok(())
        }
    }
}
