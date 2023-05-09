use core::fmt;
use std::fmt::{Debug, Formatter};
use radix_engine_common::address::{AddressDisplayContext, NO_NETWORK};
use crate::ScryptoSbor;
use radix_engine_common::types::GlobalAddress;
use radix_engine_common::types::PackageAddress;
use radix_engine_derive::ManifestSbor;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use scrypto_schema::InstanceSchema;
use utils::ContextualDisplay;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ObjectInfo {
    pub blueprint: Blueprint,
    pub global: bool,
    pub outer_object: Option<GlobalAddress>,
    pub instance_schema: Option<InstanceSchema>,
}

#[derive(Clone, PartialEq, Eq, Hash, ScryptoSbor, ManifestSbor)]
pub struct Blueprint {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
}

impl Blueprint {
    pub fn new(package_address: &PackageAddress, blueprint_name: &str) -> Self {
        Blueprint {
            package_address: *package_address,
            blueprint_name: blueprint_name.to_string(),
        }
    }

    pub fn size(&self) -> usize {
        self.package_address.as_ref().len() + self.blueprint_name.len()
    }
}


impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for Blueprint {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        write!(
            f,
            "{}:<{}>",
            self.package_address.display(*context),
            self.blueprint_name,
        )
    }
}

impl Debug for Blueprint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}