use radix_engine_interface::api::types::{
    FnIdentifier, PackageIdentifier, RENodeId, ScryptoFnIdentifier, ScryptoReceiver,
};
use radix_engine_interface::api::{ActorApi, EngineApi, Invokable};
use radix_engine_interface::constants::{EPOCH_MANAGER, PACKAGE_TOKEN};
use radix_engine_interface::crypto::*;
use radix_engine_interface::data::{scrypto_decode, scrypto_encode, ScryptoDecode};
use radix_engine_interface::model::*;
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use scrypto::engine::scrypto_env::ScryptoEnv;

/// The transaction runtime.
#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    /// Returns the current epoch
    pub fn current_epoch() -> u64 {
        ScryptoEnv
            .invoke(EpochManagerGetCurrentEpochInvocation {
                receiver: EPOCH_MANAGER,
            })
            .unwrap()
    }

    pub fn package_token() -> NonFungibleGlobalId {
        let non_fungible_local_id = NonFungibleLocalId::bytes(
            scrypto_encode(&PackageIdentifier::Scrypto(Runtime::package_address())).unwrap(),
        )
        .unwrap();
        NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id)
    }

    /// Returns the running entity.
    pub fn actor() -> ScryptoFnIdentifier {
        match ScryptoEnv.fn_identifier().unwrap() {
            FnIdentifier::Scrypto(identifier) => identifier,
            _ => panic!("Unexpected actor"),
        }
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
        let buffer = ScryptoEnv
            .invoke(ScryptoInvocation {
                package_address,
                blueprint_name: blueprint_name.as_ref().to_owned(),
                fn_name: function_name.as_ref().to_owned(),
                receiver: None,
                args,
            })
            .unwrap();
        scrypto_decode(&scrypto_encode(&buffer).unwrap()).unwrap()
    }

    /// Invokes a method on a component.
    pub fn call_method<S: AsRef<str>, T: ScryptoDecode>(
        component_address: ComponentAddress,
        method: S,
        args: Vec<u8>,
    ) -> T {
        let output = ScryptoEnv
            .invoke_method(
                ScryptoReceiver::Global(component_address),
                method.as_ref(),
                args,
            )
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    /// Returns the transaction hash.
    pub fn transaction_hash() -> Hash {
        let visible_node_ids = ScryptoEnv.sys_get_visible_nodes().unwrap();
        let node_id = visible_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::TransactionRuntime(..)))
            .expect("TransactionHash does not exist");

        ScryptoEnv
            .invoke(TransactionRuntimeGetHashInvocation {
                receiver: node_id.into(),
            })
            .unwrap()
    }

    /// Generates a UUID.
    pub fn generate_uuid() -> u128 {
        let visible_node_ids = ScryptoEnv.sys_get_visible_nodes().unwrap();
        let node_id = visible_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::TransactionRuntime(..)))
            .expect("TransactionHash does not exist");

        ScryptoEnv
            .invoke(TransactionRuntimeGenerateUuidInvocation {
                receiver: node_id.into(),
            })
            .unwrap()
    }
}
