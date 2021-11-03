use scrypto::prelude::*;

blueprint! {
    struct Token {}

    impl Token {
        pub fn new(name: String, symbol: String) -> Bucket {
            return ResourceBuilder::new()
                .metadata("name", &name)
                .metadata("symbol", &symbol)
                .new_token_fixed(1000);
        }
    }
}
