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

    CreateValidator {
        key: Value,
    },

    RegisterValidator {
        validator: Value,
    },

    UnregisterValidator {
        validator: Value,
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
    // Custom Types
    // ==============

    // Globals
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
    SystemAddress,

    // RE Nodes
    Component,
    KeyValueStore,
    Bucket,
    Proof,
    Vault,

    // Other interpreted types
    Expression,
    Blob,
    NonFungibleAddress,

    // Uninterpreted,
    Hash,
    EcdsaSecp256k1PublicKey,
    EcdsaSecp256k1Signature,
    EddsaEd25519PublicKey,
    EddsaEd25519Signature,
    Decimal,
    PreciseDecimal,
    NonFungibleId,
    Bytes,
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

    // ==============
    // Custom Types
    // ==============

    // Globals
    PackageAddress(Box<Value>),
    ComponentAddress(Box<Value>),
    ResourceAddress(Box<Value>),
    SystemAddress(Box<Value>),

    // RE Nodes
    Component(Box<Value>),
    KeyValueStore(Box<Value>),
    Bucket(Box<Value>),
    Proof(Box<Value>),
    Vault(Box<Value>),

    // Other interpreted types
    Expression(Box<Value>),
    Blob(Box<Value>),
    NonFungibleAddress(Box<Value>, Box<Value>),

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
    pub const fn kind(&self) -> Type {
        match self {
            // ==============
            // Basic Types
            // ==============
            Value::Unit => Type::Unit,
            Value::Bool(_) => Type::Bool,
            Value::I8(_) => Type::I8,
            Value::I16(_) => Type::I16,
            Value::I32(_) => Type::I32,
            Value::I64(_) => Type::I64,
            Value::I128(_) => Type::I128,
            Value::U8(_) => Type::U8,
            Value::U16(_) => Type::U16,
            Value::U32(_) => Type::U32,
            Value::U64(_) => Type::U64,
            Value::U128(_) => Type::U128,
            Value::String(_) => Type::String,
            Value::Enum(_, _) => Type::Enum,
            Value::Array(_, _) => Type::Array,
            Value::Tuple(_) => Type::Tuple,

            // ==============
            // Aliases
            // ==============
            Value::Some(_) => Type::Enum,
            Value::None => Type::Enum,
            Value::Ok(_) => Type::Enum,
            Value::Err(_) => Type::Enum,
            Value::Bytes(_) => Type::Bytes,

            // ==============
            // Custom Types
            // ==============

            // Global address types
            Value::PackageAddress(_) => Type::PackageAddress,
            Value::ComponentAddress(_) => Type::ComponentAddress,
            Value::ResourceAddress(_) => Type::ResourceAddress,
            Value::SystemAddress(_) => Type::SystemAddress,

            // RE Nodes
            Value::Component(_) => Type::Component,
            Value::KeyValueStore(_) => Type::KeyValueStore,
            Value::Bucket(_) => Type::Bucket,
            Value::Proof(_) => Type::Proof,
            Value::Vault(_) => Type::Vault,

            // Other interpreted types
            Value::Expression(_) => Type::Expression,
            Value::Blob(_) => Type::Blob,
            Value::NonFungibleAddress(_, _) => Type::NonFungibleAddress,

            // Uninterpreted,
            Value::Hash(_) => Type::Hash,
            Value::EcdsaSecp256k1PublicKey(_) => Type::EcdsaSecp256k1PublicKey,
            Value::EcdsaSecp256k1Signature(_) => Type::EcdsaSecp256k1Signature,
            Value::EddsaEd25519PublicKey(_) => Type::EddsaEd25519PublicKey,
            Value::EddsaEd25519Signature(_) => Type::EddsaEd25519Signature,
            Value::Decimal(_) => Type::Decimal,
            Value::PreciseDecimal(_) => Type::PreciseDecimal,
            Value::NonFungibleId(_) => Type::NonFungibleId,
        }
    }
}
