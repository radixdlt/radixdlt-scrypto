use radix_engine_interface::api::EngineApi;
use radix_engine_interface::api::Invokable;
use radix_engine_interface::data::ScryptoDecode;
use radix_engine_interface::model::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;

#[derive(Debug)]
pub struct BorrowedPackage(pub(crate) PackageAddress);

impl BorrowedPackage {
    pub fn sys_set_royalty_config<Y, E: Debug + ScryptoDecode>(
        &self,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        sys_calls: &mut Y,
    ) -> Result<&Self, E>
    where
        Y: EngineApi<E> + Invokable<PackageSetRoyaltyConfigInvocation, E>,
    {
        sys_calls.invoke(PackageSetRoyaltyConfigInvocation {
            receiver: self.0,
            royalty_config,
        })?;

        Ok(self)
    }
}
