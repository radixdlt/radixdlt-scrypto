use radix_engine_interface::data::manifest::{ManifestCustomValueKind, ManifestValueKind};
#[cfg(feature = "radix_engine_fuzzing")]
use strum_macros::EnumCount;

#[cfg_attr(feature = "radix_engine_fuzzing", derive(EnumCount))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    TakeFromWorktop {
        resource_address: Value,
        amount: Value,
        new_bucket: Value,
    },

    TakeNonFungiblesFromWorktop {
        ids: Value,
        resource_address: Value,
        new_bucket: Value,
    },

    TakeAllFromWorktop {
        resource_address: Value,
        new_bucket: Value,
    },

    ReturnToWorktop {
        bucket: Value,
    },

    AssertWorktopContains {
        resource_address: Value,
        amount: Value,
    },

    AssertWorktopContainsNonFungibles {
        resource_address: Value,
        ids: Value,
    },

    PopFromAuthZone {
        new_proof: Value,
    },

    PushToAuthZone {
        proof: Value,
    },

    ClearAuthZone,

    CreateProofFromAuthZone {
        resource_address: Value,
        new_proof: Value,
    },

    CreateProofFromAuthZoneOfAmount {
        resource_address: Value,
        amount: Value,
        new_proof: Value,
    },

    CreateProofFromAuthZoneOfNonFungibles {
        resource_address: Value,
        ids: Value,
        new_proof: Value,
    },

    CreateProofFromAuthZoneOfAll {
        resource_address: Value,
        new_proof: Value,
    },

    ClearSignatureProofs,

    CreateProofFromBucket {
        bucket: Value,
        new_proof: Value,
    },

    CreateProofFromBucketOfAmount {
        bucket: Value,
        amount: Value,
        new_proof: Value,
    },

    CreateProofFromBucketOfNonFungibles {
        bucket: Value,
        ids: Value,
        new_proof: Value,
    },

    CreateProofFromBucketOfAll {
        bucket: Value,
        new_proof: Value,
    },

    BurnResource {
        bucket: Value,
    },

    CloneProof {
        proof: Value,
        new_proof: Value,
    },

    DropProof {
        proof: Value,
    },

    CallFunction {
        package_address: Value,
        blueprint_name: Value,
        function_name: Value,
        args: Vec<Value>,
    },

    CallMethod {
        address: Value,
        method_name: Value,
        args: Vec<Value>,
    },

    CallRoyaltyMethod {
        address: Value,
        method_name: Value,
        args: Vec<Value>,
    },

    CallMetadataMethod {
        address: Value,
        method_name: Value,
        args: Vec<Value>,
    },

    CallAccessRulesMethod {
        address: Value,
        method_name: Value,
        args: Vec<Value>,
    },

    DropAllProofs,

    AllocateGlobalAddress {
        package_address: Value,
        blueprint_name: Value,
        address_reservation: Value,
        named_address: Value,
    },

    /* Call direct vault method aliases */
    RecallFromVault {
        vault_id: Value,
        args: Vec<Value>,
    },
    FreezeVault {
        vault_id: Value,
        args: Vec<Value>,
    },
    UnfreezeVault {
        vault_id: Value,
        args: Vec<Value>,
    },

    /* Call function aliases */
    PublishPackage {
        args: Vec<Value>,
    },
    PublishPackageAdvanced {
        args: Vec<Value>,
    },
    CreateFungibleResource {
        args: Vec<Value>,
    },
    CreateFungibleResourceWithInitialSupply {
        args: Vec<Value>,
    },
    CreateNonFungibleResource {
        args: Vec<Value>,
    },
    CreateNonFungibleResourceWithInitialSupply {
        args: Vec<Value>,
    },
    CreateAccessController {
        args: Vec<Value>,
    },
    CreateIdentity {
        args: Vec<Value>,
    },
    CreateIdentityAdvanced {
        args: Vec<Value>,
    },
    CreateAccount {
        args: Vec<Value>,
    },
    CreateAccountAdvanced {
        args: Vec<Value>,
    },

    /* call non-main method aliases */
    SetMetadata {
        address: Value,
        args: Vec<Value>,
    },
    RemoveMetadata {
        address: Value,
        args: Vec<Value>,
    },
    LockMetadata {
        address: Value,
        args: Vec<Value>,
    },
    SetComponentRoyalty {
        address: Value,
        args: Vec<Value>,
    },
    SetOwnerRole {
        address: Value,
        args: Vec<Value>,
    },
    LockOwnerRole {
        address: Value,
        args: Vec<Value>,
    },
    SetAndLockOwnerRole {
        address: Value,
        args: Vec<Value>,
    },
    SetRole {
        address: Value,
        args: Vec<Value>,
    },
    LockRole {
        address: Value,
        args: Vec<Value>,
    },
    SetAndLockRole {
        address: Value,
        args: Vec<Value>,
    },
    LockComponentRoyalty {
        address: Value,
        args: Vec<Value>,
    },
    ClaimComponentRoyalties {
        address: Value,
        args: Vec<Value>,
    },

    /* call main method aliases */
    ClaimPackageRoyalties {
        address: Value,
        args: Vec<Value>,
    },
    MintFungible {
        address: Value,
        args: Vec<Value>,
    },
    MintNonFungible {
        address: Value,
        args: Vec<Value>,
    },
    MintRuidNonFungible {
        address: Value,
        args: Vec<Value>,
    },
    CreateValidator {
        args: Vec<Value>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    // ==============
    // Simple basic value kinds
    // ==============
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    String,

    // ==============
    // Composite basic value kinds
    // ==============
    Enum,
    Array,
    Tuple,
    Map,

    // ==============
    // Value kind aliases
    // ==============
    Bytes,
    NonFungibleGlobalId,
    PackageAddress,
    ComponentAddress,
    ResourceAddress,

    // ==============
    // Custom value kinds
    // ==============
    Address,
    Bucket,
    Proof,
    Expression,
    Blob,
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,
    AddressReservation,
    NamedAddress,
}

