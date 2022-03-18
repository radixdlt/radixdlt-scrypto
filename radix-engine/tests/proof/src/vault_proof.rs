use scrypto::prelude::*;

blueprint! {
    struct VaultProof {
        vault: Vault,
    }

    impl VaultProof {
        pub fn new(bucket: Bucket) -> ComponentId {
            Self {
                vault: Vault::with_bucket(bucket),
            }
            .instantiate()
        }

        pub fn create_clone_drop_vault_proof(&self, amount: Decimal) {
            let proof = self.vault.create_proof();
            let clone = proof.clone();

            assert_eq!(self.vault.amount(), amount);
            assert_eq!(proof.amount(), amount);
            assert_eq!(clone.amount(), amount);

            clone.drop();
            proof.drop();
        }

        pub fn use_vault_proof_for_auth(&self, to_burn: Bucket) {
            self.vault.authorize(|| {
                to_burn.burn();
            });
        }

        pub fn receive_proof(_proof: Proof) {
            // auto dropped here
        }

        pub fn receive_proof_and_move_to_auth_worktop(proof: Proof) {
            AuthWorktop::push(proof); // should fail here
        }
    }
}
