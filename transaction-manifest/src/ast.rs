#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    DeclareTempBucket {
        name: Value,
    },

    DeclareTempBucketRef {
        name: Value,
    },

    TakeFromContext {
        amount: Value,
        resource_address: Value,
        to: Value,
    },

    BorrowFromContext {
        amount: Value,
        resource_address: Value,
        to: Value,
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

    DropAllBucketRefs,

    DepositAllBuckets {
        account: Value,
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
