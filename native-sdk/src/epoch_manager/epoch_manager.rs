use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::epoch_manager::{
    EpochManagerCreateValidatorInput, EpochManagerStartInput, EPOCH_MANAGER_CREATE_VALIDATOR_IDENT,
    EPOCH_MANAGER_START_IDENT,
};
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoDecode};
use radix_engine_interface::types::ComponentAddress;
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub struct EpochManager(pub ComponentAddress);

impl EpochManager {
    pub fn create_validator<Y, E: Debug + ScryptoDecode>(
        &self,
        key: EcdsaSecp256k1PublicKey,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket), E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            EPOCH_MANAGER_CREATE_VALIDATOR_IDENT,
            scrypto_encode(&EpochManagerCreateValidatorInput { key }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn start<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<(), E>
    where
        Y: ClientObjectApi<E>,
    {
        api.call_method(
            self.0.as_node_id(),
            EPOCH_MANAGER_START_IDENT,
            scrypto_encode(&EpochManagerStartInput {}).unwrap(),
        )?;

        Ok(())
    }
}
