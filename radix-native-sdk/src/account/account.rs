use radix_common::data::scrypto::scrypto_encode;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::account::{AccountDepositInput, ACCOUNT_DEPOSIT_IDENT};
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::types::ComponentAddress;
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub struct Account(pub ComponentAddress);

impl Account {
    pub fn deposit<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), E> {
        api.call_method(
            self.0.as_node_id(),
            ACCOUNT_DEPOSIT_IDENT,
            scrypto_encode(&AccountDepositInput { bucket }).unwrap(),
        )?;

        Ok(())
    }
}
