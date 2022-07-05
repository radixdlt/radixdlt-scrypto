use sbor::rust::borrow::ToOwned;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::buffer::scrypto_encode;
use crate::bytes_vec_to_struct;
use crate::component::*;
use crate::core::*;
use crate::crypto::*;
use crate::engine::{api::*, call_engine};

#[derive(Debug, TypeId, Encode, Decode)]
pub struct SystemGetCurrentEpochInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct SystemGetTransactionHashInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct SystemGetTransactionNetworkInput {}

/// The transaction runtime.
#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    /// Returns the running entity, a component if within a call-method context or a
    /// blueprint if within a call-function context.
    pub fn actor() -> ScryptoActorInfo {
        let input = RadixEngineInput::GetActor();
        let output: ScryptoActorInfo = call_engine(input);
        output
    }

    /// Returns the package ID.
    pub fn package_address() -> PackageAddress {
        let input = RadixEngineInput::GetActor();
        let output: ScryptoActorInfo = call_engine(input);
        output.to_package_address()
    }

    /// Generates a UUID.
    pub fn generate_uuid() -> u128 {
        let input = RadixEngineInput::GenerateUuid();
        let output: u128 = call_engine(input);

        output
    }

    /// Invokes a function on a blueprint.
    pub fn call_function<S: AsRef<str>, T: Decode>(
        package_address: PackageAddress,
        blueprint_name: S,
        function: S,
        args: Vec<Vec<u8>>,
    ) -> T {
        let input = RadixEngineInput::InvokeSNode(
            SNodeRef::Scrypto(ScryptoActor::Blueprint(
                package_address,
                blueprint_name.as_ref().to_owned(),
            )),
            function.as_ref().to_string(),
            bytes_vec_to_struct!(args),
        );
        call_engine(input)
    }

    /// Invokes a method on a component.
    pub fn call_method<S: AsRef<str>, T: Decode>(
        component_address: ComponentAddress,
        method: S,
        args: Vec<Vec<u8>>,
    ) -> T {
        let input = RadixEngineInput::InvokeSNode(
            SNodeRef::Scrypto(ScryptoActor::Component(component_address)),
            method.as_ref().to_string(),
            bytes_vec_to_struct!(args),
        );
        call_engine(input)
    }

    /// Returns the transaction hash.
    pub fn transaction_hash() -> Hash {
        let input = RadixEngineInput::InvokeSNode(
            SNodeRef::SystemStatic,
            "transaction_hash".to_string(),
            scrypto_encode(&SystemGetTransactionHashInput {}),
        );
        call_engine(input)
    }

    /// Returns the transaction network.
    pub fn transaction_network() -> Network {
        let input = RadixEngineInput::InvokeSNode(
            SNodeRef::SystemStatic,
            "transaction_network".to_string(),
            scrypto_encode(&SystemGetTransactionNetworkInput {}),
        );
        call_engine(input)
    }

    /// Returns the current epoch number.
    pub fn current_epoch() -> u64 {
        let input = RadixEngineInput::InvokeSNode(
            SNodeRef::SystemStatic,
            "current_epoch".to_string(),
            scrypto_encode(&SystemGetCurrentEpochInput {}),
        );
        call_engine(input)
    }
}
