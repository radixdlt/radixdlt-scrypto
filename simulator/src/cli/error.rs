pub use std::io;

pub use radix_engine::execution::*;
pub use sbor::*;

use crate::transaction::*;

#[derive(Debug)]
pub enum Error {
    NoDefaultAccount,

    NoHomeFolder,

    MissingSubCommand,

    MissingArgument(String),

    IOError(io::Error),

    JSONError(serde_json::Error),

    BuildError(BuildPackageError),

    ConstructionErr(TxnConstructionError),

    ExecutionError(RuntimeError),

    DataError(DecodeError),
}

#[derive(Debug)]
pub enum BuildPackageError {
    NotCargoPackage,

    FailedToParseCargoToml(cargo_toml::Error),

    MissingPackageInCargoToml,

    FailedToRunCargo(io::Error),

    FailedToWaitCargo(io::Error),
}
