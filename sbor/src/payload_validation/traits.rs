use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

pub trait ValidatableCustomTypeExtension: CustomTypeExtension {
    type ValidationContext;
}
