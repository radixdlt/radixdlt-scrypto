use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct MyData {}

#[blueprint]
mod bucket_test {

    struct BucketTest {
        vault: Vault,
    }

    impl BucketTest {
        pub fn create_proof_of_amount(amount: Decimal) {
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(1)
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                })
                .mint_initial_supply(2)
                .into();
            let proof = bucket.as_fungible().create_proof_of_amount(amount);
            proof.drop();
            bucket.burn();
        }

        pub fn create_vault_proof_of_amount(amount: Decimal) {
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(1)
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                })
                .mint_initial_supply(2)
                .into();

            let vault = Vault::with_bucket(bucket);
            let proof = vault.as_fungible().create_proof_of_amount(amount);
            proof.drop();
        }

        fn create_test_token(amount: u32) -> Bucket {
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(amount)
                .into();
            let proof1 = bucket.create_proof_of_all();
            let proof2 = proof1.clone();
            proof1.drop();
            proof2.drop();
            bucket
        }

        pub fn drop_fungible_empty(amount: u32) {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .mint_initial_supply(amount);

            bucket.drop_empty();
        }

        pub fn drop_non_fungible_empty(empty: bool) {
            let bucket = if empty {
                let resource_manager =
                    ResourceBuilder::new_ruid_non_fungible::<MyData>(OwnerRole::None)
                        .create_with_no_initial_supply();
                NonFungibleBucket::new(resource_manager.address())
            } else {
                ResourceBuilder::new_ruid_non_fungible::<MyData>(OwnerRole::None)
                    .mint_initial_supply([MyData {}])
            };

            bucket.drop_empty();
        }

        pub fn combine() -> Bucket {
            let mut bucket1 = Self::create_test_token(100);
            let bucket2 = bucket1.take(50);

            bucket1.put(bucket2);
            bucket1
        }

        pub fn split() -> (Bucket, Bucket) {
            let mut bucket1 = Self::create_test_token(100);
            let bucket2 = bucket1.take(Decimal::from(5));
            (bucket1, bucket2)
        }

        pub fn borrow() -> Bucket {
            let bucket = Self::create_test_token(100);
            let proof = bucket.create_proof_of_all();
            proof.drop();
            bucket
        }

        pub fn query() -> (Decimal, ResourceAddress, Bucket) {
            let bucket = Self::create_test_token(100);
            (bucket.amount(), bucket.resource_address(), bucket)
        }

        pub fn test_restricted_transfer() -> Vec<Bucket> {
            let auth_bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1)
                .into();
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(require(
                auth_bucket.resource_address()
            ))))
            .divisibility(DIVISIBILITY_MAXIMUM)
            .withdraw_roles(withdraw_roles! {
                withdrawer => OWNER;
                withdrawer_updater => rule!(deny_all);
            })
            .mint_initial_supply(5)
            .into();
            let mut vault = Vault::with_bucket(bucket);

            let token_bucket = auth_bucket
                .as_fungible()
                .authorize_with_amount(dec!(1), || vault.take(1));

            BucketTest { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
            vec![auth_bucket, token_bucket]
        }

        pub fn test_burn() -> Vec<Bucket> {
            let badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1)
                .into();
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(require(
                badge.resource_address()
            ))))
            .divisibility(DIVISIBILITY_MAXIMUM)
            .burn_roles(burn_roles! {
                burner => OWNER;
                burner_updater => rule!(deny_all);
            })
            .mint_initial_supply(5)
            .into();
            badge
                .as_fungible()
                .authorize_with_amount(dec!(1), || bucket.burn());
            vec![badge]
        }

        pub fn test_burn_freely() -> Vec<Bucket> {
            let badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1)
                .into();
            let mut bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                })
                .mint_initial_supply(5);
            let bucket2 = bucket1.take(2);
            badge
                .as_fungible()
                .authorize_with_amount(dec!(1), || bucket1.burn());
            bucket2.burn();
            vec![badge]
        }

        pub fn take_from_bucket(mut bucket: Bucket, amount: Decimal) -> (Bucket, Bucket) {
            let x = bucket.take(amount);
            (bucket, x)
        }

        pub fn create_empty_bucket_fungible() -> Bucket {
            Bucket::new(XRD)
        }

        pub fn create_empty_bucket_non_fungible() -> Bucket {
            let resource_manager =
                ResourceBuilder::new_ruid_non_fungible::<MyData>(OwnerRole::None)
                    .create_with_no_initial_supply();
            Bucket::new(resource_manager.address())
        }

        pub fn drop_locked_fungible_bucket() {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(1u32);
            let _ = bucket.create_proof_of_all();

            Self {
                vault: Vault::with_bucket(bucket.into()),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }

        pub fn drop_locked_non_fungible_bucket() {
            let bucket = ResourceBuilder::new_ruid_non_fungible::<MyData>(OwnerRole::None)
                .mint_initial_supply([MyData {}]);
            let _ = bucket.create_proof_of_all();

            Self {
                vault: Vault::with_bucket(bucket.into()),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }
    }
}

#[blueprint]
mod invalid_combine_test {
    struct InvalidCombine {
        vault: Vault,
    }

    impl InvalidCombine {
        pub fn new() -> Global<InvalidCombine> {
            let bucket = Self::create_test_token();
            Self {
                vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn lock_fee(&mut self, amount: Decimal) {
            self.vault.as_fungible().lock_fee(amount);
        }

        pub fn lock_contingent_fee(&mut self, amount: Decimal) {
            self.vault.as_fungible().lock_contingent_fee(amount);
        }

        fn create_test_token() -> Bucket {
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                })
                .mint_initial_supply(100)
                .into();
            bucket
        }

        fn create_non_fungible_test_token() -> Bucket {
            let bucket: Bucket = ResourceBuilder::new_ruid_non_fungible(OwnerRole::None)
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                })
                .mint_initial_supply(vec![()])
                .into();
            bucket
        }

        pub fn combine_fungible_invalid() -> Bucket {
            let mut bucket1 = Self::create_test_token();
            let bucket2 = Self::create_test_token();

            bucket1.put(bucket2);
            bucket1
        }

        pub fn combine_non_fungible_invalid() -> Bucket {
            let mut bucket1 = Self::create_non_fungible_test_token();
            let bucket2 = Self::create_non_fungible_test_token();

            bucket1.put(bucket2);
            bucket1
        }

        pub fn combine_fungible_vault_invalid() -> Global<InvalidCombine> {
            let bucket1 = Self::create_test_token();
            let mut vault = Vault::with_bucket(bucket1);
            let bucket2 = Self::create_test_token();

            vault.put(bucket2);

            InvalidCombine { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn combine_non_fungible_vault_invalid() -> Global<InvalidCombine> {
            let bucket1 = Self::create_non_fungible_test_token();
            let mut vault = Vault::with_bucket(bucket1);
            let bucket2 = Self::create_non_fungible_test_token();

            vault.put(bucket2);

            InvalidCombine { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn burn_fungible_invalid() {
            let bucket1 = Self::create_test_token();
            let bucket2 = Self::create_test_token();

            let resource1: ResourceManager = bucket1.resource_address().into();
            resource1.burn(bucket2);
        }

        pub fn burn_non_fungible_invalid() {
            let bucket1 = Self::create_non_fungible_test_token();
            let bucket2 = Self::create_non_fungible_test_token();

            let resource1: ResourceManager = bucket1.resource_address().into();
            resource1.burn(bucket2);
        }
    }
}
