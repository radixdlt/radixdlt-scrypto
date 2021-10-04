use std::io;

use crate::utils::*;

/// Represents a scrypto error.
#[derive(Debug)]
pub enum Error {
    MissingArgument(String),

    MissingSubCommand,

    IOError(io::Error),

    CargoError(CargoExecutionError),

    PackageAlreadyExists,
}
