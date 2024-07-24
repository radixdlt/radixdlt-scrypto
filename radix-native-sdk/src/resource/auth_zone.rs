use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::math::Decimal;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use sbor::rust::collections::IndexSet;
use sbor::rust::vec::Vec;

pub trait NativeAuthZone {
    fn drain<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Vec<Proof>, E>;

    fn drop_proofs<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<(), E>;

    fn drop_regular_proofs<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y)
        -> Result<(), E>;

    fn drop_signature_proofs<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<(), E>;

    fn pop<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Option<Proof>, E>;

    fn create_proof_of_amount<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>;

    fn create_proof_of_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        ids: &IndexSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>;

    fn create_proof_of_all<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>;

    fn push<Y: SystemApi<E>, E: SystemApiError, P: Into<Proof>>(
        &self,
        proof: P,
        api: &mut Y,
    ) -> Result<(), E>;
}

impl NativeAuthZone for AuthZoneRef {
    fn drain<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Vec<Proof>, E> {
        let rtn = api.call_method(
            &self.0,
            AUTH_ZONE_DRAIN_IDENT,
            scrypto_encode(&AuthZoneDrainInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn drop_proofs<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<(), E> {
        let rtn = api.call_method(
            &self.0,
            AUTH_ZONE_DROP_PROOFS_IDENT,
            scrypto_encode(&AuthZoneDropProofsInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn drop_regular_proofs<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<(), E> {
        let rtn = api.call_method(
            &self.0,
            AUTH_ZONE_DROP_REGULAR_PROOFS_IDENT,
            scrypto_encode(&AuthZoneDropRegularProofsInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn drop_signature_proofs<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<(), E> {
        let rtn = api.call_method(
            &self.0,
            AUTH_ZONE_DROP_SIGNATURE_PROOFS_IDENT,
            scrypto_encode(&AuthZoneDropSignatureProofsInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn pop<Y: SystemApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<Option<Proof>, E> {
        let rtn = api.call_method(
            &self.0,
            AUTH_ZONE_POP_IDENT,
            scrypto_encode(&AuthZonePopInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn create_proof_of_amount<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        amount: Decimal,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E> {
        let rtn = api.call_method(
            &self.0,
            AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_IDENT,
            scrypto_encode(&AuthZoneCreateProofOfAmountInput {
                resource_address,
                amount,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn create_proof_of_non_fungibles<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        ids: &IndexSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E> {
        let rtn = api.call_method(
            &self.0,
            AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            scrypto_encode(&AuthZoneCreateProofOfNonFungiblesInput {
                resource_address,
                ids: ids.clone(),
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn create_proof_of_all<Y: SystemApi<E>, E: SystemApiError>(
        &self,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E> {
        let rtn = api.call_method(
            &self.0,
            AUTH_ZONE_CREATE_PROOF_OF_ALL_IDENT,
            scrypto_encode(&AuthZoneCreateProofOfAllInput { resource_address }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn push<Y: SystemApi<E>, E: SystemApiError, P: Into<Proof>>(
        &self,
        proof: P,
        api: &mut Y,
    ) -> Result<(), E> {
        let proof: Proof = proof.into();

        let _rtn = api.call_method(
            &self.0,
            AUTH_ZONE_PUSH_IDENT,
            scrypto_encode(&AuthZonePushInput { proof }).unwrap(),
        )?;

        Ok(())
    }
}
