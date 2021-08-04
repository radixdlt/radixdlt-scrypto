use sbor::{Decode, Encode};

use crate::types::Hash;

/// Resource bucket type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum BucketKind {
    /// A token bucket.
    Tokens,

    /// A reference to a token bucket.
    TokensRef,

    /// A badge bucket.
    Badges,

    /// A reference to a badge bucket.
    BadgesRef,
}

/// Resource bucket id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum BucketId {
    Transient(u32),

    Persisted(Hash, u32),
}

/// Represents a resource maintained by runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub struct RID {
    kind: BucketKind,
    id: BucketId,
}

impl RID {
    /// Creates a new RID.
    pub fn new(kind: BucketKind, id: BucketId) -> Self {
        Self { kind, id }
    }

    /// Gets the borrowed form of this RID.
    pub fn to_borrowed(&self) -> Self {
        assert!(
            self.kind() == BucketKind::Tokens || self.kind() == BucketKind::Badges,
            "Can't borrow from non-reference type"
        );

        Self {
            kind: if self.kind() == BucketKind::Tokens {
                BucketKind::TokensRef
            } else {
                BucketKind::BadgesRef
            },
            id: self.id,
        }
    }

    /// Gets the owned form of this RID.
    pub fn to_owned(&self) -> Self {
        assert!(
            self.kind() == BucketKind::TokensRef || self.kind() == BucketKind::BadgesRef,
            "Already an owned type"
        );

        Self {
            kind: if self.kind() == BucketKind::TokensRef {
                BucketKind::Tokens
            } else {
                BucketKind::Badges
            },
            id: self.id,
        }
    }

    /// Gets the resource type.
    pub fn kind(&self) -> BucketKind {
        self.kind
    }

    /// Gets the resource bucket id.
    pub fn id(&self) -> BucketId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basics() {
        let rid = RID::new(BucketKind::Tokens, BucketId::Transient(5));
        let rid2 = rid.to_borrowed();
        assert_eq!(rid2.kind(), BucketKind::TokensRef);
        assert_eq!(rid2.id(), BucketId::Transient(5));
    }
}
