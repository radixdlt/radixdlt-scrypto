use scrypto::prelude::*;

#[blueprint]
mod test_environment {
    struct TestEnvironment {}

    impl TestEnvironment {
        pub fn run() {}
    }
}
