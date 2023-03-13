use radix_engine_interface::api::types::FnIdentifier;
use radix_engine_interface::api::{types::*, ClientEventApi, ClientObjectApi};
use radix_engine_interface::api::{ClientActorApi, ClientTransactionRuntimeApi};
use radix_engine_interface::blueprints::epoch_manager::{
    EpochManagerGetCurrentEpochInput, EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT,
};
use radix_engine_interface::blueprints::resource::NonFungibleGlobalId;
use radix_engine_interface::constants::{EPOCH_MANAGER, PACKAGE_TOKEN};
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::scrypto::{model::*, ScryptoCustomTypeExtension};
use radix_engine_interface::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoDescribe, ScryptoEncode,
};
use radix_engine_interface::*;
use sbor::generate_full_schema_from_single_type;
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
                RENodeId::GlobalObject(EPOCH_MANAGER.into()),
                EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT,
                scrypto_encode(&EpochManagerGetCurrentEpochInput).unwrap(),
            )
            .unwrap();

        scrypto_decode(&rtn).unwrap()
    }

    pub fn package_token() -> NonFungibleGlobalId {
        let non_fungible_local_id =
            NonFungibleLocalId::bytes(scrypto_encode(&Runtime::package_address()).unwrap())
                .unwrap();
        NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id)
    }

    /// Returns the running entity.
    pub fn actor() -> FnIdentifier {
        ScryptoEnv.get_fn_identifier().unwrap()
    }

    /// Returns the current package address.
    pub fn package_address() -> PackageAddress {
        Self::actor().package_address
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
            .call_method(
                RENodeId::GlobalObject(component_address.into()),
                method.as_ref(),
                args,
            )
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
    pub fn emit_event<T: ScryptoEncode + ScryptoDescribe>(event: T) {
        // TODO: Simplify once ScryptoEvent trait is implemented
        let event_name = {
            let (local_type_index, schema) =
                generate_full_schema_from_single_type::<T, ScryptoCustomTypeExtension>();
            (*schema
                .resolve_type_metadata(local_type_index)
                .expect("Cant fail")
                .type_name)
                .to_owned()
        };
        ScryptoEnv
            .emit_event(event_name, scrypto_encode(&event).unwrap())
            .unwrap();
    }
}
