use radix_engine_common::math::Decimal;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerCreateValidatorInput, ConsensusManagerStartInput,
    CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT, CONSENSUS_MANAGER_START_IDENT,
};
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::crypto::Secp256k1PublicKey;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoDecode};
use radix_engine_interface::types::ComponentAddress;
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub struct ConsensusManager(pub ComponentAddress);

impl ConsensusManager {
    pub fn create_validator<Y, E: Debug + ScryptoDecode>(
        &self,
        key: Secp256k1PublicKey,
        fee_factor: Decimal,
        xrd_payment: Bucket,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket), E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT,
            scrypto_encode(&ConsensusManagerCreateValidatorInput {
                key,
                fee_factor,
                xrd_payment,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn start<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<(), E>
    where
        Y: ClientObjectApi<E>,
    {
        api.call_method(
            self.0.as_node_id(),
            CONSENSUS_MANAGER_START_IDENT,
            scrypto_encode(&ConsensusManagerStartInput {}).unwrap(),
        )?;

        Ok(())
    }
}
