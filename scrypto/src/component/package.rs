use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use radix_engine_interface::api::Invokable;
use radix_engine_interface::data::ScryptoDecode;
use radix_engine_interface::model::*;
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
        let mut env = ScryptoEnv;
        env.invoke(PackageSetRoyaltyConfigInvocation {
            receiver: self.0,
            royalty_config,
        })
        .unwrap();
    }

    pub fn claim_royalty(&self) -> Bucket {
        let mut env = ScryptoEnv;
        env.invoke(PackageClaimRoyaltyInvocation { receiver: self.0 })
            .unwrap()
    }
}
