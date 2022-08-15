use scrypto::prelude::*;

blueprint! {
    struct Hello {
    }

    impl Hello {
        pub fn a(b: U32) {
            info!("{}", b);
        }
    }
}

