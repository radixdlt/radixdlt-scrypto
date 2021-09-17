use scrypto::prelude::*;

blueprint! {
    struct GumballMachine {
        gumballs: Bucket,
        collected_xrd: Bucket,
    }

    impl GumballMachine {
        pub fn new() -> Address {
            Self {
                gumballs: Resource::new_fixed(HashMap::new(), 1000),
                collected_xrd: Bucket::new(Address::RadixToken),
            }
            .instantiate()
        }

        pub fn get_gumball(&mut self, payment: Bucket) -> Bucket {
            // make sure they sent us exactly 1 XRD
            assert!(payment.amount() == 1.into(), "Wrong amount of XRD sent");

            // take ownership of their XRD
            self.collected_xrd.put(payment);

            // give them back a gumball
            self.gumballs.take(1)
        }
    }
}
