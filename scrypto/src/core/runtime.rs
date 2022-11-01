use sbor::rust::borrow::ToOwned;
use sbor::rust::string::*;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::constants::EPOCH_MANAGER;

use crate::buffer::scrypto_decode;
use crate::component::*;
use crate::core::*;
use crate::crypto::*;
use crate::engine::{api::*, scrypto_env::*, types::*};
use crate::native_fn;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct EpochManagerCreateInvocation {}

impl SysInvocation for EpochManagerCreateInvocation {
    type Output = SystemAddress;
}

impl ScryptoNativeInvocation for EpochManagerCreateInvocation {}

impl Into<NativeFnInvocation> for EpochManagerCreateInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::EpochManager(
            EpochManagerFunctionInvocation::Create(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct EpochManagerGetCurrentEpochInvocation {
    pub receiver: SystemAddress,
}

impl SysInvocation for EpochManagerGetCurrentEpochInvocation {
    type Output = u64;
}

impl ScryptoNativeInvocation for EpochManagerGetCurrentEpochInvocation {}

impl Into<NativeFnInvocation> for EpochManagerGetCurrentEpochInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::EpochManager(
            EpochManagerMethodInvocation::GetCurrentEpoch(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct EpochManagerSetEpochInvocation {
    pub receiver: SystemAddress,
    pub epoch: u64,
}

impl SysInvocation for EpochManagerSetEpochInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for EpochManagerSetEpochInvocation {}

impl Into<NativeFnInvocation> for EpochManagerSetEpochInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::EpochManager(
            EpochManagerMethodInvocation::SetEpoch(self),
        ))
    }
}

/// The transaction runtime.
#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
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
    pub fn call_function<S1: AsRef<str>, S2: AsRef<str>, T: Decode>(
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
    pub fn call_method<S: AsRef<str>, T: Decode>(
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

    native_fn! {
        pub fn current_epoch() -> u64 {
            EpochManagerGetCurrentEpochInvocation {
                receiver: EPOCH_MANAGER,
            }
        }
    }
}
