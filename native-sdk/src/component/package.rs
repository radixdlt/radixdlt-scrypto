use radix_engine_interface::api::api::EngineApi;
use radix_engine_interface::api::api::SysNativeInvokable;
use radix_engine_interface::data::ScryptoDecode;
use radix_engine_interface::model::*;
use sbor::rust::collections::HashMap;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;

#[derive(Debug)]
pub struct BorrowedPackage(pub(crate) PackageAddress);

impl BorrowedPackage {
    pub fn sys_set_royalty_config<Y, E: Debug + ScryptoDecode>(
        &self,
        royalty_config: HashMap<String, RoyaltyConfig>,
        sys_calls: &mut Y,
    ) -> Result<&Self, E>
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
