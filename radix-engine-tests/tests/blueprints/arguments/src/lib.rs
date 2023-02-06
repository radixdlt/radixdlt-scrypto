use scrypto::prelude::*;

#[blueprint]
mod arguments {
    struct Arguments {}

    impl Arguments {
        pub fn vector_argument(arg: Vec<Bucket>) -> Vec<Bucket> {
            arg
        }
        pub fn tuple_argument(arg: (Bucket, Bucket)) -> (Bucket, Bucket) {
            arg
        }
        pub fn treemap_argument(arg: BTreeMap<String, Bucket>) -> BTreeMap<String, Bucket> {
            arg
        }
        pub fn hashmap_argument(arg: HashMap<String, Bucket>) -> HashMap<String, Bucket> {
            arg
        }
        pub fn option_argument(arg: Option<Bucket>) -> Option<Bucket> {
            arg
        }
    }
}
