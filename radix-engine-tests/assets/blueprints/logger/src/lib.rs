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

        fn mutate_in_place(input: &mut u8) -> u8 {
            *input += 1;
            *input
        }

        // This function tests the logging macros.
        // If the respective log level is enabled, the macro arguments will be
        // ignored (even if an argument is an expression that mutates data or has side effects).
        pub fn mutate_input_if_log_level_enabled(level: Level, number: String) -> u8 {
            let mut number = number.parse::<u8>().unwrap();
            match level {
                Level::Error => error!("Mutated input = {}", Self::mutate_in_place(&mut number)),
                Level::Warn => warn!("Mutated input = {}", Self::mutate_in_place(&mut number)),
                Level::Info => info!("Mutated input = {}", Self::mutate_in_place(&mut number)),
                Level::Debug => debug!("Mutated input = {}", Self::mutate_in_place(&mut number)),
                Level::Trace => trace!("Mutated input = {}", Self::mutate_in_place(&mut number)),
            }
            number
        }
    }
}
