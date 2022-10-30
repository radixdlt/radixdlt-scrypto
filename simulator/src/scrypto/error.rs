use std::io;

use crate::utils::*;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),

    BuildError(BuildError),

    TestError(TestError),

    FormatError(FormatError),

    PackageAlreadyExists,
}
