use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoCategorize, ScryptoDecode,
};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

pub struct LocalAuthZone {}

impl LocalAuthZone {
    pub fn drain<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        api: &mut Y,
    ) -> Result<Vec<Proof>, E>
    where
        Y: ClientApi<E>,
    {
        let auth_zone = api.get_auth_zone()?;
        let rtn = api.call_method(
            &auth_zone,
            AUTH_ZONE_DRAIN_IDENT,
            scrypto_encode(&AuthZoneDrainInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn clear<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let auth_zone = api.get_auth_zone()?;
        let rtn = api.call_method(
            &auth_zone,
            AUTH_ZONE_CLEAR_IDENT,
            scrypto_encode(&AuthZoneClearInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn clear_signature_proofs<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let auth_zone = api.get_auth_zone()?;
        let rtn = api.call_method(
            &auth_zone,
            AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_IDENT,
            scrypto_encode(&AuthZoneClearVirtualProofsInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn pop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(api: &mut Y) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let auth_zone = api.get_auth_zone()?;
        let rtn = api.call_method(
            &auth_zone,
            AUTH_ZONE_POP_IDENT,
            scrypto_encode(&AuthZonePopInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn create_proof<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let auth_zone = api.get_auth_zone()?;
        let rtn = api.call_method(
            &auth_zone,
            AUTH_ZONE_CREATE_PROOF_IDENT,
            scrypto_encode(&AuthZoneCreateProofInput { resource_address }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn create_proof_of_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        amount: Decimal,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let auth_zone = api.get_auth_zone()?;
        let rtn = api.call_method(
            &auth_zone,
            AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_IDENT,
            scrypto_encode(&AuthZoneCreateProofOfAmountInput {
                resource_address,
                amount,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn create_proof_of_non_fungibles<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let auth_zone = api.get_auth_zone()?;
        let rtn = api.call_method(
            &auth_zone,
            AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            scrypto_encode(&AuthZoneCreateProofOfNonFungiblesInput {
                resource_address,
                ids: ids.clone(),
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn create_proof_of_all<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let auth_zone = api.get_auth_zone()?;
        let rtn = api.call_method(
            &auth_zone,
            AUTH_ZONE_CREATE_PROOF_OF_ALL_IDENT,
            scrypto_encode(&AuthZoneCreateProofOfAllInput { resource_address }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn push<P: Into<Proof>, Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        proof: P,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let proof: Proof = proof.into();

        let auth_zone = api.get_auth_zone()?;
        let _rtn = api.call_method(
            &auth_zone,
            AUTH_ZONE_PUSH_IDENT,
            scrypto_encode(&AuthZonePushInput { proof }).unwrap(),
        )?;

        Ok(())
    }
}
