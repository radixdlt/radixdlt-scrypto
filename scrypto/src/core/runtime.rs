use crate::component::*;
use crate::core::*;
use crate::crypto::*;
use crate::engine::{api::*, call_engine};
use crate::rust::borrow::ToOwned;
use crate::rust::vec::Vec;

/// The transaction runtime.
#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    /// Returns the running entity, a component if within a call-method context or a
    /// blueprint if within a call-function context.
    pub fn actor() -> ScryptoActorInfo {
        let input = GetActorInput {};
        let output: GetActorOutput = call_engine(GET_ACTOR, input);
        output.actor
    }

    /// Returns the package ID.
    pub fn package_address() -> PackageAddress {
        let input = GetActorInput {};
        let output: GetActorOutput = call_engine(GET_ACTOR, input);
        output.actor.to_package_address()
    }

    /// Generates a UUID.
    pub fn generate_uuid() -> u128 {
        let input = GenerateUuidInput {};
        let output: GenerateUuidOutput = call_engine(GENERATE_UUID, input);

        output.uuid
    }

    /// Invokes a function on a blueprint.
    pub fn call_function<S: AsRef<str>>(
        package_address: PackageAddress,
        blueprint_name: S,
        function: S,
        args: Vec<Vec<u8>>,
    ) -> Vec<u8> {
        let mut fields = Vec::new();
        for arg in args {
            fields.push(::sbor::decode_any(&arg).unwrap());
        }
        let variant = ::sbor::Value::Enum {
            name: function.as_ref().to_owned(),
            fields
        };
        let mut bytes = Vec::new();
        let mut enc = ::sbor::Encoder::with_type(&mut bytes);
        ::sbor::encode_any(None, &variant, &mut enc);

        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::Scrypto(ScryptoActor::Blueprint(
                package_address,
                blueprint_name.as_ref().to_owned(),
            )),
            function: function.as_ref().to_owned(),
            arg: bytes,
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);

        output.rtn
    }

    /// Invokes a method on a component.
    pub fn call_method<S: AsRef<str>>(
        component_address: ComponentAddress,
        method: S,
        args: Vec<Vec<u8>>,
    ) -> Vec<u8> {
        let mut fields = Vec::new();
        for arg in args {
            fields.push(::sbor::decode_any(&arg).unwrap());
        }
        let variant = ::sbor::Value::Enum {
            name: method.as_ref().to_owned(),
            fields
        };
        let mut bytes = Vec::new();
        let mut enc = ::sbor::Encoder::with_type(&mut bytes);
        ::sbor::encode_any(None, &variant, &mut enc);

        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::Scrypto(ScryptoActor::Component(component_address)),
            function: method.as_ref().to_owned(),
            arg: bytes,
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);

        output.rtn
    }

    /// Returns the transaction hash.
    pub fn transaction_hash() -> Hash {
        let input = GetTransactionHashInput {};
        let output: GetTransactionHashOutput = call_engine(GET_TRANSACTION_HASH, input);
        output.transaction_hash
    }

    /// Returns the current epoch number.
    pub fn current_epoch() -> u64 {
        let input = GetCurrentEpochInput {};
        let output: GetCurrentEpochOutput = call_engine(GET_CURRENT_EPOCH, input);
        output.current_epoch
    }
}
