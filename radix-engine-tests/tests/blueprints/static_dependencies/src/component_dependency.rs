use scrypto::prelude::*;

#[blueprint]
mod faucet_call {
    const FAUCET_ADDRESS: ComponentAddress = FAUCET;

    struct FaucetCall {
    }

    impl FaucetCall {
        pub fn call_faucet_lock_fee() {
            let amount: Decimal = 10.into();
            Runtime::call_method(
                FAUCET,
                "lock_fee",
                scrypto_args!(amount),
            )
        }
    }
}