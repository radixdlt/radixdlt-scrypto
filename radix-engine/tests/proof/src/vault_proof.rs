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

        pub fn create_clone_drop_vault_proof_by_amount(
            &self,
            total_amount: Decimal,
            proof_amount: Decimal,
        ) {
            let proof = self.vault.create_proof_by_amount(proof_amount);
            let clone = proof.clone();

            assert_eq!(self.vault.amount(), total_amount);
            assert_eq!(proof.amount(), proof_amount);
            assert_eq!(clone.amount(), proof_amount);

            clone.drop();
            proof.drop();
        }

        pub fn create_clone_drop_vault_proof_by_ids(
            &self,
            total_ids: BTreeSet<NonFungibleId>,
            proof_ids: BTreeSet<NonFungibleId>,
        ) {
            let proof = self.vault.create_proof_by_ids(&proof_ids);
            let clone = proof.clone();

            assert_eq!(self.vault.get_non_fungible_ids(), total_ids);
            assert_eq!(proof.get_non_fungible_ids(), proof_ids);
            assert_eq!(clone.get_non_fungible_ids(), proof_ids);

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

        pub fn receive_proof_and_move_to_auth_zone(proof: Proof) {
            AuthZone::push(proof); // should fail here
        }

        pub fn compose_vault_and_bucket_proof(&mut self, bucket: Bucket) {
            self.vault.authorize(|| {
                bucket.authorize(|| {
                    let proof = AuthZone::create_proof(bucket.resource_def_id());
                    assert_eq!(proof.amount(), self.vault.amount() + bucket.amount());
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
                    let proof = AuthZone::create_proof_by_amount(amount, bucket.resource_def_id());
                    assert_eq!(proof.amount(), amount);
                    proof.drop();
                })
            });
            self.vault.put(bucket);
        }

        pub fn compose_vault_and_bucket_proof_by_ids(
            &mut self,
            bucket: Bucket,
            ids: BTreeSet<NonFungibleId>,
        ) {
            self.vault.authorize(|| {
                bucket.authorize(|| {
                    let proof = AuthZone::create_proof_by_ids(&ids, bucket.resource_def_id());
                    assert_eq!(proof.get_non_fungible_ids(), ids);
                    proof.drop();
                })
            });
            self.vault.put(bucket);
        }
    }
}
