use scrypto::prelude::*;

#[blueprint]
mod faucet_call {
    const FAUCET_PACKAGE: PackageAddress =
        address!("package_rdx1pkgxxxxxxxxxfaucetxxxxxxxxx000034355863xxxxxxxxxfaucet");
    const FAUCET_ADDRESS: ComponentAddress =
        address!("component_sim1cptxxxxxxxxxfaucetxxxxxxxxx000527798379xxxxxxxxxhkrefh");

    import_blueprint!(
        FAUCET_PACKAGE,
        Faucet,
        "Faucet",
        "OwnedFaucet",
        "GlobalFaucet",
        FaucetFunctions {},
        {
            fn lock_fee(&self, amount: Decimal);
        }
    );

    struct FaucetCall {}

    impl FaucetCall {
        pub fn call_faucet_lock_fee() {
            let amount: Decimal = 10.into();
            let faucet: Global<Faucet> = FAUCET_ADDRESS.into();
            faucet.lock_fee(amount);
        }

        pub fn call_faucet_lock_fee2(faucet: Global<Faucet>) {
            let amount: Decimal = 10.into();
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
        pub fn new(preallocated_address_bytes: [u8; 30], secret: String) -> Global<Preallocated> {
            Self { secret }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .with_address(ComponentAddress::new_or_panic(preallocated_address_bytes))
                .globalize()
        }

        pub fn get_secret(&self) -> String {
            self.secret.clone()
        }
    }
}
