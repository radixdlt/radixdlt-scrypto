use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::consensus_manager::{
    ValidatorAcceptsDelegatedStakeInput, ValidatorRegisterInput, ValidatorStakeInput,
    ValidatorUpdateAcceptDelegatedStakeInput, VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT,
    VALIDATOR_REGISTER_IDENT, VALIDATOR_STAKE_IDENT, VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT,
};
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::types::ComponentAddress;
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub struct Validator(pub ComponentAddress);

impl Validator {
    pub fn register<Y: SystemObjectApi<E>, E: SystemApiError>(&self, api: &mut Y) -> Result<(), E> {
        api.call_method(
            self.0.as_node_id(),
            VALIDATOR_REGISTER_IDENT,
            scrypto_encode(&ValidatorRegisterInput {}).unwrap(),
        )?;

        Ok(())
    }

    pub fn update_accept_delegated_stake<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        accept_delegated_stake: bool,
        api: &mut Y,
    ) -> Result<(), E> {
        api.call_method(
            self.0.as_node_id(),
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT,
            scrypto_encode(&ValidatorUpdateAcceptDelegatedStakeInput {
                accept_delegated_stake,
            })
            .unwrap(),
        )?;

        Ok(())
    }

    pub fn accepts_delegated_stake<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        api: &mut Y,
    ) -> Result<bool, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT,
            scrypto_encode(&ValidatorAcceptsDelegatedStakeInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn stake<Y: SystemObjectApi<E>, E: SystemApiError>(
        &self,
        stake: Bucket,
        api: &mut Y,
    ) -> Result<Bucket, E> {
        let rtn = api.call_method(
            self.0.as_node_id(),
            VALIDATOR_STAKE_IDENT,
            scrypto_encode(&ValidatorStakeInput { stake }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }
}
