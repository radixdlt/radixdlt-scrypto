use crate::manifest::token::Span;
use radix_common::data::manifest::{ManifestCustomValueKind, ManifestValueKind};
use strum::{EnumCount, EnumDiscriminants, FromRepr};

use super::generator::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumDiscriminants, EnumCount)]
#[strum_discriminants(derive(FromRepr))]
pub enum Instruction {
    //========================================
    // PSEUDO-INSTRUCTIONS AT THE START
    //========================================
    UsePreallocatedAddress {
        package_address: ValueWithSpan,
        blueprint_name: ValueWithSpan,
        address_reservation: ValueWithSpan,
        preallocated_address: ValueWithSpan,
    },

    UseChild {
        named_intent: ValueWithSpan,
        subintent_hash: ValueWithSpan,
    },

    //========================================
    // NORMAL INSTRUCTIONS
    //========================================

    // Bucket Lifecycle
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
    BurnResource {
        bucket: ValueWithSpan,
    },

    // Resource Assertions
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
    AssertWorktopIsEmpty, // Alias
    AssertWorktopResourcesOnly {
        constraints: ValueWithSpan,
    },
    AssertWorktopResourcesInclude {
        constraints: ValueWithSpan,
    },
    AssertNextCallReturnsOnly {
        constraints: ValueWithSpan,
    },
    AssertNextCallReturnsInclude {
        constraints: ValueWithSpan,
    },
    AssertBucketContents {
        bucket: ValueWithSpan,
        constraint: ValueWithSpan,
    },

    // Proof Lifecycle
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
    CloneProof {
        proof: ValueWithSpan,
        new_proof: ValueWithSpan,
    },
    DropProof {
        proof: ValueWithSpan,
    },
    PushToAuthZone {
        proof: ValueWithSpan,
    },
    PopFromAuthZone {
        new_proof: ValueWithSpan,
    },
    DropAuthZoneSignatureProofs,
    DropAuthZoneRegularProofs,
    DropAuthZoneProofs,
    DropNamedProofs,
    DropAllProofs,

    // Invocations
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

    // Address Allocation
    AllocateGlobalAddress {
        package_address: ValueWithSpan,
        blueprint_name: ValueWithSpan,
        address_reservation: ValueWithSpan,
        named_address: ValueWithSpan,
    },

    // Interaction with other intents
    YieldToParent {
        args: Vec<ValueWithSpan>,
    },
    YieldToChild {
        child: ValueWithSpan,
        args: Vec<ValueWithSpan>,
    },
    VerifyParent {
        access_rule: ValueWithSpan,
    },

    //===============
    // Direct vault aliases
    //===============
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

    //===============
    // Call function aliases
    //===============
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

    //===============
    // Non-main method aliases
    //===============
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

    //===============
    // Main method aliases
    //===============
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

/// This represents a slightly wider range of possibilities
/// than an SBOR ManifestValueKind, including aliases and
/// string-manifest-syntax-specific value kinds.
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

    // ==============
    // Pretend value kinds
    // ==============
    Intent,
    NamedIntent,
}

impl ValueKind {
    pub fn from_ident(ident: &str) -> Option<Self> {
        let value_kind = match ident {
            // ==============
            // Basic simple types
            // ==============
            "Bool" => Self::Bool,
            "I8" => Self::I8,
            "I16" => Self::I16,
            "I32" => Self::I32,
            "I64" => Self::I64,
            "I128" => Self::I128,
            "U8" => Self::U8,
            "U16" => Self::U16,
            "U32" => Self::U32,
            "U64" => Self::U64,
            "U128" => Self::U128,
            "String" => Self::String,

            // ==============
            // Basic composite types
            // ==============
            "Enum" => Self::Enum,
            "Array" => Self::Array,
            "Tuple" => Self::Tuple,
            "Map" => Self::Map,

            // ==============
            // Value kind aliases
            // ==============
            "Bytes" => Self::Bytes,
            "NonFungibleGlobalId" => Self::NonFungibleGlobalId,

            // ==============
            // Custom types
            // ==============
            "Address" => Self::Address,
            "Bucket" => Self::Bucket,
            "Proof" => Self::Proof,
            "Expression" => Self::Expression,
            "Blob" => Self::Blob,
            "Decimal" => Self::Decimal,
            "PreciseDecimal" => Self::PreciseDecimal,
            "NonFungibleLocalId" => Self::NonFungibleLocalId,
            "AddressReservation" => Self::AddressReservation,
            "NamedAddress" => Self::NamedAddress,
            "Intent" => Self::Intent,
            "NamedIntent" => Self::NamedIntent,

            _ => {
                return None;
            }
        };
        Some(value_kind)
    }
}

