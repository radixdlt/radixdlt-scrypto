use scrypto::prelude::*;

external_component! {
    Faucet {
        fn lock_fee(&self, amount: Decimal);
    }
}

/*
impl HasStub for Faucet {
    type Stub = Faucet;
}

#[derive(Copy, Clone)]
pub struct Faucet {
    pub handle: ::scrypto::component::ObjectStubHandle,
}

impl ::scrypto::component::ObjectStub for Faucet {
    fn new(handle: ::scrypto::component::ObjectStubHandle) -> Self {
        Self {
            handle
        }
    }
    fn handle(&self) -> &::scrypto::component::ObjectStubHandle {
        &self.handle
    }
}

impl Faucet {
    pub fn lock_fee(&self, amount: Decimal) {
        self.call_raw("lock_fee", scrypto_args!(amount))
    }
}
 */

/*
impl HasTypeInfo for Faucet {
    const PACKAGE_ADDRESS: Option<PackageAddress> = None;
    const BLUEPRINT_NAME: &'static str = "Faucet";
    const OWNED_TYPE_NAME: &'static str = "OwnedFaucet";
    const GLOBAL_TYPE_NAME: &'static str = "GlobalFaucet";
}
 */

#[blueprint]
mod faucet_call {
    const FAUCET_ADDRESS: ComponentAddress =
        address!("component_sim1cptxxxxxxxxxfaucetxxxxxxxxx000527798379xxxxxxxxxhkrefh");

    struct FaucetCall {}

    impl FaucetCall {
        pub fn call_faucet_lock_fee() {
            let amount: Decimal = 10.into();
            let faucet: Global<Faucet> = FAUCET_ADDRESS.into();
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
