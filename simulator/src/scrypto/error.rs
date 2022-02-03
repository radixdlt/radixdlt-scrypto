use std::io;

use crate::utils::*;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),

    CargoError(CargoExecutionError),

    PackageAlreadyExists,
}
