use radix_engine_interface::data::{ScryptoCustomTypeId, ScryptoSborTypeId};

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    /* Rust types */
    Unit,
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
    NonFungibleAddress,

    // ==============
    // Custom Types
    // ==============

    // RE global address types
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
    SystemAddress,

    // RE interpreted types
    Own,
    Blob,

    // TX interpreted types
    Bucket,
    Proof,
    Expression,

    // Uninterpreted,
    Hash,
    EcdsaSecp256k1PublicKey,
    EcdsaSecp256k1Signature,
    EddsaEd25519PublicKey,
    EddsaEd25519Signature,
    Decimal,
    PreciseDecimal,
    NonFungibleId,
}

impl Type {
    pub fn type_id(&self) -> ScryptoSborTypeId {
        match self {
            Type::Unit => ScryptoSborTypeId::Unit,
            Type::Bool => ScryptoSborTypeId::Bool,
            Type::I8 => ScryptoSborTypeId::I8,
            Type::I16 => ScryptoSborTypeId::I16,
            Type::I32 => ScryptoSborTypeId::I32,
            Type::I64 => ScryptoSborTypeId::I64,
            Type::I128 => ScryptoSborTypeId::I128,
            Type::U8 => ScryptoSborTypeId::U8,
            Type::U16 => ScryptoSborTypeId::U16,
            Type::U32 => ScryptoSborTypeId::U32,
            Type::U64 => ScryptoSborTypeId::U64,
            Type::U128 => ScryptoSborTypeId::U128,
            Type::String => ScryptoSborTypeId::String,
            Type::Enum => ScryptoSborTypeId::Enum,
            Type::Array => ScryptoSborTypeId::Array,
            Type::Tuple => ScryptoSborTypeId::Tuple,

            // Aliases
            Type::Bytes => ScryptoSborTypeId::Array,
            Type::NonFungibleAddress => ScryptoSborTypeId::Tuple,

            // RE global address types
            Type::PackageAddress => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::PackageAddress),
            Type::ComponentAddress => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::ComponentAddress)
            }
            Type::ResourceAddress => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::ResourceAddress)
            }
            Type::SystemAddress => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::SystemAddress),

            // RE interpreted types
            Type::Own => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Own),
            Type::Blob => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Blob),

            // Tx interpreted types
            Type::Bucket => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Bucket),
            Type::Proof => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Proof),
            Type::Expression => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Expression),

            // Uninterpreted
            Type::Hash => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Hash),
            Type::EcdsaSecp256k1PublicKey => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::EcdsaSecp256k1PublicKey)
            }
            Type::EcdsaSecp256k1Signature => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::EcdsaSecp256k1Signature)
            }
            Type::EddsaEd25519PublicKey => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::EddsaEd25519PublicKey)
            }
            Type::EddsaEd25519Signature => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::EddsaEd25519Signature)
            }
            Type::Decimal => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Decimal),
            Type::PreciseDecimal => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::PreciseDecimal),
            Type::NonFungibleId => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::NonFungibleId),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    // ==============
    // Basic Types
    // ==============
    Unit,
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

    Enum(String, Vec<Value>),
    Array(Type, Vec<Value>),
    Tuple(Vec<Value>),

    // ==============
    // Aliases
    // ==============
    Some(Box<Value>),
    None,
    Ok(Box<Value>),
    Err(Box<Value>),
    Bytes(Box<Value>),
    NonFungibleAddress(Box<Value>, Box<Value>),

    // ==============
    // Custom Types
    // ==============

    // Globals
    PackageAddress(Box<Value>),
    ComponentAddress(Box<Value>),
    ResourceAddress(Box<Value>),
    SystemAddress(Box<Value>),

    // RE interpreted types
    Own(Box<Value>),
    Blob(Box<Value>),

    // TX interpreted types
    Bucket(Box<Value>),
    Proof(Box<Value>),
    Expression(Box<Value>),

    // Uninterpreted,
    Hash(Box<Value>),
    EcdsaSecp256k1PublicKey(Box<Value>),
    EcdsaSecp256k1Signature(Box<Value>),
    EddsaEd25519PublicKey(Box<Value>),
    EddsaEd25519Signature(Box<Value>),
    Decimal(Box<Value>),
    PreciseDecimal(Box<Value>),
    NonFungibleId(Box<Value>),
}

impl Value {
    pub const fn type_id(&self) -> ScryptoSborTypeId {
        match self {
            // ==============
            // Basic Types
            // ==============
            Value::Unit => ScryptoSborTypeId::Unit,
            Value::Bool(_) => ScryptoSborTypeId::Bool,
            Value::I8(_) => ScryptoSborTypeId::I8,
            Value::I16(_) => ScryptoSborTypeId::I16,
            Value::I32(_) => ScryptoSborTypeId::I32,
            Value::I64(_) => ScryptoSborTypeId::I64,
            Value::I128(_) => ScryptoSborTypeId::I128,
            Value::U8(_) => ScryptoSborTypeId::U8,
            Value::U16(_) => ScryptoSborTypeId::U16,
            Value::U32(_) => ScryptoSborTypeId::U32,
            Value::U64(_) => ScryptoSborTypeId::U64,
            Value::U128(_) => ScryptoSborTypeId::U128,
            Value::String(_) => ScryptoSborTypeId::String,
            Value::Enum(_, _) => ScryptoSborTypeId::Enum,
            Value::Array(_, _) => ScryptoSborTypeId::Array,
            Value::Tuple(_) => ScryptoSborTypeId::Tuple,

            // ==============
            // Aliases
            // ==============
            Value::Some(_) => ScryptoSborTypeId::Enum,
            Value::None => ScryptoSborTypeId::Enum,
            Value::Ok(_) => ScryptoSborTypeId::Enum,
            Value::Err(_) => ScryptoSborTypeId::Enum,
            Value::Bytes(_) => ScryptoSborTypeId::Array,
            Value::NonFungibleAddress(_, _) => ScryptoSborTypeId::Tuple,

            // ==============
            // Custom Types
            // ==============

            // Global address types
            Value::PackageAddress(_) => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::PackageAddress)
            }
            Value::ComponentAddress(_) => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::ComponentAddress)
            }
            Value::ResourceAddress(_) => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::ResourceAddress)
            }
            Value::SystemAddress(_) => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::SystemAddress)
            }

            // RE interpreted
            Value::Own(_) => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Own),
            Value::Blob(_) => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Blob),

            // TX interpreted
            Value::Bucket(_) => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Bucket),
            Value::Proof(_) => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Proof),
            Value::Expression(_) => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Expression),

            // Uninterpreted,
            Value::Hash(_) => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Hash),
            Value::EcdsaSecp256k1PublicKey(_) => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::EcdsaSecp256k1PublicKey)
            }
            Value::EcdsaSecp256k1Signature(_) => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::EcdsaSecp256k1Signature)
            }
            Value::EddsaEd25519PublicKey(_) => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::EddsaEd25519PublicKey)
            }
            Value::EddsaEd25519Signature(_) => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::EddsaEd25519Signature)
            }
            Value::Decimal(_) => ScryptoSborTypeId::Custom(ScryptoCustomTypeId::Decimal),
            Value::PreciseDecimal(_) => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::PreciseDecimal)
            }
            Value::NonFungibleId(_) => {
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::NonFungibleId)
            }
        }
    }
}
