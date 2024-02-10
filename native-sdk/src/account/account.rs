use native_blueprints_interface::account::{AccountDepositInput, ACCOUNT_DEPOSIT_IDENT};
use native_blueprints_interface::resource::Bucket;
use radix_engine_common::data::scrypto::{scrypto_encode, ScryptoDecode};
use radix_engine_common::types::ComponentAddress;
use radix_engine_system_api::ClientObjectApi;
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub struct Account(pub ComponentAddress);

impl Account {
    pub fn deposit<Y, E: Debug + ScryptoDecode>(&self, bucket: Bucket, api: &mut Y) -> Result<(), E>
    where
        Y: ClientObjectApi<E>,
    {
        api.call_method(
            self.0.as_node_id(),
            ACCOUNT_DEPOSIT_IDENT,
            scrypto_encode(&AccountDepositInput { bucket }).unwrap(),
        )?;

        Ok(())
    }
}
