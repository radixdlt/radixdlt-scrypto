use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::{
    PackageSetRoyaltyInput, PACKAGE_SET_ROYALTY_IDENT,
};
use radix_engine_interface::data::scrypto::{scrypto_encode, ScryptoDecode};
use radix_engine_interface::types::PackageAddress;
use radix_engine_interface::types::*;
use sbor::rust::fmt::Debug;
use sbor::rust::string::ToString;

#[derive(Debug)]
pub struct BorrowedPackage(pub PackageAddress);

impl BorrowedPackage {
    pub fn set_royalty<Y, E: Debug + ScryptoDecode, S: ToString>(
        &self,
        blueprint: S,
        fn_name: S,
        royalty: RoyaltyAmount,
        api: &mut Y,
    ) -> Result<&Self, E>
    where
        Y: ClientApi<E>,
    {
        api.call_method_advanced(
            self.0.as_node_id(),
            false,
            ObjectModuleId::Main,
            PACKAGE_SET_ROYALTY_IDENT,
            scrypto_encode(&PackageSetRoyaltyInput {
                blueprint: blueprint.to_string(),
                fn_name: fn_name.to_string(),
                royalty,
            })
            .unwrap(),
        )?;

        Ok(self)
    }
}
