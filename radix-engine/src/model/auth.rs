use scrypto::rust::collections::HashMap;
use scrypto::types::*;

pub enum Auth {
    Package(Address),
    ResourceHolder(Address),
    SuperUser,
}

impl Auth {
    // Checks if this authentication matches the given authority.
    pub fn check(&self, authority: Address) -> bool {
        match self {
            Auth::Package(a) | Auth::ResourceHolder(a) => *a == authority,
            Auth::SuperUser => true,
        }
    }

    // Checks if this authentication has been granted some permission.
    pub fn check_for(&self, authorities: &HashMap<Address, u16>, permission: u16) -> bool {
        match self {
            Auth::Package(a) | Auth::ResourceHolder(a) => {
                if let Some(v) = authorities.get(a) {
                    v & permission != 0
                } else {
                    false
                }
            }
            Auth::SuperUser => true,
        }
    }
}
