use sbor::*;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

/// A transaction consists a sequence of instructions.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Transaction {
    pub instructions: Vec<Instruction>,
}

/// Represents an instruction in transaction
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum Instruction {
    /// Declares a temporary bucket for later use.
    DeclareTempBucket,

    /// Declares a temporary bucket ref for later use.
    DeclareTempBucketRef,

    /// Takes resource from transaction context to a temporary bucket.
    TakeFromContext {
        amount: Decimal,
        resource_address: Address,
        to: Bid,
    },

    /// Borrows resource from transaction context to a temporary bucket ref.
    ///
    /// A bucket will be created to support the reference and it will stay within the context.
    BorrowFromContext {
        amount: Decimal,
        resource_address: Address,
        to: Rid,
    },

    /// Calls a blueprint function.
    ///
    /// Buckets and bucket refs in arguments moves from transaction context to the callee.
    CallFunction {
        package_address: Address,
        blueprint_name: String,
        function: String,
        args: Vec<Vec<u8>>,
    },

    /// Calls a component method.
    ///
    /// Buckets and bucket refs in arguments moves from transaction context to the callee.
    CallMethod {
        component_address: Address,
        method: String,
        args: Vec<Vec<u8>>,
    },

    /// Drops all bucket refs.
    DropAllBucketRefs,

    /// With method with all resources from transaction context.
    CallMethodWithAllResources {
        component_address: Address,
        method: String,
    },

    /// Marks the end of transaction with signatures.
    /// TODO: replace public key address with signature.
    End { signatures: Vec<Address> },
}
