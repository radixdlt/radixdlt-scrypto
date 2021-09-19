use std::path::PathBuf;

use clap::{crate_version, App, Arg, ArgMatches, SubCommand};

use crate::scrypto::*;
use crate::utils::*;

const ARG_PATH: &str = "PATH";

/// Constructs a `build` subcommand.
pub fn make_build<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_BUILD)
        .about("Builds a package")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_PATH)
                .long("path")
                .takes_value(true)
                .help("Specifies the package dir.")
                .required(false),
        )
}

/// Handles a `build` request.
pub fn handle_build(matches: &ArgMatches) -> Result<(), Error> {
    let pkg_path = matches
        .value_of(ARG_PATH)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    build_package(pkg_path)
        .map(|_| ())
        .map_err(Error::CargoError)
}
