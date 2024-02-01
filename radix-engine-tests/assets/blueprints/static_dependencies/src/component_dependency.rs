use scrypto::prelude::*;

#[blueprint]
mod faucet_call {
    extern_blueprint!(
        "package_rdx1pkgxxxxxxxxxfaucetxxxxxxxxx000034355863xxxxxxxxxfaucet",
        Faucet as FiFi {
            fn lock_fee(&self, amount: Decimal);
        }
    );

    const FAUCET: Global<FiFi> = global_component!(
        FiFi,
        "component_sim1cptxxxxxxxxxfaucetxxxxxxxxx000527798379xxxxxxxxxhkrefh"
    );

    struct FaucetCall {}

    impl FaucetCall {
        pub fn call_faucet_lock_fee() {
            let amount: Decimal = 10.into();
            FAUCET.lock_fee(amount);
        }

        pub fn call_faucet_lock_fee2(faucet: Global<FiFi>) {
            let amount: Decimal = 10.into();
            faucet.lock_fee(amount);
        }

        pub fn call_faucet_lock_fee3() {
            let amount: Decimal = 10.into();
            let faucet = global_component!(
                FiFi,
                "component_sim1cptxxxxxxxxxfaucetxxxxxxxxx000527798379xxxxxxxxxhkrefh"
            );
            faucet.lock_fee(amount);
        }
    }
}

#[blueprint]
mod preallocated {
    struct Preallocated {
        secret: String,
    }

    impl Preallocated {
        pub fn new(
            preallocated_address: GlobalAddressReservation,
            secret: String,
        ) -> Global<Preallocated> {
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
