use radix_engine_interface::data::manifest::{ManifestCustomValueKind, ManifestValueKind};

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

    RecallResource {
        vault_id: Value,
        amount: Value,
    },

    DropAllProofs,

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

    /* Call method aliases */
    SetMetadata {
        address: Value,
        args: Vec<Value>,
    },
    RemoveMetadata {
        address: Value,
        args: Vec<Value>,
    },
    SetPackageRoyaltyConfig {
        address: Value,
        args: Vec<Value>,
    },
    SetComponentRoyaltyConfig {
        address: Value,
        args: Vec<Value>,
    },
    ClaimPackageRoyalty {
        address: Value,
        args: Vec<Value>,
    },
    ClaimComponentRoyalty {
        address: Value,
        args: Vec<Value>,
    },
    SetMethodAccessRule {
        address: Value,
        args: Vec<Value>,
    },
    SetGroupAccessRule {
        address: Value,
        args: Vec<Value>,
    },
    SetGroupMutability {
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
    MintUuidNonFungible {
        address: Value,
        args: Vec<Value>,
    },
    CreateValidator {
        address: Value,
        args: Vec<Value>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    /* Rust types */
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

    /* Struct and enum */
    Enum,

    /* [T; N] and (A, B, C) */
    Array,
    Tuple,

    // ==============
    // Alias
    // ==============
    Bytes,
    NonFungibleGlobalId,
    PackageAddress,
    ComponentAddress,
    ResourceAddress,

    // ==============
    // Custom Types
    // ==============
    Address,
    Bucket,
    Proof,
    Expression,
    Blob,

    // Uninterpreted,
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,
}

impl Type {
    pub fn value_kind(&self) -> ManifestValueKind {
        match self {
            Type::Bool => ManifestValueKind::Bool,
            Type::I8 => ManifestValueKind::I8,
            Type::I16 => ManifestValueKind::I16,
            Type::I32 => ManifestValueKind::I32,
            Type::I64 => ManifestValueKind::I64,
            Type::I128 => ManifestValueKind::I128,
            Type::U8 => ManifestValueKind::U8,
            Type::U16 => ManifestValueKind::U16,
            Type::U32 => ManifestValueKind::U32,
            Type::U64 => ManifestValueKind::U64,
            Type::U128 => ManifestValueKind::U128,
            Type::String => ManifestValueKind::String,
            Type::Enum => ManifestValueKind::Enum,
            Type::Array => ManifestValueKind::Array,
            Type::Tuple => ManifestValueKind::Tuple,

            // Aliases
            Type::Bytes => ManifestValueKind::Array,
            Type::NonFungibleGlobalId => ManifestValueKind::Tuple,
            Type::PackageAddress => ManifestValueKind::Custom(ManifestCustomValueKind::Address),
            Type::ComponentAddress => ManifestValueKind::Custom(ManifestCustomValueKind::Address),
            Type::ResourceAddress => ManifestValueKind::Custom(ManifestCustomValueKind::Address),

            // Custom types
            Type::Address => ManifestValueKind::Custom(ManifestCustomValueKind::Address),
            Type::Bucket => ManifestValueKind::Custom(ManifestCustomValueKind::Bucket),
            Type::Proof => ManifestValueKind::Custom(ManifestCustomValueKind::Proof),
            Type::Expression => ManifestValueKind::Custom(ManifestCustomValueKind::Expression),
            Type::Blob => ManifestValueKind::Custom(ManifestCustomValueKind::Blob),
            Type::Decimal => ManifestValueKind::Custom(ManifestCustomValueKind::Decimal),
            Type::PreciseDecimal => {
                ManifestValueKind::Custom(ManifestCustomValueKind::PreciseDecimal)
            }
            Type::NonFungibleLocalId => {
                ManifestValueKind::Custom(ManifestCustomValueKind::NonFungibleLocalId)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    // ==============
    // Basic Types
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

    Enum(u8, Vec<Value>),
    Array(Type, Vec<Value>),
    Tuple(Vec<Value>),
    Map(Type, Type, Vec<Value>),

    // ==============
    // Aliases
    // ==============
    Some(Box<Value>),
    None,
    Ok(Box<Value>),
    Err(Box<Value>),
    Bytes(Box<Value>),
    NonFungibleGlobalId(Box<Value>),

    // ==============
    // Custom Types
    // ==============
    Address(Box<Value>),
    Bucket(Box<Value>),
    Proof(Box<Value>),
    Expression(Box<Value>),
    Blob(Box<Value>),
    Decimal(Box<Value>),
    PreciseDecimal(Box<Value>),
    NonFungibleLocalId(Box<Value>),
}

impl Value {
    pub const fn value_kind(&self) -> ManifestValueKind {
        match self {
            // ==============
            // Basic Types
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
            // Aliases
            // ==============
            Value::Some(_) => ManifestValueKind::Enum,
            Value::None => ManifestValueKind::Enum,
            Value::Ok(_) => ManifestValueKind::Enum,
            Value::Err(_) => ManifestValueKind::Enum,
            Value::Bytes(_) => ManifestValueKind::Array,
            Value::NonFungibleGlobalId(_) => ManifestValueKind::Tuple,

            // ==============
            // Custom Types
            // ==============
            Value::Address(_) => ManifestValueKind::Custom(ManifestCustomValueKind::Address),
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
        }
    }
}
