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
        function: Value,
        args: Vec<Value>,
    },

    CallMethod {
        component_address: Value,
        method: Value,
        args: Vec<Value>,
    },

    PublishPackage {
        package_blob: Value,
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
    Struct,
    Enum,
    Option,
    Result,

    /* [T; N] and (A, B, C) */
    Array,
    Tuple,

    /* Collections */
    List,
    Set,
    Map,

    /* Custom types */
    Decimal,
    PreciseDecimal,
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
    Hash,
    Bucket,
    Proof,
    NonFungibleId,
    NonFungibleAddress,
    Expression,
    Blob,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
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

    Struct(Vec<Value>),
    Enum(String, Vec<Value>),
    Option(Box<Option<Value>>),
    Result(Box<Result<Value, Value>>),

    Array(Type, Vec<Value>),
    Tuple(Vec<Value>),

    List(Type, Vec<Value>),
    Set(Type, Vec<Value>),
    Map(Type, Type, Vec<Value>),

    Decimal(Box<Value>),
    PreciseDecimal(Box<Value>),
    PackageAddress(Box<Value>),
    ComponentAddress(Box<Value>),
    ResourceAddress(Box<Value>),
    Hash(Box<Value>),
    Bucket(Box<Value>),
    Proof(Box<Value>),
    NonFungibleId(Box<Value>),
    NonFungibleAddress(Box<Value>),
    Expression(Box<Value>),
    Blob(Box<Value>),
}

impl Value {
    pub const fn kind(&self) -> Type {
        match self {
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
            Value::Struct(_) => Type::Struct,
            Value::Enum(_, _) => Type::Enum,
            Value::Option(_) => Type::Option,
            Value::Array(_, _) => Type::Array,
            Value::Tuple(_) => Type::Tuple,
            Value::Result(_) => Type::Result,
            Value::List(_, _) => Type::List,
            Value::Set(_, _) => Type::Set,
            Value::Map(_, _, _) => Type::Map,
            Value::Decimal(_) => Type::Decimal,
            Value::PreciseDecimal(_) => Type::PreciseDecimal,
            Value::PackageAddress(_) => Type::PackageAddress,
            Value::ComponentAddress(_) => Type::ComponentAddress,
            Value::ResourceAddress(_) => Type::ResourceAddress,
            Value::Hash(_) => Type::Hash,
            Value::Bucket(_) => Type::Bucket,
            Value::Proof(_) => Type::Proof,
            Value::NonFungibleId(_) => Type::NonFungibleId,
            Value::NonFungibleAddress(_) => Type::NonFungibleAddress,
            Value::Expression(_) => Type::Expression,
            Value::Blob(_) => Type::Blob,
        }
    }
}
