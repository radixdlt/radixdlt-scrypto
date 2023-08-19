use scrypto::api::ClientObjectApi;
use scrypto::api::FieldValue;
use scrypto::prelude::*;

#[blueprint]
mod fake_bucket {
    struct FakeBucket {}

    impl FakeBucket {
        pub fn free_1000_xrd(bucket: Bucket) -> Bucket {
            // See LiquidFungibleResource definition
            let first_substate = Decimal::from(1000u32);
            let substates: Vec<FieldValue> = vec![FieldValue::new(&first_substate)];

            let custom_node = ScryptoVmV1Api
                .new_simple_object("FakeBucket", substates)
                .unwrap();
            let fake_bucket = scrypto_encode(&BucketPutInput {
                bucket: Bucket(Own(custom_node)),
            })
            .unwrap();
            ScryptoVmV1Api
                .call_method(bucket.0.as_node_id(), BUCKET_PUT_IDENT, fake_bucket)
                .unwrap();
            bucket
        }
    }
}
