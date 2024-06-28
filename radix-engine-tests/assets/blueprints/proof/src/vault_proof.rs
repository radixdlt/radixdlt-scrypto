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

        pub fn create_clone_drop_vault_proof_by_amount(
            &self,
            amount: Decimal,
            proof_amount: Decimal,
        ) {
            let proof = self
                .vault
                .as_fungible()
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
            non_fungible_local_ids: IndexSet<NonFungibleLocalId>,
            proof_non_fungible_local_ids: IndexSet<NonFungibleLocalId>,
        ) {
            let proof = self
                .vault
                .as_non_fungible()
                .create_proof_of_non_fungibles(&proof_non_fungible_local_ids.clone())
                .skip_checking();
            assert_eq!(proof.resource_address(), self.vault.resource_address());
            let clone = proof.clone();

            assert_eq!(
                self.vault.as_non_fungible().non_fungible_local_ids(100),
                non_fungible_local_ids
            );
            assert_eq!(proof.non_fungible_local_ids(), proof_non_fungible_local_ids);
            assert_eq!(clone.non_fungible_local_ids(), proof_non_fungible_local_ids);

            clone.drop();
            proof.drop();
        }

        pub fn use_vault_proof_for_auth(&self, to_burn: Bucket) {
            self.vault.as_fungible().authorize_with_amount(dec!(1), || {
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

        pub fn receive_proof_and_pass_to_scrypto_function(proof: Proof) {
            Blueprint::<VaultProof>::receive_proof(proof);
        }

        pub fn receive_proof_and_drop(proof: Proof) {
            proof.drop();
        }

        pub fn compose_vault_and_bucket_proof(&mut self, bucket: Bucket) {
            self.vault.as_fungible().authorize_with_amount(dec!(1), || {
                bucket.as_fungible().authorize_with_amount(dec!(1), || {
                    let proof = LocalAuthZone::create_proof_of_all(bucket.resource_address())
                        .skip_checking();
                    assert_eq!(proof.resource_address(), self.vault.resource_address());
                    assert_eq!(proof.amount(), dec!(2));
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
            self.vault.as_fungible().authorize_with_amount(dec!(1), || {
                bucket.authorize_with_all(|| {
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
            ids: IndexSet<NonFungibleLocalId>,
        ) {
            let vault_fungible_ids = self.vault.as_non_fungible().non_fungible_local_ids(100);
            self.vault
                .as_non_fungible()
                .authorize_with_non_fungibles(&vault_fungible_ids, || {
                    bucket.authorize_with_all(|| {
                        let proof = LocalAuthZone::create_proof_of_non_fungibles(
                            ids.clone(),
                            bucket.resource_address(),
                        )
                        .skip_checking();
                        assert_eq!(proof.resource_address(), self.vault.resource_address());
                        assert_eq!(proof.non_fungible_local_ids(), ids);
                        proof.drop();
                    })
                });
            self.vault.put(bucket);
        }
    }
}
