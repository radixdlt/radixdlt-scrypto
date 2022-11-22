use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use radix_engine_interface::api::api::EngineApi;
use radix_engine_interface::api::api::SysNativeInvokable;
use radix_engine_interface::data::ScryptoCustomTypeId;
use radix_engine_interface::model::*;
use sbor::rust::collections::HashMap;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use sbor::*;

/// Represents a published package.
#[derive(Debug)]
pub struct BorrowedPackage(pub(crate) PackageAddress);

impl BorrowedPackage {
    /// Invokes a function on this package.
    pub fn call<T: Decode<ScryptoCustomTypeId>>(
        &self,
        blueprint_name: &str,
        function: &str,
        args: Vec<u8>,
    ) -> T {
        Runtime::call_function(self.0, blueprint_name, function, args)
    }

    pub fn set_royalty_config(
        &mut self,
        royalty_config: HashMap<String, RoyaltyConfig>,
    ) -> &mut Self {
        self.sys_set_royalty_config(royalty_config, &mut ScryptoEnv)
            .unwrap()
    }

    pub fn sys_set_royalty_config<Y, E: Debug + Decode<ScryptoCustomTypeId>>(
        &mut self,
        royalty_config: HashMap<String, RoyaltyConfig>,
        sys_calls: &mut Y,
    ) -> Result<&mut Self, E>
    where
        Y: EngineApi<E> + SysNativeInvokable<PackageSetRoyaltyConfigInvocation, E>,
    {
        sys_calls.sys_invoke(PackageSetRoyaltyConfigInvocation {
            receiver: self.0,
            royalty_config,
        })?;

        Ok(self)
    }
}
