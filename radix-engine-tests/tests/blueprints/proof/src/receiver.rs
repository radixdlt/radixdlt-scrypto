use scrypto::prelude::*;

#[blueprint]
mod receiver {
    struct Receiver {
        vault: Vault,
    }

    impl Receiver {
        pub fn assert_first_proof(
            mut proofs: Vec<Proof>,
            amount: Decimal,
            resource_address: ResourceAddress,
        ) {
            let proof = proofs.remove(0).unsafe_skip_proof_validation();
            assert_eq!(proof.amount(), amount);
            assert_eq!(proof.resource_address(), resource_address);
        }

        pub fn assert_amount(proof: Proof, amount: Decimal, resource_address: ResourceAddress) {
            let proof = proof.unsafe_skip_proof_validation();
            assert_eq!(proof.amount(), amount);
            assert_eq!(proof.resource_address(), resource_address);
        }

        pub fn assert_ids(
            proof: Proof,
            ids: BTreeSet<NonFungibleLocalId>,
            resource_address: ResourceAddress,
        ) {
            let proof = proof.unsafe_skip_proof_validation();
            assert_eq!(proof.non_fungible_local_ids(), ids);
            assert_eq!(proof.resource_address(), resource_address);
        }
    }
}
