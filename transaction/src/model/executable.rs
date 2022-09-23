use crate::model::Instruction;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::component::ComponentAddress;
use scrypto::core::{NativeFnIdentifier, Receiver};
use scrypto::crypto::*;
use scrypto::resource::NonFungibleAddress;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum MethodIdentifier {
    Scrypto {
        component_address: ComponentAddress,
        ident: String,
    },
    Native {
        receiver: Receiver,
        native_fn_identifier: NativeFnIdentifier,
    },
}

/// A common trait for all transactions that can be executed by Radix Engine.
pub trait ExecutableTransaction {
    /// Returns the transaction hash, which must be globally unique.
    fn transaction_hash(&self) -> Hash;

    /// Returns the manifest size.
    fn manifest_instructions_size(&self) -> u32;

    /// Returns the limit of cost units consumable
    fn cost_unit_limit(&self) -> u32;

    /// Returns the tip percentage
    fn tip_percentage(&self) -> u32;

    /// Returns the instructions to execute.
    fn instructions(&self) -> &[Instruction];

    fn initial_proofs(&self) -> Vec<NonFungibleAddress>;

    fn blobs(&self) -> &[Vec<u8>];
}
