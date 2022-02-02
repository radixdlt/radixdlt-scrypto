use scrypto::rust::collections::HashMap;
use scrypto::rust::collections::HashSet;
use scrypto::types::*;

/// Represents the authenticated actor.
#[derive(Debug, Clone)]
pub enum Actor {
    SuperUser,

    Package(Address),

    PackageWithBadges(Address, HashSet<Address>),
}

impl Actor {
    // Checks if this actor is the authority.
    pub fn check(&self, authority: Address) -> bool {
        match self {
            Self::SuperUser => true,
            Self::Package(pkg) => *pkg == authority,
            Self::PackageWithBadges(pkg, badges) => {
                *pkg == authority || badges.contains(&authority)
            }
        }
    }

    // Checks if this actor is a member of the authorities and has the given permission.
    pub fn check_permission(&self, authorities: &HashMap<Address, u64>, permission: u64) -> bool {
        match self {
            Self::SuperUser => true,
            Self::Package(pkg) => {
                if let Some(v) = authorities.get(pkg) {
                    v & permission == permission
                } else {
                    false
                }
            }
            Self::PackageWithBadges(pkg, badges) => {
                if let Some(v) = authorities.get(pkg) {
                    return v & permission == permission;
                }

                for badge in badges {
                    if let Some(v) = authorities.get(badge) {
                        return v & permission == permission;
                    }
                }
                false
            }
        }
    }
}
