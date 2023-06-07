use scrypto::prelude::*;

#[blueprint]
mod faucet_call {
    const FAUCET_ADDRESS: ComponentAddress =
        address!("component_sim1cptxxxxxxxxxfaucetxxxxxxxxx000527798379xxxxxxxxxhkrefh");

    struct FaucetCall {}

    impl FaucetCall {
        pub fn call_faucet_lock_fee() {
            let amount: Decimal = 10.into();
            Runtime::call_method(FAUCET_ADDRESS, "lock_fee", scrypto_args!(amount))
        }
    }
}

#[blueprint]
mod preallocated {
    struct Preallocated {
        secret: String,
    }

    impl Preallocated {
        pub fn new(preallocated_address: Owned<AnyComponent>, secret: String) -> Global<Preallocated> {
            Self { secret }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(preallocated_address)
                .globalize()
        }

        pub fn get_secret(&self) -> String {
            self.secret.clone()
        }
    }
}
