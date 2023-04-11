use radix_engine_interface::api::kernel_modules::auth_api::ClientAuthApi;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::epoch_manager::{
    EpochManagerGetCurrentEpochInput, EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT,
};
use radix_engine_interface::blueprints::resource::{AccessRule, NonFungibleGlobalId};
use radix_engine_interface::constants::{EPOCH_MANAGER, PACKAGE_TOKEN};
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoDescribe, ScryptoEncode,
};
use radix_engine_interface::traits::ScryptoEvent;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::prelude::*;
use scrypto::engine::scrypto_env::ScryptoEnv;

/// The transaction runtime.
#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    /// Returns the current epoch
    pub fn current_epoch() -> u64 {
        let rtn = ScryptoEnv
            .call_method(
                EPOCH_MANAGER.as_node_id(),
                EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT,
                scrypto_encode(&EpochManagerGetCurrentEpochInput).unwrap(),
            )
            .unwrap();

        scrypto_decode(&rtn).unwrap()
    }

    /// Returns the running entity.
    pub fn blueprint() -> Blueprint {
        ScryptoEnv.get_blueprint().unwrap()
    }

    pub fn global_address() -> ComponentAddress {
        let address: GlobalAddress = ScryptoEnv.get_global_address().unwrap();
        ComponentAddress::new_unchecked(address.into())
    }

    /// Returns the current package address.
    pub fn package_address() -> PackageAddress {
        Self::blueprint().package_address
    }

    pub fn package_token() -> NonFungibleGlobalId {
        let non_fungible_local_id =
            NonFungibleLocalId::bytes(scrypto_encode(&Runtime::package_address()).unwrap())
                .unwrap();
        NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id)
    }

    /// Invokes a function on a blueprint.
    pub fn call_function<S1: AsRef<str>, S2: AsRef<str>, T: ScryptoDecode>(
        package_address: PackageAddress,
        blueprint_name: S1,
        function_name: S2,
        args: Vec<u8>,
    ) -> T {
        let output = ScryptoEnv
            .call_function(
                package_address,
                blueprint_name.as_ref(),
                function_name.as_ref(),
                args,
            )
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    /// Invokes a method on a component.
    pub fn call_method<S: AsRef<str>, T: ScryptoDecode>(
        component_address: ComponentAddress,
        method: S,
        args: Vec<u8>,
    ) -> T {
        let output = ScryptoEnv
            .call_method(component_address.as_node_id(), method.as_ref(), args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    /// Returns the transaction hash.
    pub fn transaction_hash() -> Hash {
        ScryptoEnv.get_transaction_hash().unwrap()
    }

    /// Generates a UUID.
    pub fn generate_uuid() -> u128 {
        ScryptoEnv.generate_uuid().unwrap()
    }

    /// Emits an application event
    pub fn emit_event<T: ScryptoEncode + ScryptoDescribe + ScryptoEvent>(event: T) {
        ScryptoEnv
            .emit_event(T::event_name().to_owned(), scrypto_encode(&event).unwrap())
            .unwrap();
    }

    pub fn assert_access_rule(access_rule: AccessRule) {
        let mut env = ScryptoEnv;
        env.assert_access_rule(access_rule).unwrap();
    }
}
