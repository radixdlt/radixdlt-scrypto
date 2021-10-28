use scrypto::types::*;

pub enum Auth {
    PackageAuth(Address),
    BadgeAuth(Address),
    NoAuth,
}

impl Auth {
    pub fn contains(&self, required: Address) -> bool {
        match self {
            Auth::PackageAuth(a) => *a == required,
            Auth::BadgeAuth(a) => *a == required,
            Auth::NoAuth => true,
        }
    }
}
