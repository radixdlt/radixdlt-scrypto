use scrypto::engine::types::*;
use scrypto::values::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionValidationError {
    ParseScryptoValueError(ParseScryptoValueError),
    IdValidatorError(IdValidatorError),
    VaultNotAllowed(VaultId),
    LazyMapNotAllowed(LazyMapId),
    InvalidSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdAllocatorError {
    OutOfID,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdValidatorError {
    IdAllocatorError(IdAllocatorError),
    BucketNotFound(BucketId),
    ProofNotFound(ProofId),
    BucketLocked(BucketId),
}
