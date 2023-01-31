use radix_engine_interface::api::package::PackageSetRoyaltyConfigInvocation;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientNodeApi;
use radix_engine_interface::api::ClientSubstateApi;
use radix_engine_interface::api::Invokable;
use radix_engine_interface::data::ScryptoDecode;
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
        Y: ClientNodeApi<E>
            + ClientSubstateApi<E>
            + Invokable<PackageSetRoyaltyConfigInvocation, E>,
    {
        sys_calls.invoke(PackageSetRoyaltyConfigInvocation {
            receiver: self.0,
            royalty_config,
        })?;

        Ok(self)
    }
}
