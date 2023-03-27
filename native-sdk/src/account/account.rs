use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::account::{AccountDepositInput, ACCOUNT_DEPOSIT_IDENT};
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::data::scrypto::model::ComponentAddress;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::data::scrypto::{scrypto_encode, ScryptoDecode};
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub struct Account(pub Own);

impl Account {
    pub fn deposit<Y, E: Debug + ScryptoDecode>(&self, bucket: Bucket, api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        api.call_method(
            &self.0.as_node_id,
            ACCOUNT_DEPOSIT_IDENT,
            scrypto_encode(&AccountDepositInput { bucket }).unwrap(),
        )?;

        Ok(())
    }
}
