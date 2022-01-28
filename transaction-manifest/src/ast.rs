#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    CreateTempBucket {
        amount: Value,
        resource_address: Value,
        new_bucket: Value,
    },

    CreateTempBucketRef {
        bucket: Value,
        new_bucket_ref: Value,
    },

    CloneTempBucketRef {
        bucket_ref: Value,
        new_bucket_ref: Value,
    },

    DropTempBucketRef {
        bucket_ref: Value,
    },

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

    CallMethodWithAllResources {
        component_address: Value,
        method: Value,
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
    Struct,
    Enum,
    Option,
    Box,
    Array,
    Tuple,
    Result,

    /* Containers */
    Vec,
    TreeSet,
    TreeMap,
    HashSet,
    HashMap,

    /* Custom types */
    Decimal,
    BigDecimal,
    Address,
    Hash,
    Bucket,
    BucketRef,
    LazyMap,
    Vault,
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
    Struct(Fields),
    Enum(u8, Fields),
    Option(Box<Option<Value>>),
    Box(Box<Value>),
    Array(Type, Vec<Value>),
    Tuple(Vec<Value>),
    Result(Box<Result<Value, Value>>),

    Vec(Type, Vec<Value>),
    TreeSet(Type, Vec<Value>),
    TreeMap(Type, Type, Vec<Value>),
    HashSet(Type, Vec<Value>),
    HashMap(Type, Type, Vec<Value>),

    Decimal(Box<Value>),
    BigDecimal(Box<Value>),
    Address(Box<Value>),
    Hash(Box<Value>),
    Bucket(Box<Value>),
    BucketRef(Box<Value>),
    LazyMap(Box<Value>),
    Vault(Box<Value>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fields {
    Named(Vec<Value>),

    Unnamed(Vec<Value>),

    Unit,
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
            Value::Box(_) => Type::Box,
            Value::Array(_, _) => Type::Array,
            Value::Tuple(_) => Type::Tuple,
            Value::Result(_) => Type::Result,
            Value::Vec(_, _) => Type::Vec,
            Value::TreeSet(_, _) => Type::TreeSet,
            Value::TreeMap(_, _, _) => Type::TreeMap,
            Value::HashSet(_, _) => Type::HashSet,
            Value::HashMap(_, _, _) => Type::HashMap,
            Value::Decimal(_) => Type::Decimal,
            Value::BigDecimal(_) => Type::BigDecimal,
            Value::Address(_) => Type::Address,
            Value::Hash(_) => Type::Hash,
            Value::Bucket(_) => Type::Bucket,
            Value::BucketRef(_) => Type::BucketRef,
            Value::LazyMap(_) => Type::LazyMap,
            Value::Vault(_) => Type::Vault,
        }
    }
}
