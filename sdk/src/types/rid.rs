extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

use serde::{Deserialize, Serialize};

pub const RID_TOKENS: u8 = 0;
pub const RID_BADGES: u8 = 1;
pub const RID_BADGES_REF: u8 = 2;

/// Remote object ID
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String")]
#[serde(into = "String")]
pub struct RID {
    kind: u8,
    id: u64,
}

impl From<&str> for RID {
    fn from(s: &str) -> Self {
        let tokens: Vec<&str> = s.split("-").collect();
        Self {
            kind: tokens[1].parse::<u8>().unwrap(),
            id: tokens[2].parse::<u64>().unwrap(),
        }
    }
}

impl From<String> for RID {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl Into<String> for RID {
    fn into(self) -> String {
        format!("R-{}-{}", self.kind, self.id)
    }
}

impl ToString for RID {
    fn to_string(&self) -> String {
        self.clone().into()
    }
}

impl fmt::Debug for RID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl RID {
    pub fn new(kind: u8, id: u64) -> Self {
        Self { kind, id }
    }

    pub fn next(&self) -> Self {
        Self::new(self.kind, self.id + 1)
    }

    pub fn to_borrowed(&self) -> Self {
        assert_eq!(self.kind, RID_BADGES);
        Self {
            kind: RID_BADGES_REF,
            id: self.id,
        }
    }

    pub fn to_owned(&self) -> Self {
        assert_eq!(self.kind, RID_BADGES_REF);
        Self {
            kind: RID_BADGES,
            id: self.id,
        }
    }

    pub fn kind(&self) -> u8 {
        self.kind
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}
