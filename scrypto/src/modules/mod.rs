mod access_rules;
mod metadata;
mod royalty;
mod module;

use std::marker::PhantomData;
use std::ops::Deref;
pub use access_rules::Mutability::*;
pub use access_rules::*;
pub use metadata::*;
pub use royalty::*;
pub use module::*;
use scrypto::prelude::GlobalAddress;
use crate::prelude::Own;


