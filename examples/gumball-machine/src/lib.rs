use scrypto::prelude::*;

blueprint! {
    struct GumballMachine {
        gumballs: Vault,
        collected_xrd: Vault,
    }

    impl GumballMachine {
        pub fn new() -> Address {
            Self {
                gumballs: Vault::wrap(
                    ResourceBuilder::new()
                        .metadata("name", "Gumball")
                        .metadata("symbol", "gum")
                        .metadata("description", "The best gumball in the world.")
                        .create_fixed(1000)
                ),
                collected_xrd: Vault::new(Address::RadixToken)
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
