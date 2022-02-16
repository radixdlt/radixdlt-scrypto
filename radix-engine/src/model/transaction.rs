use sbor::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

/// Represents an unvalidated transaction.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Transaction {
    pub instructions: Vec<Instruction>,
}

/// Represents an unvalidated instruction in transaction
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum Instruction {
    /// Takes fixed amount resource from worktop.
    TakeFromWorktop {
        amount: Decimal,
        resource_address: Address,
    },

    /// Takes all of a given resource from worktop.
    TakeAllFromWorktop { resource_address: Address },

    /// Takes non-fungibles from worktop.
    TakeNonFungiblesFromWorktop {
        keys: BTreeSet<NonFungibleKey>,
        resource_address: Address,
    },

    /// Returns resource to worktop.
    ReturnToWorktop { bid: Bid },

    /// Asserts worktop contains at least this amount.
    AssertWorktopContains {
        amount: Decimal,
        resource_address: Address,
    },

    /// Creates a bucket ref.
    CreateBucketRef { bid: Bid },

    /// Clones a bucket ref.
    CloneBucketRef { rid: Rid },

    /// Drops a bucket ref.
    DropBucketRef { rid: Rid },

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

    /// With method with all resources from transaction context.
    CallMethodWithAllResources {
        component_address: Address,
        method: String,
    },

    /// Marks the end of transaction with signatures.
    /// TODO: replace public key with signature.
    End { signatures: Vec<EcdsaPublicKey> },
}
