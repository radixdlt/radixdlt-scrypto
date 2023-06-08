use scrypto::prelude::*;

#[blueprint]
mod logger {
    struct Logger {
        vaults: Vec<Vault>,
    }

    impl Logger {
        pub fn no_panic_log_message(message: String) {
            info!("{}", message)
        }

        pub fn panic_log_message(message: String) {
            info!("{}", message);
            panic!("I'm panicking!")
        }
    }
}
