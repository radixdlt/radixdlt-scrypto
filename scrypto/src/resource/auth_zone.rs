use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeSet;
use scrypto::engine::scrypto_env::ScryptoEnv;

pub trait ScryptoAuthZone {
    fn push<P: Into<Proof>>(&self, proof: P);

    fn pop(&self) -> Proof;

    fn create_proof_of_amount<A: Into<Decimal>>(
        &self,
        amount: A,
        resource_address: ResourceAddress,
    ) -> Proof;

    fn create_proof_of_non_fungibles(
        &self,
        ids: BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    ) -> Proof;

    fn create_proof_of_all(&self, resource_address: ResourceAddress) -> Proof;

    fn drop_proofs(&self);

    fn drop_signature_proofs(&self);

    fn drop_regular_proofs(&self);
}

impl ScryptoAuthZone for OwnedAuthZone {
    fn push<P: Into<Proof>>(&self, proof: P) {
        let proof: Proof = proof.into();
        let mut env = ScryptoEnv;
        env.call_method(
            self.0.as_node_id(),
            AUTH_ZONE_PUSH_IDENT,
            scrypto_encode(&AuthZonePushInput { proof }).unwrap(),
        )
        .unwrap();
    }

    fn pop(&self) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                AUTH_ZONE_POP_IDENT,
                scrypto_encode(&AuthZonePopInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_of_amount<A: Into<Decimal>>(
        &self,
        amount: A,
        resource_address: ResourceAddress,
    ) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                AUTH_ZONE_CREATE_PROOF_OF_AMOUNT_IDENT,
                scrypto_encode(&AuthZoneCreateProofOfAmountInput {
                    resource_address,
                    amount: amount.into(),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_of_non_fungibles(
        &self,
        ids: BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    ) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                AUTH_ZONE_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
                scrypto_encode(&AuthZoneCreateProofOfNonFungiblesInput {
                    resource_address,
                    ids,
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_of_all(&self, resource_address: ResourceAddress) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                AUTH_ZONE_CREATE_PROOF_OF_ALL_IDENT,
                scrypto_encode(&AuthZoneCreateProofOfAllInput { resource_address }).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn drop_proofs(&self) {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                AUTH_ZONE_DROP_PROOFS_IDENT,
                scrypto_encode(&AuthZoneDropProofsInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn drop_signature_proofs(&self) {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                AUTH_ZONE_DROP_SIGNATURE_PROOFS_IDENT,
                scrypto_encode(&AuthZoneDropSignatureProofsInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn drop_regular_proofs(&self) {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                AUTH_ZONE_DROP_REGULAR_PROOFS_IDENT,
                scrypto_encode(&AuthZoneDropRegularProofsInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }
}