impl ValueKind {
    pub fn value_kind(&self) -> ManifestValueKind {
        match self {
            // ==============
            // Simple basic value kinds
            // ==============
            ValueKind::Bool => ManifestValueKind::Bool,
            ValueKind::I8 => ManifestValueKind::I8,
            ValueKind::I16 => ManifestValueKind::I16,
            ValueKind::I32 => ManifestValueKind::I32,
            ValueKind::I64 => ManifestValueKind::I64,
            ValueKind::I128 => ManifestValueKind::I128,
            ValueKind::U8 => ManifestValueKind::U8,
            ValueKind::U16 => ManifestValueKind::U16,
            ValueKind::U32 => ManifestValueKind::U32,
            ValueKind::U64 => ManifestValueKind::U64,
            ValueKind::U128 => ManifestValueKind::U128,
            ValueKind::String => ManifestValueKind::String,

            // ==============
            // Composite basic value kinds
            // ==============
            ValueKind::Enum => ManifestValueKind::Enum,
            ValueKind::Array => ManifestValueKind::Array,
            ValueKind::Tuple => ManifestValueKind::Tuple,
            ValueKind::Map => ManifestValueKind::Map,

            // ==============
            // Value kind aliases
            // ==============
            ValueKind::Bytes => ManifestValueKind::Array,
            ValueKind::NonFungibleGlobalId => ManifestValueKind::Tuple,
            ValueKind::PackageAddress => {
                ManifestValueKind::Custom(ManifestCustomValueKind::Address)
            }
            ValueKind::ComponentAddress => {
                ManifestValueKind::Custom(ManifestCustomValueKind::Address)
            }
            ValueKind::ResourceAddress => {
                ManifestValueKind::Custom(ManifestCustomValueKind::Address)
            }

            // ==============
            // Custom value kinds
            // ==============
            ValueKind::Address => ManifestValueKind::Custom(ManifestCustomValueKind::Address),
            ValueKind::NamedAddress => ManifestValueKind::Custom(ManifestCustomValueKind::Address),
            ValueKind::Bucket => ManifestValueKind::Custom(ManifestCustomValueKind::Bucket),
            ValueKind::Proof => ManifestValueKind::Custom(ManifestCustomValueKind::Proof),
            ValueKind::Expression => ManifestValueKind::Custom(ManifestCustomValueKind::Expression),
            ValueKind::Blob => ManifestValueKind::Custom(ManifestCustomValueKind::Blob),
            ValueKind::Decimal => ManifestValueKind::Custom(ManifestCustomValueKind::Decimal),
            ValueKind::PreciseDecimal => {
                ManifestValueKind::Custom(ManifestCustomValueKind::PreciseDecimal)
            }
            ValueKind::NonFungibleLocalId => {
                ManifestValueKind::Custom(ManifestCustomValueKind::NonFungibleLocalId)
            }
            ValueKind::AddressReservation => {
                ManifestValueKind::Custom(ManifestCustomValueKind::AddressReservation)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    // ==============
    // Basic values
    // ==============
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    String(String),

    // ==============
    // Composite basic values
    // ==============
    Enum(u8, Vec<Value>),
    Array(ValueKind, Vec<Value>),
    Tuple(Vec<Value>),
    Map(ValueKind, ValueKind, Vec<(Value, Value)>),

    // ==============
    // Alias values
    // ==============
    Some(Box<Value>),
    None,
    Ok(Box<Value>),
    Err(Box<Value>),
    Bytes(Box<Value>),
    NonFungibleGlobalId(Box<Value>),

    // ==============
    // Custom values
    // ==============
    Address(Box<Value>),
    NamedAddress(Box<Value>),
    Bucket(Box<Value>),
    Proof(Box<Value>),
    Expression(Box<Value>),
    Blob(Box<Value>),
    Decimal(Box<Value>),
    PreciseDecimal(Box<Value>),
    NonFungibleLocalId(Box<Value>),
    AddressReservation(Box<Value>),
}

impl Value {
    pub const fn value_kind(&self) -> ManifestValueKind {
        match self {
            // ==============
            // Basic values
            // ==============
            Value::Bool(_) => ManifestValueKind::Bool,
            Value::I8(_) => ManifestValueKind::I8,
            Value::I16(_) => ManifestValueKind::I16,
            Value::I32(_) => ManifestValueKind::I32,
            Value::I64(_) => ManifestValueKind::I64,
            Value::I128(_) => ManifestValueKind::I128,
            Value::U8(_) => ManifestValueKind::U8,
            Value::U16(_) => ManifestValueKind::U16,
            Value::U32(_) => ManifestValueKind::U32,
            Value::U64(_) => ManifestValueKind::U64,
            Value::U128(_) => ManifestValueKind::U128,
            Value::String(_) => ManifestValueKind::String,
            Value::Enum(_, _) => ManifestValueKind::Enum,
            Value::Array(_, _) => ManifestValueKind::Array,
            Value::Tuple(_) => ManifestValueKind::Tuple,
            Value::Map(_, _, _) => ManifestValueKind::Map,

            // ==============
            // Aliase values
            // ==============
            Value::Some(_) => ManifestValueKind::Enum,
            Value::None => ManifestValueKind::Enum,
            Value::Ok(_) => ManifestValueKind::Enum,
            Value::Err(_) => ManifestValueKind::Enum,
            Value::Bytes(_) => ManifestValueKind::Array,
            Value::NonFungibleGlobalId(_) => ManifestValueKind::Tuple,

            // ==============
            // Custom values
            // ==============
            Value::Address(_) => ManifestValueKind::Custom(ManifestCustomValueKind::Address),
            Value::NamedAddress(_) => ManifestValueKind::Custom(ManifestCustomValueKind::Address),
            Value::Bucket(_) => ManifestValueKind::Custom(ManifestCustomValueKind::Bucket),
            Value::Proof(_) => ManifestValueKind::Custom(ManifestCustomValueKind::Proof),
            Value::Expression(_) => ManifestValueKind::Custom(ManifestCustomValueKind::Expression),
            Value::Blob(_) => ManifestValueKind::Custom(ManifestCustomValueKind::Blob),
            Value::Decimal(_) => ManifestValueKind::Custom(ManifestCustomValueKind::Decimal),
            Value::PreciseDecimal(_) => {
                ManifestValueKind::Custom(ManifestCustomValueKind::PreciseDecimal)
            }
            Value::NonFungibleLocalId(_) => {
                ManifestValueKind::Custom(ManifestCustomValueKind::NonFungibleLocalId)
            }
            Value::AddressReservation(_) => {
                ManifestValueKind::Custom(ManifestCustomValueKind::AddressReservation)
            }
        }
    }
}
