use scrypto::prelude::*;

#[blueprint]
mod logger {
    struct Logger {
        vaults: Vec<Vault>,
    }

    impl Logger {
        pub fn emit_log(message: String) {
            info!("{}", message)
        }

        pub fn rust_panic(message: String) {
            panic!("{}", message)
        }

        pub fn scrypto_panic(message: String) {
            Runtime::panic(message)
        }

        pub fn assert_length_5(message: String) {
            assert_eq!(message.len(), 5);
        }
    }
}
