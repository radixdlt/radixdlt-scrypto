use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::math::Decimal;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use sbor::rust::collections::IndexSet;
use scrypto::engine::scrypto_env::ScryptoVmV1Api;

pub trait ScryptoAuthZone {
    fn push<P: Into<Proof>>(&self, proof: P);

    fn pop(&self) -> Option<Proof>;

    fn create_proof_of_amount<A: Into<Decimal>>(
        &self,
        amount: A,
        resource_address: ResourceAddress,
    ) -> Proof;

    fn create_proof_of_non_fungibles(
        &self,
        ids: IndexSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    ) -> NonFungibleProof;

    fn create_proof_of_all(&self, resource_address: ResourceAddress) -> Proof;

    fn drop_proofs(&self);

    fn drop_signature_proofs(&self);

    fn drop_regular_proofs(&self);
}

impl ScryptoAuthZone for AuthZoneRef {
    fn push<P: Into<Proof>>(&self, proof: P) {
        let proof: Proof = proof.into();
        ScryptoVmV1Api::object_call(
            &self.0,
            AUTH_ZONE_PUSH_IDENT,
            scrypto_encode(&AuthZonePushInput { proof }).unwrap(),
        );
    }

    fn pop(&self) -> Option<Proof> {
        let rtn = ScryptoVmV1Api::object_call(
            &self.0,
            AUTH_ZONE_POP_IDENT,
            scrypto_encode(&AuthZonePopInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_of_amount<A: Into<Decimal>>(
        &self,
        amount: A,
        resource_address: ResourceAddress,
    ) -> Proof {
        let rtn = ScryptoVmV1Api::object_call(
            &self.0,
            AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_IDENT,
            scrypto_encode(&AuthZoneCreateProofOfAmountInput {
                resource_address,
                amount: amount.into(),
            })
            .unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_of_non_fungibles(
        &self,
        ids: IndexSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    ) -> NonFungibleProof {
        let rtn = ScryptoVmV1Api::object_call(
            &self.0,
            AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            scrypto_encode(&AuthZoneCreateProofOfNonFungiblesInput {
                resource_address,
                ids,
            })
            .unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_of_all(&self, resource_address: ResourceAddress) -> Proof {
        let rtn = ScryptoVmV1Api::object_call(
            &self.0,
            AUTH_ZONE_CREATE_PROOF_OF_ALL_IDENT,
            scrypto_encode(&AuthZoneCreateProofOfAllInput { resource_address }).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn drop_proofs(&self) {
        let rtn = ScryptoVmV1Api::object_call(
            &self.0,
            AUTH_ZONE_DROP_PROOFS_IDENT,
            scrypto_encode(&AuthZoneDropProofsInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn drop_signature_proofs(&self) {
        let rtn = ScryptoVmV1Api::object_call(
            &self.0,
            AUTH_ZONE_DROP_SIGNATURE_PROOFS_IDENT,
            scrypto_encode(&AuthZoneDropSignatureProofsInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn drop_regular_proofs(&self) {
        let rtn = ScryptoVmV1Api::object_call(
            &self.0,
            AUTH_ZONE_DROP_REGULAR_PROOFS_IDENT,
            scrypto_encode(&AuthZoneDropRegularProofsInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }
}
