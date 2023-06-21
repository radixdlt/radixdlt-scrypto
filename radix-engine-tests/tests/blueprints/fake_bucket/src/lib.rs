use scrypto::api::ClientObjectApi;
use scrypto::engine::scrypto_env::ScryptoEnv;
use scrypto::prelude::*;

#[blueprint]
mod fake_bucket {
    struct FakeBucket {}

    impl FakeBucket {
        pub fn free_1000_xrd(bucket: Bucket) -> Bucket {
            // See LiquidFungibleResource definition
            let first_substate = Decimal::from(1000u32);
            let substates: Vec<Vec<u8>> = vec![scrypto_encode(&first_substate).unwrap()];

            let custom_node = ScryptoEnv
                .new_simple_object("FakeBucket", substates)
                .unwrap();
            let fake_bucket = scrypto_encode(&BucketPutInput {
                bucket: Bucket(Own(custom_node)),
            })
            .unwrap();
            ScryptoEnv
                .call_method(bucket.0.as_node_id(), BUCKET_PUT_IDENT, fake_bucket)
                .unwrap();
            bucket
        }
    }
}
