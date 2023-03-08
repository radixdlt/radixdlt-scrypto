use crate::api::types::*;
use crate::data::scrypto::model::Own;
use crate::data::scrypto::model::*;
use crate::schema::*;
use crate::*;
use sbor::rust::fmt;
use sbor::rust::fmt::{Debug, Formatter};
use sbor::rust::prelude::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Clone, Sbor, PartialEq, Eq)]
pub struct PackageCodeSubstate {
    pub code: Vec<u8>,
}

impl PackageCodeSubstate {
    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

impl Debug for PackageCodeSubstate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PackageCodeSubstate").finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct PackageInfoSubstate {
    pub schema: PackageSchema,
    pub dependent_resources: BTreeSet<ResourceAddress>,
    pub dependent_components: BTreeSet<ComponentAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct PackageRoyaltyConfigSubstate {
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct PackageRoyaltyAccumulatorSubstate {
    /// The vault for collecting package royalties.
    ///
    /// It's optional to break circular dependency - creating package royalty vaults
    /// requires the `resource` package existing in the first place.
    pub royalty_vault: Option<Own>,
}
