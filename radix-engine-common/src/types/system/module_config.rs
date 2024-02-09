use crate::internal_prelude::RoleAssignmentInit;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_derive::{ManifestSbor, ScryptoSbor};

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Default, Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct ModuleConfig<T: Default> {
    pub init: T,
    pub roles: RoleAssignmentInit,
}
