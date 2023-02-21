use radix_engine::types::*;

// The test is ensuring the below compiles, to avoid regression of an issue where
// Sbor works with generic parameters but ScryptoSbor doesn't
#[derive(Clone, PartialEq, Eq, Hash, Debug, ScryptoSbor)]
pub struct Thing<T> {
    pub field: T,
}
