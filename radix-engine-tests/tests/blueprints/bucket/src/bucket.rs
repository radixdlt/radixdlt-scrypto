use scrypto::api::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct MyData {}

#[blueprint]
mod bucket_test {

    struct BucketTest {
        vault: Vault,
    }

    impl BucketTest {
        fn create_test_token(amount: u32) -> Bucket {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mint_initial_supply(amount);
            let proof1 = bucket.create_proof();
            let proof2 = proof1.clone();
            proof1.drop();
            proof2.drop();
            bucket
        }

        pub fn drop_bucket() {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mint_initial_supply(1u32);

            ScryptoEnv
                .drop_object(bucket.0.as_node_id().clone())
                .unwrap();
        }

        pub fn drop_empty(amount: u32) {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .mint_initial_supply(amount);

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
            let proof = bucket.create_proof();
            proof.drop();
            bucket
        }

        pub fn query() -> (Decimal, ResourceAddress, Bucket) {
            let bucket = Self::create_test_token(100);
            (bucket.amount(), bucket.resource_address(), bucket)
        }

        pub fn test_restricted_transfer() -> Vec<Bucket> {
            let auth_bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .restrict_withdraw(
                    rule!(require(auth_bucket.resource_address())),
                    rule!(deny_all),
                )
                .mint_initial_supply(5);
            let mut vault = Vault::with_bucket(bucket);

            let token_bucket = auth_bucket.authorize(|| vault.take(1));

            BucketTest { vault }.instantiate().globalize();
            vec![auth_bucket, token_bucket]
        }

        pub fn test_burn() -> Vec<Bucket> {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .burnable(rule!(require(badge.resource_address())), rule!(deny_all))
                .mint_initial_supply(5);
            badge.authorize(|| bucket.burn());
            vec![badge]
        }

        pub fn test_burn_freely() -> Vec<Bucket> {
            let badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1);
            let mut bucket1 = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .burnable(rule!(allow_all), rule!(deny_all))
                .mint_initial_supply(5);
            let bucket2 = bucket1.take(2);
            badge.authorize(|| bucket1.burn());
            bucket2.burn();
            vec![badge]
        }

        pub fn take_from_bucket(mut bucket: Bucket, amount: Decimal) -> (Bucket, Bucket) {
            let x = bucket.take(amount);
            (bucket, x)
        }

        pub fn create_empty_bucket_fungible() -> Bucket {
            Bucket::new(RADIX_TOKEN)
        }

        pub fn create_empty_bucket_non_fungible() -> Bucket {
            let resource_address =
                ResourceBuilder::new_uuid_non_fungible::<MyData>().create_with_no_initial_supply();
            Bucket::new(resource_address)
        }
    }
}
