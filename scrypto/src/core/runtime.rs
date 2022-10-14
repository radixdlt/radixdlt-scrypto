use sbor::rust::borrow::ToOwned;
use sbor::rust::string::*;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::constants::SYS_SYSTEM_COMPONENT;
use scrypto::engine::types::GlobalAddress;

use crate::buffer::scrypto_encode;
use crate::component::*;
use crate::core::*;
use crate::crypto::*;
use crate::engine::types::RENodeId;
use crate::engine::{api::*, call_engine};

#[derive(Debug, TypeId, Encode, Decode)]
pub struct SystemCreateInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct SystemGetCurrentEpochInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct SystemSetEpochInput {
    pub epoch: u64,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct SystemGetTransactionHashInput {}

/// The transaction runtime.
#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    /// Returns the running entity, a component if within a call-method context or a
    /// blueprint if within a call-function context.
    pub fn actor() -> ScryptoActor {
        let input = RadixEngineInput::GetActor();
        let output: ScryptoActor = call_engine(input);
        output
    }

    pub fn package_address() -> PackageAddress {
        match Self::actor() {
            ScryptoActor::Blueprint(package_address, _)
            | ScryptoActor::Component(_, package_address, _) => package_address,
        }
    }

    /// Generates a UUID.
    pub fn generate_uuid() -> u128 {
        let input = RadixEngineInput::GenerateUuid();
        let output: u128 = call_engine(input);

        output
    }

    /// Invokes a function on a blueprint.
    pub fn call_function<S1: AsRef<str>, S2: AsRef<str>, T: Decode>(
        package_address: PackageAddress,
        blueprint_name: S1,
        function_name: S2,
        args: Vec<u8>,
    ) -> T {
        let input = RadixEngineInput::InvokeScryptoFunction(
            ScryptoFunctionIdent {
                package_address,
                blueprint_name: blueprint_name.as_ref().to_owned(),
                function_name: function_name.as_ref().to_owned(),
            },
            args,
        );
        call_engine(input)
    }

    /// Invokes a method on a component.
    pub fn call_method<S: AsRef<str>, T: Decode>(
        component_address: ComponentAddress,
        method: S,
        args: Vec<u8>,
    ) -> T {
        let input = RadixEngineInput::InvokeScryptoMethod(
            ScryptoMethodIdent {
                receiver: ScryptoReceiver::Global(component_address),
                method_name: method.as_ref().to_string(),
            },
            args,
        );
        call_engine(input)
    }

    /// Returns the transaction hash.
    pub fn transaction_hash() -> Hash {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethodIdent {
                receiver: Receiver::Ref(RENodeId::Global(GlobalAddress::Component(
                    SYS_SYSTEM_COMPONENT,
                ))),
                method_name: SystemMethod::GetTransactionHash.to_string(),
            },
            scrypto_encode(&SystemGetTransactionHashInput {}),
        );
        call_engine(input)
    }

    /// Returns the current epoch number.
    pub fn current_epoch() -> u64 {
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethodIdent {
                receiver: Receiver::Ref(RENodeId::Global(GlobalAddress::Component(
                    SYS_SYSTEM_COMPONENT,
                ))),
                method_name: SystemMethod::GetCurrentEpoch.to_string(),
            },
            scrypto_encode(&SystemGetCurrentEpochInput {}),
        );
        call_engine(input)
    }
}
