use scrypto::prelude::*;

blueprint! {
    struct GumballMachine {
        gumballs: Tokens,
        collected_xrd: Tokens,
    }

    impl GumballMachine {
        pub fn new() -> Address {
            let gumballs = ResourceBuilder::new()
                .name("Gumball")
                .symbol("gum")
                .description("The best gumball in the world.")
                .create_tokens_fixed(1000);

            Self {
                gumballs,
                collected_xrd: Tokens::new(Address::RadixToken),
            }
            .instantiate()
        }

        pub fn get_gumball(&mut self, payment: Tokens) -> Tokens {
            // make sure they sent us exactly 1 XRD
            assert!(payment.amount() == 1.into(), "Wrong amount of XRD sent");

            // take ownership of their XRD
            self.collected_xrd.put(payment);

            // give them back a gumball
            self.gumballs.take(1)
        }
    }
}
