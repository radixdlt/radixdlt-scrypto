use radix_common::constants::*;
use radix_common::data::scrypto::model::Own;
use radix_common::data::scrypto::*;
use radix_common::types::*;
use radix_engine_interface::api::*;
use radix_engine_interface::object_modules::royalty::*;
use radix_engine_interface::types::*;
use sbor::rust::prelude::*;

pub struct ComponentRoyalty(pub Own);

impl ComponentRoyalty {
    pub fn create<Y: SystemApi<E>, E: SystemApiError>(
        royalty_config: ComponentRoyaltyConfig,
        api: &mut Y,
    ) -> Result<Own, E> {
        let rtn = api.call_function(
            ROYALTY_MODULE_PACKAGE,
            COMPONENT_ROYALTY_BLUEPRINT,
            COMPONENT_ROYALTY_CREATE_IDENT,
            scrypto_encode(&ComponentRoyaltyCreateInput { royalty_config }).unwrap(),
        )?;
        let component_royalty: Own = scrypto_decode(&rtn).unwrap();

        Ok(component_royalty)
    }

    pub fn set_royalty<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        method_name: &str,
        amount: RoyaltyAmount,
        api: &mut Y,
    ) -> Result<ComponentRoyaltySetOutput, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
            scrypto_encode(&ComponentRoyaltySetInput {
                method: method_name.to_owned(),
                amount,
            })
            .unwrap(),
        )?;
        let rtn = scrypto_decode::<ComponentRoyaltySetOutput>(&rtn).unwrap();
        Ok(rtn)
    }

    pub fn lock_royalty<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        method_name: &str,
        api: &mut Y,
    ) -> Result<ComponentRoyaltyLockOutput, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT,
            scrypto_encode(&ComponentRoyaltyLockInput {
                method: method_name.to_owned(),
            })
            .unwrap(),
        )?;
        let rtn = scrypto_decode::<ComponentRoyaltyLockOutput>(&rtn).unwrap();
        Ok(rtn)
    }

    pub fn claim_royalty<Y: SystemApi<E>, E: SystemApiError>(
        &mut self,
        api: &mut Y,
    ) -> Result<ComponentClaimRoyaltiesOutput, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT,
            scrypto_encode(&ComponentClaimRoyaltiesInput {}).unwrap(),
        )?;
        let rtn = scrypto_decode::<ComponentClaimRoyaltiesOutput>(&rtn).unwrap();
        Ok(rtn)
    }
}
