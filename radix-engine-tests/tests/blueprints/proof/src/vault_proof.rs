use scrypto::prelude::*;

#[blueprint]
mod vault_proof {
    struct VaultProof {
        vault: Vault,
    }

    impl VaultProof {
        pub fn new(bucket: Bucket) -> ComponentAddress {
            Self {
                vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .globalize()
        }

        pub fn create_clone_drop_vault_proof(&self, amount: Decimal) {
            let proof = self.vault.create_proof();
            let proof = proof.validate_proof(self.vault.resource_address()).unwrap();
            let clone = proof.clone();

            assert_eq!(self.vault.amount(), amount);
            assert_eq!(proof.amount(), amount);
            assert_eq!(clone.amount(), amount);

            clone.drop();
            proof.drop();
        }

        pub fn create_clone_drop_vault_proof_by_amount(
            &self,
            amount: Decimal,
            proof_amount: Decimal,
        ) {
            let proof = self.vault.create_proof_by_amount(proof_amount);
            let proof = proof.validate_proof(self.vault.resource_address()).unwrap();
            let clone = proof.clone();

            assert_eq!(self.vault.amount(), amount);
            assert_eq!(proof.amount(), proof_amount);
            assert_eq!(clone.amount(), proof_amount);

            clone.drop();
            proof.drop();
        }

        pub fn create_clone_drop_vault_proof_by_ids(
            &self,
            non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
            proof_non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
        ) {
            let proof = self
                .vault
                .create_proof_by_ids(&proof_non_fungible_local_ids);
            let proof = proof.validate_proof(self.vault.resource_address()).unwrap();
            let clone = proof.clone();

            assert_eq!(self.vault.non_fungible_local_ids(), non_fungible_local_ids);
            assert_eq!(proof.non_fungible_local_ids(), proof_non_fungible_local_ids);
            assert_eq!(clone.non_fungible_local_ids(), proof_non_fungible_local_ids);

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

        pub fn receive_proofs(_proofs: Vec<Proof>) {
            // auto dropped here
        }

        pub fn receive_proof_and_push_to_auth_zone(proof: Proof) {
            ComponentAuthZone::push(proof); // should fail here
        }

        pub fn compose_vault_and_bucket_proof(&mut self, bucket: Bucket) {
            let expected_amount = self.vault.amount() + bucket.amount();
            self.vault.authorize(|| {
                bucket.authorize(|| {
                    let proof = ComponentAuthZone::create_proof(bucket.resource_address());
                    let proof = proof.validate_proof(self.vault.resource_address()).unwrap();
                    assert_eq!(proof.amount(), expected_amount);
                    proof.drop();
                })
            });
            self.vault.put(bucket);
        }

        pub fn compose_vault_and_bucket_proof_by_amount(
            &mut self,
            bucket: Bucket,
            amount: Decimal,
        ) {
            self.vault.authorize(|| {
                bucket.authorize(|| {
                    let proof = ComponentAuthZone::create_proof_by_amount(
                        amount,
                        bucket.resource_address(),
                    );
                    let proof = proof.validate_proof(self.vault.resource_address()).unwrap();
                    assert_eq!(proof.amount(), amount);
                    proof.drop();
                })
            });
            self.vault.put(bucket);
        }

        pub fn compose_vault_and_bucket_proof_by_ids(
            &mut self,
            bucket: Bucket,
            ids: BTreeSet<NonFungibleLocalId>,
        ) {
            self.vault.authorize(|| {
                bucket.authorize(|| {
                    let proof =
                        ComponentAuthZone::create_proof_by_ids(&ids, bucket.resource_address());
                    let proof = proof.validate_proof(self.vault.resource_address()).unwrap();
                    assert_eq!(proof.non_fungible_local_ids(), ids);
                    proof.drop();
                })
            });
            self.vault.put(bucket);
        }
    }
}
