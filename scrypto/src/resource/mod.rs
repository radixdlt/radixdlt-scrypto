mod badges;
mod badges_ref;
mod bucket;
mod bucket_ref;
mod tokens;
mod tokens_ref;

pub const SCRYPTO_TYPE_TOKENS: u8 = 0x90;
pub const SCRYPTO_TYPE_TOKENS_REF: u8 = 0x91;
pub const SCRYPTO_TYPE_BADGES: u8 = 0x92;
pub const SCRYPTO_TYPE_BADGES_REF: u8 = 0x93;

pub use badges::Badges;
pub use badges_ref::BadgesRef;
pub use bucket::Bucket;
pub use bucket_ref::BucketRef;
pub use tokens::Tokens;
pub use tokens_ref::TokensRef;
