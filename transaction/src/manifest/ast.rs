use radix_engine_interface::data::{ScryptoCustomValueKind, ScryptoValueKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    TakeFromWorktop {
        resource_address: Value,
        new_bucket: Value,
    },

    TakeFromWorktopByAmount {
        amount: Value,
        resource_address: Value,
        new_bucket: Value,
    },

    TakeFromWorktopByIds {
        ids: Value,
        resource_address: Value,
        new_bucket: Value,
    },

    ReturnToWorktop {
        bucket: Value,
    },

    AssertWorktopContains {
        resource_address: Value,
    },

    AssertWorktopContainsByAmount {
        amount: Value,
        resource_address: Value,
    },

    AssertWorktopContainsByIds {
        ids: Value,
        resource_address: Value,
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

    CreateProofFromAuthZoneByAmount {
        amount: Value,
        resource_address: Value,
        new_proof: Value,
    },

    CreateProofFromAuthZoneByIds {
        ids: Value,
        resource_address: Value,
        new_proof: Value,
    },

    CreateProofFromBucket {
        bucket: Value,
        new_proof: Value,
    },

    CloneProof {
        proof: Value,
        new_proof: Value,
    },

    DropProof {
        proof: Value,
    },

    DropAllProofs,

    CallFunction {
        package_address: Value,
        blueprint_name: Value,
        function_name: Value,
        args: Vec<Value>,
    },

    CallMethod {
        component_address: Value,
        method_name: Value,
        args: Vec<Value>,
    },

    PublishPackage {
        code: Value,
        abi: Value,
        royalty_config: Value,
        metadata: Value,
        access_rules: Value,
    },

    PublishPackageWithOwner {
        code: Value,
        abi: Value,
        owner_badge: Value,
    },

    BurnResource {
        bucket: Value,
    },

    // TODO: Dedicated bucket for this?
    RecallResource {
        vault_id: Value,
        amount: Value,
    },

    SetMetadata {
        entity_address: Value,
        key: Value,
        value: Value,
    },

    SetPackageRoyaltyConfig {
        package_address: Value,
        royalty_config: Value,
    },

    SetComponentRoyaltyConfig {
        component_address: Value,
        royalty_config: Value,
    },

    // TODO: Dedicated bucket for this?
    ClaimPackageRoyalty {
        package_address: Value,
    },

    // TODO: Dedicated bucket for this?
    ClaimComponentRoyalty {
        component_address: Value,
    },

    SetMethodAccessRule {
        entity_address: Value,
        index: Value,
        key: Value,
        rule: Value,
    },

    MintFungible {
        resource_address: Value,
        amount: Value,
    },

    MintNonFungible {
        resource_address: Value,
        entries: Value,
    },

    MintUuidNonFungible {
        resource_address: Value,
        entries: Value,
    },

    CreateFungibleResource {
        divisibility: Value,
        metadata: Value,
        access_rules: Value,
        initial_supply: Value,
    },

    CreateFungibleResourceWithOwner {
        divisibility: Value,
        metadata: Value,
        owner_badge: Value,
        initial_supply: Value,
    },

    CreateNonFungibleResource {
        id_type: Value,
        metadata: Value,
        access_rules: Value,
        initial_supply: Value,
    },

    CreateNonFungibleResourceWithOwner {
        id_type: Value,
        metadata: Value,
        owner_badge: Value,
        initial_supply: Value,
    },

    CreateValidator {
        key: Value,
        owner_access_rule: Value,
    },
    CreateAccessController {
        controlled_asset: Value,
        primary_role: Value,
        recovery_role: Value,
        confirmation_role: Value,
        timed_recovery_delay_in_minutes: Value,
    },
    CreateIdentity {
        access_rule: Value,
    },

    AssertAccessRule {
        access_rule: Value,
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

    // ==============
    // Custom Types
    // ==============

    // RE interpreted types
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
    Own,

    // TX interpreted types
    Bucket,
    Proof,
    Expression,
    Blob,

    // Uninterpreted,
    Hash,
    EcdsaSecp256k1PublicKey,
    EcdsaSecp256k1Signature,
    EddsaEd25519PublicKey,
    EddsaEd25519Signature,
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,
}

impl Type {
    pub fn value_kind(&self) -> ScryptoValueKind {
        match self {
            Type::Bool => ScryptoValueKind::Bool,
            Type::I8 => ScryptoValueKind::I8,
            Type::I16 => ScryptoValueKind::I16,
            Type::I32 => ScryptoValueKind::I32,
            Type::I64 => ScryptoValueKind::I64,
            Type::I128 => ScryptoValueKind::I128,
            Type::U8 => ScryptoValueKind::U8,
            Type::U16 => ScryptoValueKind::U16,
            Type::U32 => ScryptoValueKind::U32,
            Type::U64 => ScryptoValueKind::U64,
            Type::U128 => ScryptoValueKind::U128,
            Type::String => ScryptoValueKind::String,
            Type::Enum => ScryptoValueKind::Enum,
            Type::Array => ScryptoValueKind::Array,
            Type::Tuple => ScryptoValueKind::Tuple,

            // Aliases
            Type::Bytes => ScryptoValueKind::Array,
            Type::NonFungibleGlobalId => ScryptoValueKind::Tuple,

            // RE interpreted types
            Type::PackageAddress => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::PackageAddress)
            }
            Type::ComponentAddress => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::ComponentAddress)
            }
            Type::ResourceAddress => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::ResourceAddress)
            }
            Type::Own => ScryptoValueKind::Custom(ScryptoCustomValueKind::Own),

            // Tx interpreted types
            Type::Bucket => ScryptoValueKind::Custom(ScryptoCustomValueKind::Bucket),
            Type::Proof => ScryptoValueKind::Custom(ScryptoCustomValueKind::Proof),
            Type::Expression => ScryptoValueKind::Custom(ScryptoCustomValueKind::Expression),
            Type::Blob => ScryptoValueKind::Custom(ScryptoCustomValueKind::Blob),

            // Uninterpreted
            Type::Hash => ScryptoValueKind::Custom(ScryptoCustomValueKind::Hash),
            Type::EcdsaSecp256k1PublicKey => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::EcdsaSecp256k1PublicKey)
            }
            Type::EcdsaSecp256k1Signature => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::EcdsaSecp256k1Signature)
            }
            Type::EddsaEd25519PublicKey => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::EddsaEd25519PublicKey)
            }
            Type::EddsaEd25519Signature => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::EddsaEd25519Signature)
            }
            Type::Decimal => ScryptoValueKind::Custom(ScryptoCustomValueKind::Decimal),
            Type::PreciseDecimal => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::PreciseDecimal)
            }
            Type::NonFungibleLocalId => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::NonFungibleLocalId)
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

    // RE interpreted types
    PackageAddress(Box<Value>),
    ComponentAddress(Box<Value>),
    ResourceAddress(Box<Value>),
    Own(Box<Value>),

    // TX interpreted types
    Bucket(Box<Value>),
    Proof(Box<Value>),
    Expression(Box<Value>),
    Blob(Box<Value>),

    // Uninterpreted,
    Hash(Box<Value>),
    EcdsaSecp256k1PublicKey(Box<Value>),
    EcdsaSecp256k1Signature(Box<Value>),
    EddsaEd25519PublicKey(Box<Value>),
    EddsaEd25519Signature(Box<Value>),
    Decimal(Box<Value>),
    PreciseDecimal(Box<Value>),
    NonFungibleLocalId(Box<Value>),
}

