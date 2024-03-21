use crate::manifest::token::Span;
use radix_common::data::manifest::{ManifestCustomValueKind, ManifestValueKind};
use strum::{EnumCount, EnumDiscriminants, FromRepr};

#[derive(Debug, Clone, PartialEq, Eq, EnumDiscriminants, EnumCount)]
#[strum_discriminants(derive(FromRepr))]
pub enum Instruction {
    TakeFromWorktop {
        resource_address: ValueWithSpan,
        amount: ValueWithSpan,
        new_bucket: ValueWithSpan,
    },

    TakeNonFungiblesFromWorktop {
        ids: ValueWithSpan,
        resource_address: ValueWithSpan,
        new_bucket: ValueWithSpan,
    },

    TakeAllFromWorktop {
        resource_address: ValueWithSpan,
        new_bucket: ValueWithSpan,
    },

    ReturnToWorktop {
        bucket: ValueWithSpan,
    },

    AssertWorktopContains {
        resource_address: ValueWithSpan,
        amount: ValueWithSpan,
    },

    AssertWorktopContainsNonFungibles {
        resource_address: ValueWithSpan,
        ids: ValueWithSpan,
    },

    AssertWorktopContainsAny {
        resource_address: ValueWithSpan,
    },

    PopFromAuthZone {
        new_proof: ValueWithSpan,
    },

    PushToAuthZone {
        proof: ValueWithSpan,
    },

    CreateProofFromAuthZoneOfAmount {
        resource_address: ValueWithSpan,
        amount: ValueWithSpan,
        new_proof: ValueWithSpan,
    },

    CreateProofFromAuthZoneOfNonFungibles {
        resource_address: ValueWithSpan,
        ids: ValueWithSpan,
        new_proof: ValueWithSpan,
    },

    CreateProofFromAuthZoneOfAll {
        resource_address: ValueWithSpan,
        new_proof: ValueWithSpan,
    },

    DropAuthZoneSignatureProofs,

    DropAuthZoneRegularProofs,

    DropAuthZoneProofs,

    CreateProofFromBucketOfAmount {
        bucket: ValueWithSpan,
        amount: ValueWithSpan,
        new_proof: ValueWithSpan,
    },

    CreateProofFromBucketOfNonFungibles {
        bucket: ValueWithSpan,
        ids: ValueWithSpan,
        new_proof: ValueWithSpan,
    },

    CreateProofFromBucketOfAll {
        bucket: ValueWithSpan,
        new_proof: ValueWithSpan,
    },

    BurnResource {
        bucket: ValueWithSpan,
    },

    CloneProof {
        proof: ValueWithSpan,
        new_proof: ValueWithSpan,
    },

    DropProof {
        proof: ValueWithSpan,
    },

    CallFunction {
        package_address: ValueWithSpan,
        blueprint_name: ValueWithSpan,
        function_name: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },

    CallMethod {
        address: ValueWithSpan,
        method_name: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },

    CallRoyaltyMethod {
        address: ValueWithSpan,
        method_name: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },

    CallMetadataMethod {
        address: ValueWithSpan,
        method_name: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },

    CallRoleAssignmentMethod {
        address: ValueWithSpan,
        method_name: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },

    CallDirectVaultMethod {
        address: ValueWithSpan,
        method_name: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },

    DropNamedProofs,

    DropAllProofs,

    AllocateGlobalAddress {
        package_address: ValueWithSpan,
        blueprint_name: ValueWithSpan,
        address_reservation: ValueWithSpan,
        named_address: ValueWithSpan,
    },

    /* Call direct vault method aliases */
    RecallFromVault {
        vault_id: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    FreezeVault {
        vault_id: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    UnfreezeVault {
        vault_id: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    RecallNonFungiblesFromVault {
        vault_id: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },

    /* Call function aliases */
    PublishPackage {
        args: Vec<ValueWithSpan>,
    },
    PublishPackageAdvanced {
        args: Vec<ValueWithSpan>,
    },
    CreateFungibleResource {
        args: Vec<ValueWithSpan>,
    },
    CreateFungibleResourceWithInitialSupply {
        args: Vec<ValueWithSpan>,
    },
    CreateNonFungibleResource {
        args: Vec<ValueWithSpan>,
    },
    CreateNonFungibleResourceWithInitialSupply {
        args: Vec<ValueWithSpan>,
    },
    CreateAccessController {
        args: Vec<ValueWithSpan>,
    },
    CreateIdentity {
        args: Vec<ValueWithSpan>,
    },
    CreateIdentityAdvanced {
        args: Vec<ValueWithSpan>,
    },
    CreateAccount {
        args: Vec<ValueWithSpan>,
    },
    CreateAccountAdvanced {
        args: Vec<ValueWithSpan>,
    },

    /* call non-main method aliases */
    SetMetadata {
        address: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    RemoveMetadata {
        address: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    LockMetadata {
        address: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    SetComponentRoyalty {
        address: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    SetOwnerRole {
        address: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    LockOwnerRole {
        address: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    SetRole {
        address: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    LockComponentRoyalty {
        address: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    ClaimComponentRoyalties {
        address: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },

    /* call main method aliases */
    ClaimPackageRoyalties {
        address: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    MintFungible {
        address: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    MintNonFungible {
        address: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    MintRuidNonFungible {
        address: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    CreateValidator {
        args: Vec<ValueWithSpan>,
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
    Enum(u8, Vec<ValueWithSpan>),
    Array(ValueKindWithSpan, Vec<ValueWithSpan>),
    Tuple(Vec<ValueWithSpan>),
    Map(
        ValueKindWithSpan,
        ValueKindWithSpan,
        Vec<(ValueWithSpan, ValueWithSpan)>,
    ),

    // ==============
    // Alias values
    // ==============
    Some(Box<ValueWithSpan>),
    None,
    Ok(Box<ValueWithSpan>),
    Err(Box<ValueWithSpan>),
    Bytes(Box<ValueWithSpan>),
    NonFungibleGlobalId(Box<ValueWithSpan>),

    // ==============
    // Custom values
    // ==============
    Address(Box<ValueWithSpan>),
    NamedAddress(Box<ValueWithSpan>),
    Bucket(Box<ValueWithSpan>),
    Proof(Box<ValueWithSpan>),
    Expression(Box<ValueWithSpan>),
    Blob(Box<ValueWithSpan>),
    Decimal(Box<ValueWithSpan>),
    PreciseDecimal(Box<ValueWithSpan>),
    NonFungibleLocalId(Box<ValueWithSpan>),
    AddressReservation(Box<ValueWithSpan>),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueKindWithSpan {
    pub value_kind: ValueKind,
    pub span: Span,
}

/// In case of composite Value variants, eg. Enum, Tuple
/// span information of the delimiters such as (, ) or <, >
/// commas, discriminators is not stored.
/// This is acceptable, since we still are able to produce error messages,
/// which are precise enough.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueWithSpan {
    pub value: Value,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionWithSpan {
    pub instruction: Instruction,
    pub span: Span,
}
