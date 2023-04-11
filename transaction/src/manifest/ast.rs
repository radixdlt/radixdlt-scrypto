use radix_engine_interface::data::manifest::{ManifestCustomValueKind, ManifestValueKind};

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

    ClearSignatureProofs,

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
        schema: Value,
        royalty_config: Value,
        metadata: Value,
    },
    PublishPackageAdvanced {
        code: Value,
        schema: Value,
        royalty_config: Value,
        metadata: Value,
        access_rules: Value,
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

    RemoveMetadata {
        entity_address: Value,
        key: Value,
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
        key: Value,
        rule: Value,
    },

    MintFungible {
        resource_address: Value,
        amount: Value,
    },

    MintNonFungible {
        resource_address: Value,
        args: Value,
    },

    MintUuidNonFungible {
        resource_address: Value,
        args: Value,
    },

    CreateFungibleResource {
        divisibility: Value,
        metadata: Value,
        access_rules: Value,
    },

    CreateFungibleResourceWithInitialSupply {
        divisibility: Value,
        metadata: Value,
        access_rules: Value,
        initial_supply: Value,
    },

    CreateNonFungibleResource {
        id_type: Value,
        schema: Value,
        metadata: Value,
        access_rules: Value,
    },

    CreateNonFungibleResourceWithInitialSupply {
        id_type: Value,
        schema: Value,
        metadata: Value,
        access_rules: Value,
        initial_supply: Value,
    },

    CreateValidator {
        key: Value,
    },
    CreateAccessController {
        controlled_asset: Value,
        rule_set: Value,
        timed_recovery_delay_in_minutes: Value,
    },

    CreateIdentity {},
    CreateIdentityAdvanced {
        config: Value,
    },

    CreateAccount {},
    CreateAccountAdvanced {
        config: Value,
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
