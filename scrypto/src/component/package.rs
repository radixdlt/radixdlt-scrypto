use crate::engine::scrypto_env::ScryptoEnv;
use crate::modules::{Attachable, Attached, ModuleHandle};
use crate::runtime::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::package::{
    PackageClaimRoyaltyInput, PackageSetRoyaltyConfigInput, PACKAGE_CLAIM_ROYALTY_IDENT,
    PACKAGE_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoDecode};
use radix_engine_interface::types::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::prelude::Metadata;

/// Represents a published package.
#[derive(Debug)]
pub struct BorrowedPackage(pub PackageAddress);

impl BorrowedPackage {
    /// Invokes a function on this package.
    pub fn call<T: ScryptoDecode>(&self, blueprint_name: &str, function: &str, args: Vec<u8>) -> T {
        Runtime::call_function(self.0, blueprint_name, function, args)
    }

    pub fn metadata(&self) -> Attached<Metadata> {
        let metadata = Metadata::attached(self.0.into());
        Attached::new(metadata)
    }

    pub fn set_royalty_config(&self, royalty_config: BTreeMap<String, RoyaltyConfig>) {
        ScryptoEnv
            .call_method_advanced(
                self.0.as_node_id(),
                false,
                ObjectModuleId::Main,
                PACKAGE_SET_ROYALTY_CONFIG_IDENT,
                scrypto_encode(&PackageSetRoyaltyConfigInput { royalty_config }).unwrap(),
            )
            .unwrap();
    }

    pub fn claim_royalty(&self) -> Bucket {
        let rtn = ScryptoEnv
            .call_method_advanced(
                self.0.as_node_id(),
                false,
                ObjectModuleId::Main,
                PACKAGE_CLAIM_ROYALTY_IDENT,
                scrypto_encode(&PackageClaimRoyaltyInput {}).unwrap(),
            )
            .unwrap();

        scrypto_decode(&rtn).unwrap()
    }
}
