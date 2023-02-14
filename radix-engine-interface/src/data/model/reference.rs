use crate::{api::types::*, *};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum Reference {
    Package(PackageAddress),
    Component(ComponentAddress),
    Resource(ResourceAddress),
}