impl Value {
    pub const fn value_kind(&self) -> ScryptoValueKind {
        match self {
            // ==============
            // Basic Types
            // ==============
            Value::Bool(_) => ScryptoValueKind::Bool,
            Value::I8(_) => ScryptoValueKind::I8,
            Value::I16(_) => ScryptoValueKind::I16,
            Value::I32(_) => ScryptoValueKind::I32,
            Value::I64(_) => ScryptoValueKind::I64,
            Value::I128(_) => ScryptoValueKind::I128,
            Value::U8(_) => ScryptoValueKind::U8,
            Value::U16(_) => ScryptoValueKind::U16,
            Value::U32(_) => ScryptoValueKind::U32,
            Value::U64(_) => ScryptoValueKind::U64,
            Value::U128(_) => ScryptoValueKind::U128,
            Value::String(_) => ScryptoValueKind::String,
            Value::Enum(_, _) => ScryptoValueKind::Enum,
            Value::Array(_, _) => ScryptoValueKind::Array,
            Value::Tuple(_) => ScryptoValueKind::Tuple,
            Value::Map(_, _, _) => ScryptoValueKind::Map,

            // ==============
            // Aliases
            // ==============
            Value::Some(_) => ScryptoValueKind::Enum,
            Value::None => ScryptoValueKind::Enum,
            Value::Ok(_) => ScryptoValueKind::Enum,
            Value::Err(_) => ScryptoValueKind::Enum,
            Value::Bytes(_) => ScryptoValueKind::Array,
            Value::NonFungibleGlobalId(_) => ScryptoValueKind::Tuple,

            // ==============
            // Custom Types
            // ==============

            // RE interpreted
            Value::PackageAddress(_) => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::PackageAddress)
            }
            Value::ComponentAddress(_) => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::ComponentAddress)
            }
            Value::ResourceAddress(_) => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::ResourceAddress)
            }
            Value::Own(_) => ScryptoValueKind::Custom(ScryptoCustomValueKind::Own),

            // TX interpreted
            Value::Bucket(_) => ScryptoValueKind::Custom(ScryptoCustomValueKind::Bucket),
            Value::Proof(_) => ScryptoValueKind::Custom(ScryptoCustomValueKind::Proof),
            Value::Expression(_) => ScryptoValueKind::Custom(ScryptoCustomValueKind::Expression),
            Value::Blob(_) => ScryptoValueKind::Custom(ScryptoCustomValueKind::Blob),

            // Uninterpreted,
            Value::Hash(_) => ScryptoValueKind::Custom(ScryptoCustomValueKind::Hash),
            Value::EcdsaSecp256k1PublicKey(_) => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::EcdsaSecp256k1PublicKey)
            }
            Value::EcdsaSecp256k1Signature(_) => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::EcdsaSecp256k1Signature)
            }
            Value::EddsaEd25519PublicKey(_) => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::EddsaEd25519PublicKey)
            }
            Value::EddsaEd25519Signature(_) => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::EddsaEd25519Signature)
            }
            Value::Decimal(_) => ScryptoValueKind::Custom(ScryptoCustomValueKind::Decimal),
            Value::PreciseDecimal(_) => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::PreciseDecimal)
            }
            Value::NonFungibleLocalId(_) => {
                ScryptoValueKind::Custom(ScryptoCustomValueKind::NonFungibleLocalId)
            }
        }
    }
}
