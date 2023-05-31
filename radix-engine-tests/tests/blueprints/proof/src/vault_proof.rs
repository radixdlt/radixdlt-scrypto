use scrypto::prelude::*;

#[blueprint]
mod vault_proof {
    struct VaultProof {
        vault: Vault,
    }

    impl VaultProof {
        pub fn new(bucket: Bucket) -> Global<VaultProof> {
            Self {
                vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn create_clone_drop_vault_proof(&self, amount: Decimal) {
            let proof = self.vault.create_proof().skip_checking();
            assert_eq!(proof.resource_address(), self.vault.resource_address());
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
            let proof = self
                .vault
                .create_proof_of_amount(proof_amount)
                .skip_checking();
            assert_eq!(proof.resource_address(), self.vault.resource_address());
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
                .as_non_fungible()
                .create_proof_of_non_fungibles(proof_non_fungible_local_ids.clone())
                .skip_checking();
            assert_eq!(proof.resource_address(), self.vault.resource_address());
            let clone = proof.clone();

            assert_eq!(
                self.vault.as_non_fungible().non_fungible_local_ids(),
                non_fungible_local_ids
            );
            assert_eq!(
                proof.as_non_fungible().non_fungible_local_ids(),
                proof_non_fungible_local_ids
            );
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
            LocalAuthZone::push(proof); // should fail here
        }

        pub fn compose_vault_and_bucket_proof(&mut self, bucket: Bucket) {
            let expected_amount = Decimal::ONE;
            self.vault.authorize(|| {
                bucket.authorize(|| {
                    let proof =
                        LocalAuthZone::create_proof(bucket.resource_address()).skip_checking();
                    assert_eq!(proof.resource_address(), self.vault.resource_address());
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
                    let proof =
                        LocalAuthZone::create_proof_of_amount(amount, bucket.resource_address())
                            .skip_checking();
                    assert_eq!(proof.resource_address(), self.vault.resource_address());
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
                    let proof = LocalAuthZone::create_proof_of_non_fungibles(
                        ids.clone(),
                        bucket.resource_address(),
                    )
                    .skip_checking();
                    assert_eq!(proof.resource_address(), self.vault.resource_address());
                    assert_eq!(proof.as_non_fungible().non_fungible_local_ids(), ids);
                    proof.drop();
                })
            });
            self.vault.put(bucket);
        }
    }
}
