use sbor::*;

use crate::resource::*;
use crate::rust::collections::BTreeSet;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::vec::Vec;

/// Used for determining the resource within a context.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ResourceSpecifier {
    /// Some specific amount
    Some(Amount, ResourceDefId),

    /// All of the specified resource within the context.
    All(ResourceDefId),
}

/// Represents an error when parsing `Resource` from string.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParseResourceSpecifierError {
    MissingResourceDefId,
    InvalidAmount,
    InvalidNonFungibleId,
    InvalidResourceDefId,
}

impl fmt::Display for ParseResourceSpecifierError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseResourceSpecifierError {}

// Currently used by resim only.
// TODO: extend to support manifest use case.
impl FromStr for ResourceSpecifier {
    type Err = ParseResourceSpecifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<&str> = s.trim().split(',').collect();

        if tokens.len() >= 2 {
            let resource_def_id = tokens
                .last()
                .unwrap()
                .parse::<ResourceDefId>()
                .map_err(|_| ParseResourceSpecifierError::InvalidResourceDefId)?;
            if tokens[0].starts_with('#') {
                let mut ids = BTreeSet::<NonFungibleId>::new();
                for key in &tokens[..tokens.len() - 1] {
                    if key.starts_with('#') {
                        ids.insert(
                            key[1..]
                                .parse()
                                .map_err(|_| ParseResourceSpecifierError::InvalidNonFungibleId)?,
                        );
                    } else {
                        return Err(ParseResourceSpecifierError::InvalidNonFungibleId);
                    }
                }
                Ok(ResourceSpecifier::Some(
                    Amount::NonFungible { ids },
                    resource_def_id,
                ))
            } else {
                if tokens.len() == 2 {
                    Ok(ResourceSpecifier::Some(
                        Amount::Fungible {
                            amount: tokens[0]
                                .parse()
                                .map_err(|_| ParseResourceSpecifierError::InvalidAmount)?,
                        },
                        resource_def_id,
                    ))
                } else {
                    Err(ParseResourceSpecifierError::InvalidAmount)
                }
            }
        } else {
            Err(ParseResourceSpecifierError::MissingResourceDefId)
        }
    }
}

impl ResourceSpecifier {
    pub fn amount(&self) -> Option<Amount> {
        match self {
            ResourceSpecifier::Some(amount, ..) => Some(amount.clone()),
            ResourceSpecifier::All(..) => None,
        }
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        match self {
            ResourceSpecifier::Some(_, resource_def_id)
            | ResourceSpecifier::All(resource_def_id) => *resource_def_id,
        }
    }
}
