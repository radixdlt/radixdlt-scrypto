use super::HasTypeInfo;
use crate::prelude::{Global, HasStub, ObjectStub, ObjectStubHandle};
use radix_engine_common::prelude::PACKAGE_PACKAGE;
use radix_engine_interface::blueprints::package::{
    PackageClaimRoyaltyInput, PackageSetRoyaltyConfigInput, PACKAGE_BLUEPRINT,
    PACKAGE_CLAIM_ROYALTY_IDENT, PACKAGE_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::prelude::*;

pub type Package = Global<PackageStub>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PackageStub(pub ObjectStubHandle);

impl HasStub for PackageStub {
    type Stub = Self;
}

impl HasTypeInfo for PackageStub {
    const PACKAGE_ADDRESS: Option<PackageAddress> = Some(PACKAGE_PACKAGE);

    const BLUEPRINT_NAME: &'static str = PACKAGE_BLUEPRINT;

    const OWNED_TYPE_NAME: &'static str = "OwnedPackage";

    const GLOBAL_TYPE_NAME: &'static str = "GlobalPackage";
}

impl ObjectStub for PackageStub {
    fn new(handle: ObjectStubHandle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &ObjectStubHandle {
        &self.0
    }
}

impl PackageStub {
    pub fn set_royalty_config(&self, royalty_config: BTreeMap<String, RoyaltyConfig>) {
        self.call_ignore_rtn(
            PACKAGE_SET_ROYALTY_CONFIG_IDENT,
            &PackageSetRoyaltyConfigInput { royalty_config },
        );
    }

    pub fn claim_royalty(&self) -> Bucket {
        self.call(PACKAGE_CLAIM_ROYALTY_IDENT, &PackageClaimRoyaltyInput {})
    }
}
