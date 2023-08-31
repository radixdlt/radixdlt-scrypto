use scrypto::prelude::*;

#[blueprint]
mod fake_bucket {
    struct FakeBucket {}

    impl FakeBucket {
        pub fn free_1000_xrd(bucket: Bucket) -> Bucket {
            // See LiquidFungibleResource definition
            let first_substate = Decimal::from(1000u32);
            let substates: IndexMap<u8, FieldValue> =
                indexmap![0u8 => FieldValue::new(&first_substate)];

            let custom_node = ScryptoVmV1Api::object_new("FakeBucket", substates);
            let fake_bucket = scrypto_encode(&BucketPutInput {
                bucket: Bucket(Own(custom_node)),
            })
            .unwrap();
            ScryptoVmV1Api::object_call(bucket.0.as_node_id(), BUCKET_PUT_IDENT, fake_bucket);
            bucket
        }
    }
}
