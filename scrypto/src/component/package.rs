use crate::prelude::{Global, HasStub, ObjectStub, ObjectStubHandle};
use radix_engine_interface::blueprints::package::{
    PackageClaimRoyaltyInput, PackageSetRoyaltyConfigInput, PACKAGE_CLAIM_ROYALTY_IDENT,
    PACKAGE_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::types::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;

pub type Package = Global<PackageStub>;

pub struct PackageStub(ObjectStubHandle);

impl HasStub for PackageStub {
    type Stub = Self;
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