impl core::fmt::Display for ValueKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
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
    Intent(Box<ValueWithSpan>),
    NamedIntent(Box<ValueWithSpan>),
}

impl Value {
    pub const fn value_kind(&self) -> ValueKind {
        match self {
            // ==============
            // Basic values
            // ==============
            Value::Bool(_) => ValueKind::Bool,
            Value::I8(_) => ValueKind::I8,
            Value::I16(_) => ValueKind::I16,
            Value::I32(_) => ValueKind::I32,
            Value::I64(_) => ValueKind::I64,
            Value::I128(_) => ValueKind::I128,
            Value::U8(_) => ValueKind::U8,
            Value::U16(_) => ValueKind::U16,
            Value::U32(_) => ValueKind::U32,
            Value::U64(_) => ValueKind::U64,
            Value::U128(_) => ValueKind::U128,
            Value::String(_) => ValueKind::String,
            Value::Enum(_, _) => ValueKind::Enum,
            Value::Array(_, _) => ValueKind::Array,
            Value::Tuple(_) => ValueKind::Tuple,
            Value::Map(_, _, _) => ValueKind::Map,

            // ==============
            // Alias values
            // ==============
            Value::Some(_) => ValueKind::Enum,
            Value::None => ValueKind::Enum,
            Value::Ok(_) => ValueKind::Enum,
            Value::Err(_) => ValueKind::Enum,
            Value::Bytes(_) => ValueKind::Bytes,
            Value::NonFungibleGlobalId(_) => ValueKind::NonFungibleGlobalId,

            // ==============
            // Custom values
            // ==============
            Value::Address(_) => ValueKind::Address,
            Value::NamedAddress(_) => ValueKind::NamedAddress,
            Value::Bucket(_) => ValueKind::Bucket,
            Value::Proof(_) => ValueKind::Proof,
            Value::Expression(_) => ValueKind::Expression,
            Value::Blob(_) => ValueKind::Blob,
            Value::Decimal(_) => ValueKind::Decimal,
            Value::PreciseDecimal(_) => ValueKind::PreciseDecimal,
            Value::NonFungibleLocalId(_) => ValueKind::NonFungibleLocalId,
            Value::AddressReservation(_) => ValueKind::AddressReservation,
            Value::Intent(_) => ValueKind::Intent,
            Value::NamedIntent(_) => ValueKind::NamedIntent,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueKindWithSpan {
    pub value_kind: ValueKind,
    pub span: Span,
}

impl ValueKindWithSpan {
    pub fn sbor_value_kind(&self) -> Result<ManifestValueKind, GeneratorError> {
        let value_kind = match self.value_kind {
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
            ValueKind::NamedIntent => {
                return Err(GeneratorError {
                    span: self.span,
                    error_kind: GeneratorErrorKind::NamedIntentCannotBeUsedAsValueKind,
                })
            }
            ValueKind::Intent => {
                return Err(GeneratorError {
                    span: self.span,
                    error_kind: GeneratorErrorKind::IntentCannotBeUsedAsValueKind,
                })
            }
        };
        Ok(value_kind)
    }
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

impl ValueWithSpan {
    pub fn value_kind(&self) -> ValueKindWithSpan {
        ValueKindWithSpan {
            value_kind: self.value.value_kind(),
            span: self.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionWithSpan {
    pub instruction: Instruction,
    pub span: Span,
}
