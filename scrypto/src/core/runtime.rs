use radix_engine_lib::engine::api::{SysNativeInvokable, EngineApi};
use radix_engine_lib::engine::types::{
    ScryptoActor, ScryptoFunctionIdent, ScryptoMethodIdent, ScryptoPackage, ScryptoReceiver,
};
use radix_engine_lib::model::*;
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt::Debug;
use sbor::rust::string::*;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::constants::EPOCH_MANAGER;
use scrypto::engine::scrypto_env::ScryptoEnv;

use radix_engine_lib::crypto::*;
use radix_engine_lib::data::{scrypto_decode, ScryptoCustomTypeId};

/// The transaction runtime.
#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    pub fn current_epoch() -> u64 {
        Self::sys_current_epoch(&mut ScryptoEnv).unwrap()
    }

    pub fn sys_current_epoch<Y, E>(env: &mut Y) -> Result<u64, E>
    where
        Y: SysNativeInvokable<EpochManagerGetCurrentEpochInvocation, E>,
        E: Debug + TypeId<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    {
        env.sys_invoke(EpochManagerGetCurrentEpochInvocation {
            receiver: EPOCH_MANAGER,
        })
    }

    /// Returns the running entity, a component if within a call-method context or a
    /// blueprint if within a call-function context.
    pub fn actor() -> ScryptoActor {
        let mut syscalls = ScryptoEnv;
        syscalls.sys_get_actor().unwrap()
    }

    pub fn package_address() -> PackageAddress {
        match Self::actor() {
            ScryptoActor::Blueprint(package_address, _)
            | ScryptoActor::Component(_, package_address, _) => package_address,
        }
    }

    /// Generates a UUID.
    pub fn generate_uuid() -> u128 {
        let mut syscalls = ScryptoEnv;
        syscalls.sys_generate_uuid().unwrap()
    }

    /// Invokes a function on a blueprint.
    pub fn call_function<S1: AsRef<str>, S2: AsRef<str>, T: Decode<ScryptoCustomTypeId>>(
        package_address: PackageAddress,
        blueprint_name: S1,
        function_name: S2,
        args: Vec<u8>,
    ) -> T {
        let mut syscalls = ScryptoEnv;
        let rtn = syscalls
            .sys_invoke_scrypto_function(
                ScryptoFunctionIdent {
                    package: ScryptoPackage::Global(package_address),
                    blueprint_name: blueprint_name.as_ref().to_owned(),
                    function_name: function_name.as_ref().to_owned(),
                },
                args,
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    /// Invokes a method on a component.
    pub fn call_method<S: AsRef<str>, T: Decode<ScryptoCustomTypeId>>(
        component_address: ComponentAddress,
        method: S,
        args: Vec<u8>,
    ) -> T {
        let mut syscalls = ScryptoEnv;
        let rtn = syscalls
            .sys_invoke_scrypto_method(
                ScryptoMethodIdent {
                    receiver: ScryptoReceiver::Global(component_address),
                    method_name: method.as_ref().to_string(),
                },
                args,
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    /// Returns the transaction hash.
    pub fn transaction_hash() -> Hash {
        let mut syscalls = ScryptoEnv;
        syscalls.sys_get_transaction_hash().unwrap()
    }
}
