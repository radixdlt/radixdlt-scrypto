use sbor::{TypeId, Encode, Decode};

// TODO: we may be able to squeeze network identifier into the other fields, like the `v` byte in signature.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum Network {
    LocalSimulator,
    InternalTestnet,
}