use radix_engine_interface::api::api::{EngineApi, Invokable};
use radix_engine_interface::api::types::{
    RENodeId, ScryptoActor, ScryptoFunctionIdent, ScryptoMethodIdent, ScryptoPackage,
    ScryptoReceiver,
};
use radix_engine_interface::constants::EPOCH_MANAGER;
use radix_engine_interface::crypto::*;
use radix_engine_interface::data::{scrypto_decode, ScryptoDecode};
use radix_engine_interface::model::*;
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt::Debug;
use sbor::rust::string::*;
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

    /// Returns the running entity.
    pub fn actor() -> ScryptoActor {
        ScryptoEnv.sys_get_actor().unwrap()
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
            .invoke(ScryptoInvocation::Function(
                ScryptoFunctionIdent {
                    package: ScryptoPackage::Global(package_address),
                    blueprint_name: blueprint_name.as_ref().to_owned(),
                    function_name: function_name.as_ref().to_owned(),
                },
                args,
            ))
            .unwrap();
        scrypto_decode(&buffer).unwrap()
    }

    /// Invokes a method on a component.
    pub fn call_method<S: AsRef<str>, T: ScryptoDecode>(
        component_address: ComponentAddress,
        method: S,
        args: Vec<u8>,
    ) -> T {
        let buffer = ScryptoEnv
            .invoke(ScryptoInvocation::Method(
                ScryptoMethodIdent {
                    receiver: ScryptoReceiver::Global(component_address),
                    method_name: method.as_ref().to_string(),
                },
                args,
            ))
            .unwrap();
        scrypto_decode(&buffer).unwrap()
    }

    /// Returns the transaction hash.
    pub fn transaction_hash() -> Hash {
        let visible_node_ids = ScryptoEnv.sys_get_visible_nodes().unwrap();
        let node_id = visible_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::TransactionHash(..)))
            .expect("TransactionHash does not exist");

        ScryptoEnv
            .invoke(TransactionHashGetInvocation {
                receiver: node_id.into(),
            })
            .unwrap()
    }

    /// Generates a UUID.
    pub fn generate_uuid() -> u128 {
        let visible_node_ids = ScryptoEnv.sys_get_visible_nodes().unwrap();
        let node_id = visible_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::TransactionHash(..)))
            .expect("TransactionHash does not exist");

        ScryptoEnv
            .invoke(TransactionHashGenerateUuidInvocation {
                receiver: node_id.into(),
            })
            .unwrap()
    }
}
