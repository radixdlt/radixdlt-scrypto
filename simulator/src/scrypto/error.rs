pub use std::io;

pub use sbor::*;
pub use scrypto::types::*;

use crate::utils::*;

#[derive(Debug)]
pub enum Error {
    MissingArgument(String),

    MissingSubCommand,

    IOError(io::Error),

    CargoError(CargoExecutionError),

    PackageAlreadyExists,
}
