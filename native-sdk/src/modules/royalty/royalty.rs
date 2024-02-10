use module_blueprints_interface::royalty::*;
use radix_engine_common::data::scrypto::model::Own;
use radix_engine_common::data::scrypto::*;
use radix_engine_common::prelude::*;
use radix_engine_system_interface::*;

pub struct ComponentRoyalty(pub Own);

impl ComponentRoyalty {
    pub fn create<Y, E: Debug + ScryptoDecode>(
        royalty_config: ComponentRoyaltyConfig,
        api: &mut Y,
    ) -> Result<Own, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_function(
            ROYALTY_MODULE_PACKAGE,
            COMPONENT_ROYALTY_BLUEPRINT,
            COMPONENT_ROYALTY_CREATE_IDENT,
            scrypto_encode(&ComponentRoyaltyCreateInput { royalty_config }).unwrap(),
        )?;
        let component_royalty: Own = scrypto_decode(&rtn).unwrap();

        Ok(component_royalty)
    }

    pub fn set_royalty<Y, E: Debug + ScryptoDecode>(
        &mut self,
        method_name: &str,
        amount: RoyaltyAmount,
        api: &mut Y,
    ) -> Result<ComponentRoyaltySetOutput, E>
    where
        Y: ClientApi<E>,
    {
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

    pub fn lock_royalty<Y, E: Debug + ScryptoDecode>(
        &mut self,
        method_name: &str,
        api: &mut Y,
    ) -> Result<ComponentRoyaltyLockOutput, E>
    where
        Y: ClientApi<E>,
    {
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

    pub fn claim_royalty<Y, E: Debug + ScryptoDecode>(
        &mut self,
        api: &mut Y,
    ) -> Result<ComponentClaimRoyaltiesOutput, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT,
            scrypto_encode(&ComponentClaimRoyaltiesInput {}).unwrap(),
        )?;
        let rtn = scrypto_decode::<ComponentClaimRoyaltiesOutput>(&rtn).unwrap();
        Ok(rtn)
    }
}
