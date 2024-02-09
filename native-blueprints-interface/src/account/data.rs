#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::prelude::*;
use sbor::rust::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ScryptoSbor, ManifestSbor)]
pub enum ResourcePreference {
    /// The resource is on the allow list.
    Allowed,

    /// The resource is on the deny list.
    Disallowed,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, ScryptoSbor, ManifestSbor, Clone, Copy, Hash)]
pub enum DefaultDepositRule {
    /// Allows the deposit of all resources - the deny list is honored in this state.o
    Accept,

    /// Disallows the deposit of all resources - the allow list is honored in this state.
    Reject,

    /// Only deposits of existing resources is accepted - both allow and deny lists are honored in
    /// this mode.
    AllowExisting,
}
