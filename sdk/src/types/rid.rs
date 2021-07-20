extern crate alloc;
use alloc::string::String;

use serde::{Deserialize, Serialize};

/// Resource type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RID {
    kind: ResourceType,
    id: String,
}

impl RID {
    /// Creates a new RID.
    pub fn new(kind: ResourceType, id: String) -> Self {
        Self { kind, id }
    }

    /// Gets the next RID.
    pub fn next(&self, f: fn(&String) -> String) -> Self {
        Self::new(self.kind, f(&self.id))
    }

    /// Gets the borrowed form of this RID.
    pub fn to_borrowed(&self) -> Self {
        assert!(
            self.kind() == ResourceType::Tokens || self.kind() == ResourceType::Badges,
            "Can't borrow from non-reference type"
        );

        Self {
            kind: if self.kind() == ResourceType::Tokens {
                ResourceType::TokensRef
            } else {
                ResourceType::BadgesRef
            },
            id: self.id.clone(),
        }
    }

    /// Gets the owned form of this RID.
    pub fn to_owned(&self) -> Self {
        assert!(
            self.kind() == ResourceType::TokensRef || self.kind() == ResourceType::BadgesRef,
            "Already an owned type"
        );

        Self {
            kind: if self.kind() == ResourceType::TokensRef {
                ResourceType::Tokens
            } else {
                ResourceType::Badges
            },
            id: self.id.clone(),
        }
    }

    /// Gets the resource type.
    pub fn kind(&self) -> ResourceType {
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
        let rid = RID::new(ResourceType::Tokens, "awesome-bucket-id".to_string());
        let rid2 = rid.next(|_| "new-bucket-id".to_string());
        let rid3 = rid2.to_borrowed();
        assert_eq!(rid3.kind(), ResourceType::TokensRef);
        assert_eq!(rid3.id(), "new-bucket-id");
    }
}
