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

pub trait NativeAuthZone {
    fn drain<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Vec<Proof>, E>
    where
        Y: ClientApi<E>;

    fn drop_proofs<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>;

    fn drop_regular_proofs<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>;

    fn drop_signature_proofs<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>;

    fn pop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(&self, api: &mut Y) -> Result<Proof, E>
    where
        Y: ClientApi<E>;

    fn create_proof_of_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        amount: Decimal,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>;

    fn create_proof_of_non_fungibles<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>;

    fn create_proof_of_all<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>;

    fn push<P: Into<Proof>, Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        proof: P,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>;
}

impl NativeAuthZone for OwnedAuthZone {
    fn drain<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Vec<Proof>, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            AUTH_ZONE_DRAIN_IDENT,
            scrypto_encode(&AuthZoneDrainInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn drop_proofs<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            AUTH_ZONE_DROP_PROOFS_IDENT,
            scrypto_encode(&AuthZoneDropProofsInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn drop_regular_proofs<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            AUTH_ZONE_DROP_REGULAR_PROOFS_IDENT,
            scrypto_encode(&AuthZoneDropRegularProofsInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn drop_signature_proofs<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            AUTH_ZONE_DROP_SIGNATURE_PROOFS_IDENT,
            scrypto_encode(&AuthZoneDropSignatureProofsInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn pop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(&self, api: &mut Y) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            AUTH_ZONE_POP_IDENT,
            scrypto_encode(&AuthZonePopInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn create_proof_of_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        amount: Decimal,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_IDENT,
            scrypto_encode(&AuthZoneCreateProofOfAmountInput {
                resource_address,
                amount,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn create_proof_of_non_fungibles<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            scrypto_encode(&AuthZoneCreateProofOfNonFungiblesInput {
                resource_address,
                ids: ids.clone(),
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn create_proof_of_all<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            AUTH_ZONE_CREATE_PROOF_OF_ALL_IDENT,
            scrypto_encode(&AuthZoneCreateProofOfAllInput { resource_address }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn push<P: Into<Proof>, Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        proof: P,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let proof: Proof = proof.into();

        let _rtn = api.call_method(
            self.0.as_node_id(),
            AUTH_ZONE_PUSH_IDENT,
            scrypto_encode(&AuthZonePushInput { proof }).unwrap(),
        )?;

        Ok(())
    }
}
