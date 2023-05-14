use crate::prelude::{Global, ObjectStub, ObjectStubHandle};
use radix_engine_interface::blueprints::package::{
    PackageClaimRoyaltyInput, PackageSetRoyaltyConfigInput, PACKAGE_CLAIM_ROYALTY_IDENT,
    PACKAGE_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::data::scrypto::scrypto_encode;
use radix_engine_interface::types::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;

pub type Package = Global<PackageStub>;

pub struct PackageStub(ObjectStubHandle);

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
            scrypto_encode(&PackageSetRoyaltyConfigInput { royalty_config }).unwrap(),
        );
    }

    pub fn claim_royalty(&self) -> Bucket {
        self.call(
            PACKAGE_CLAIM_ROYALTY_IDENT,
            scrypto_encode(&PackageClaimRoyaltyInput {}).unwrap(),
        )
    }
}
