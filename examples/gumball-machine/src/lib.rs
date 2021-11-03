use scrypto::prelude::*;

blueprint! {
    struct GumballMachine {
        gumballs: Vault,
        collected_xrd: Vault,
    }

    impl GumballMachine {
        pub fn new() -> Component {
            let bucket = ResourceBuilder::new()
                .metadata("name", "Gumball")
                .metadata("symbol", "gum")
                .metadata("description", "The best gumball in the world.")
                .new_token_fixed(1000);
            Self {
                gumballs: Vault::with_bucket(bucket),
                collected_xrd: Vault::new(RADIX_TOKEN)
            }
            .instantiate()
        }

        pub fn get_gumball(&mut self, payment: Bucket) -> Bucket {
            // make sure they sent us exactly 1 XRD
            scrypto_assert!(payment.amount() == 1.into(), "Wrong amount of XRD sent");

            // take ownership of their XRD
            self.collected_xrd.put(payment);

            // give them back a gumball
            self.gumballs.take(1)
        }
    }
}
