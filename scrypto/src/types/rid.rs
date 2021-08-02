extern crate alloc;
use alloc::string::String;

use sbor::{Decode, Encode};

/// Resource type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum ResourceKind {
    /// A token bucket.
    Tokens,

    /// A reference to a token bucket.
    TokensRef,

    /// A badge bucket.
    Badges,

    /// A reference to a badge bucket.
    BadgesRef,
}

/// Represents a resource maintained by runtime.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct RID {
    kind: ResourceKind,
    id: String,
}

impl RID {
    /// Creates a new RID.
    pub fn new(kind: ResourceKind, id: String) -> Self {
        Self { kind, id }
    }

    /// Gets the next RID.
    pub fn next(&self, f: fn(&String) -> String) -> Self {
        Self::new(self.kind, f(&self.id))
    }

    /// Gets the borrowed form of this RID.
    pub fn to_borrowed(&self) -> Self {
        assert!(
            self.kind() == ResourceKind::Tokens || self.kind() == ResourceKind::Badges,
            "Can't borrow from non-reference type"
        );

        Self {
            kind: if self.kind() == ResourceKind::Tokens {
                ResourceKind::TokensRef
            } else {
                ResourceKind::BadgesRef
            },
            id: self.id.clone(),
        }
    }

    /// Gets the owned form of this RID.
    pub fn to_owned(&self) -> Self {
        assert!(
            self.kind() == ResourceKind::TokensRef || self.kind() == ResourceKind::BadgesRef,
            "Already an owned type"
        );

        Self {
            kind: if self.kind() == ResourceKind::TokensRef {
                ResourceKind::Tokens
            } else {
                ResourceKind::Badges
            },
            id: self.id.clone(),
        }
    }

    /// Gets the resource type.
    pub fn kind(&self) -> ResourceKind {
        self.kind
    }

    /// Gets the resource bucket id.
    pub fn id(&self) -> &String {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::ToString;

    use crate::types::*;

    #[test]
    fn test_basics() {
        let rid = RID::new(ResourceKind::Tokens, "awesome-bucket-id".to_string());
        let rid2 = rid.next(|_| "new-bucket-id".to_string());
        let rid3 = rid2.to_borrowed();
        assert_eq!(rid3.kind(), ResourceKind::TokensRef);
        assert_eq!(rid3.id(), "new-bucket-id");
    }
}
