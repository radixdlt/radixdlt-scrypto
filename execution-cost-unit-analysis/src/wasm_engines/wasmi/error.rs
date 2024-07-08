#![allow(clippy::enum_variant_names)]

use radix_common::prelude::*;
use radix_wasmi::errors::InstantiationError;
use radix_wasmi::*;

#[derive(Debug)]
#[allow(dead_code)]
pub enum WasmiInstantiationError {
    ValidationError(Error),
    PreInstantiationError(Error),
    InstantiationError(InstantiationError),
}
