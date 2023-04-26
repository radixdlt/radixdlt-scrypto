use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::account::{AccountDepositInput, ACCOUNT_DEPOSIT_IDENT};
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::data::scrypto::{scrypto_encode, ScryptoDecode};
use radix_engine_interface::types::ComponentAddress;
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
