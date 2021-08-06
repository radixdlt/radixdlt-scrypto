use sbor::{Decode, Encode};

use crate::types::Hash;

/// Resource bucket id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum BID {
    Transient(u32),

    Persisted(Hash, u32),
}
