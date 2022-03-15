use sbor::*;

use crate::resource::*;
use crate::rust::collections::BTreeSet;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::vec::Vec;

/// Used for determining the resource within a context.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ResourceDeterminer {
    /// Some specific amount
    Some(ResourceAmount, ResourceDefId),

    /// All of the specified resource within the context.
    All(ResourceDefId),
}

/// Represents an error when parsing `Resource` from string.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParseResourceDeterminerError {
    MissingResourceDefId,
    InvalidAmount,
    InvalidNonFungibleId,
    InvalidResourceDefId,
}

impl fmt::Display for ParseResourceDeterminerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseResourceDeterminerError {}

// Currently used by resim only.
// TODO: extend to support manifest use case.
impl FromStr for ResourceDeterminer {
    type Err = ParseResourceDeterminerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<&str> = s.trim().split(',').collect();

        if tokens.len() >= 2 {
            let resource_def_id = tokens
                .last()
                .unwrap()
                .parse::<ResourceDefId>()
                .map_err(|_| ParseResourceDeterminerError::InvalidResourceDefId)?;
            if tokens[0].starts_with('#') {
                let mut ids = BTreeSet::<NonFungibleId>::new();
                for key in &tokens[..tokens.len() - 1] {
                    if key.starts_with('#') {
                        ids.insert(
                            key[1..]
                                .parse()
                                .map_err(|_| ParseResourceDeterminerError::InvalidNonFungibleId)?,
                        );
                    } else {
                        return Err(ParseResourceDeterminerError::InvalidNonFungibleId);
                    }
                }
                Ok(ResourceDeterminer::Some(
                    ResourceAmount::NonFungible { ids },
                    resource_def_id,
                ))
            } else {
                if tokens.len() == 2 {
                    Ok(ResourceDeterminer::Some(
                        ResourceAmount::Fungible {
                            amount: tokens[0]
                                .parse()
                                .map_err(|_| ParseResourceDeterminerError::InvalidAmount)?,
                        },
                        resource_def_id,
                    ))
                } else {
                    Err(ParseResourceDeterminerError::InvalidAmount)
                }
            }
        } else {
            Err(ParseResourceDeterminerError::MissingResourceDefId)
        }
    }
}

impl ResourceDeterminer {
    pub fn amount(&self) -> Option<ResourceAmount> {
        match self {
            ResourceDeterminer::Some(amount, ..) => Some(amount.clone()),
            ResourceDeterminer::All(..) => None,
        }
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        match self {
            ResourceDeterminer::Some(_, resource_def_id)
            | ResourceDeterminer::All(resource_def_id) => *resource_def_id,
        }
    }
}
