// There is no main function in Scrypto.
#![no_main]

use scrypto::constructs::*;
use scrypto::resource::*;
use scrypto::types::*;
use scrypto::*;

blueprint! {
    struct GumballMachine {
        gumballs: Tokens,
        collected_xrd: Tokens,
    }
    
    impl GumballMachine {
        pub fn new() -> Address {
            let component = Self {
                gumballs: Resource::new_fixed(
                    "gum",
                    "Gumball",
                    "The best gumball in the world.",
                    "https://www.example.com/",
                    "https://www.example.com/icon.png",
                    100.into(),
                ),
                collected_xrd: Tokens::new_empty(Address::RadixToken),
            }
            .instantiate();

            info!("New gumball machine: {}", component.address());
            component.into()
        }
    
        pub fn get_gumball(&mut self, payment: Tokens) -> Tokens {
            // make sure they sent in XRD
            assert!(payment.resource() == Address::RadixToken, "Need to pay in XRD");
    
            // make sure they sent us exactly 1 XRD
            assert!(payment.amount() == 1.into(), "Wrong amount of XRD sent");
    
            // take ownership of their XRD
            self.collected_xrd.put(payment);
    
            // give them back a gumball
            self.gumballs.take(1.into())
        }
    }
}
