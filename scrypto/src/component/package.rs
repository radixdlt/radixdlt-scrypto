use crate::prelude::{Global, HasStub, ObjectStub, ObjectStubHandle};
use radix_engine_interface::blueprints::package::{
    PackageClaimRoyaltyInput, PackageSetRoyaltyConfigInput, PACKAGE_CLAIM_ROYALTY_IDENT,
    PACKAGE_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::data::scrypto::well_known_scrypto_custom_types::package_address_type_data;
use radix_engine_interface::data::scrypto::well_known_scrypto_custom_types::PACKAGE_ADDRESS_ID;
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;
use sbor::*;

#[derive(Debug, Clone, Copy, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
#[sbor(transparent)]
pub struct Package(Global<PackageStub>);

impl Describe<ScryptoCustomTypeKind> for Package {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::WellKnown([PACKAGE_ADDRESS_ID]);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        package_address_type_data()
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
