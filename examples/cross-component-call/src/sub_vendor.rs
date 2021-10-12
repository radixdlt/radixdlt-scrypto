use scrypto::prelude::*;

use crate::vendor::Vendor;

blueprint! {
    struct SubVendor {
        vendor: Vendor
    }

    impl SubVendor {
        pub fn new() -> Component {
            Self {
                vendor: Vendor::new().into()
            }
            .instantiate()
        }

        pub fn get_gumball(&self, payment: Bucket) -> Bucket {
            self.vendor.get_gumball(payment)
        }
    }
}
