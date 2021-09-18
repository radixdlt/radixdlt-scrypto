use clap::{crate_version, App, ArgMatches, SubCommand};

use crate::scrypto::*;
use crate::utils::*;

/// Constructs a `build` subcommand.
pub fn make_build<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_BUILD)
        .about("Builds a package")
        .version(crate_version!())
}

/// Handles a `build` request.
pub fn handle_build(_matches: &ArgMatches) -> Result<(), Error> {
    build_package(std::env::current_dir().unwrap())
        .map(|_| ())
        .map_err(Error::CargoError)
}
