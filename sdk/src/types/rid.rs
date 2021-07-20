extern crate alloc;
use alloc::string::String;

use serde::{Deserialize, Serialize};

/// Remote object type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectType {
    /// A token bucket
    Tokens,

    /// A reference to a token bucket
    TokensRef,

    /// A badge bucket
    Badges,

    /// A reference to a badge bucket
    BadgesRef,
}

/// Represents a remote object, maintained by runtime
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RID {
    kind: ObjectType,
    id: String,
}

impl RID {
    /// Creates a new RID
    pub fn new(kind: ObjectType, id: String) -> Self {
        Self { kind, id }
    }

    /// Gets the next RID
    pub fn next(&self, f: fn(&String) -> String) -> Self {
        Self::new(self.kind, f(&self.id))
    }

    /// Gets the borrowed form of this RID
    pub fn to_borrowed(&self) -> Self {
        assert!(
            self.kind() == ObjectType::Tokens || self.kind() == ObjectType::Badges,
            "Can't borrow from non-reference type"
        );

        Self {
            kind: if self.kind() == ObjectType::Tokens {
                ObjectType::TokensRef
            } else {
                ObjectType::BadgesRef
            },
            id: self.id.clone(),
        }
    }

    /// Gets the owned form of this RID
    pub fn to_owned(&self) -> Self {
        assert!(
            self.kind() == ObjectType::TokensRef || self.kind() == ObjectType::BadgesRef,
            "Already an owned type"
        );

        Self {
            kind: if self.kind() == ObjectType::TokensRef {
                ObjectType::Tokens
            } else {
                ObjectType::Badges
            },
            id: self.id.clone(),
        }
    }

    /// Gets the object type
    pub fn kind(&self) -> ObjectType {
        self.kind
    }

    /// Get the resource bucket id
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
        let rid = RID::new(ObjectType::Tokens, "awesome-bucket-id".to_string());
        let rid2 = rid.next(|_| "new-bucket-id".to_string());
        let rid3 = rid2.to_borrowed();
        assert_eq!(rid3.kind(), ObjectType::TokensRef);
        assert_eq!(rid3.id(), "new-bucket-id");
    }
}
