use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use radix_engine_interface::api::package::{
    PackageClaimRoyaltyInvocation,
};
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::{ClientComponentApi, ClientNativeInvokeApi};
use radix_engine_interface::api::node_modules::royalty::{PACKAGE_ROYALTY_SET_ROYALTY_CONFIG_IDENT, PackageSetRoyaltyConfigInput};
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::data::{scrypto_encode, ScryptoDecode};
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

/// Represents a published package.
#[derive(Debug)]
pub struct BorrowedPackage(pub(crate) PackageAddress);

impl BorrowedPackage {
    /// Invokes a function on this package.
    pub fn call<T: ScryptoDecode>(&self, blueprint_name: &str, function: &str, args: Vec<u8>) -> T {
        Runtime::call_function(self.0, blueprint_name, function, args)
    }

    pub fn set_royalty_config(&self, royalty_config: BTreeMap<String, RoyaltyConfig>) {
        ScryptoEnv.call_module_method(
            ScryptoReceiver::Package(self.0),
            NodeModuleId::PackageRoyalty,
            PACKAGE_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
            scrypto_encode(&PackageSetRoyaltyConfigInput {
                royalty_config,
            }).unwrap()
        ).unwrap();
    }

    pub fn claim_royalty(&self) -> Bucket {
        let mut env = ScryptoEnv;
        env.call_native(PackageClaimRoyaltyInvocation { receiver: self.0 })
            .unwrap()
    }
}
