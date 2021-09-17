use crate::utils::*;
use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

blueprint! {
    struct ResourceTest {
        resource: Address,
        bucket: Bucket,
        secret: String,
    }

    impl ResourceTest {
        pub fn create_mutable() -> Bucket {
           let resource = create_mutable("r1", Context::package_address());
           mint_bucket(resource, 100)
        }

        pub fn create_fixed() -> Bucket {
           create_fixed("r2", 100.into())
        }

        pub fn query() -> ResourceInfo {
            let resource = create_mutable("r3", Context::package_address());
            Resource::from(resource).info()
        }
    }
}
