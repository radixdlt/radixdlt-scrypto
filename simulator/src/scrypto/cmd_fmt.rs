use std::path::PathBuf;

use clap::{crate_version, App, Arg, ArgMatches, SubCommand};

use crate::scrypto::*;
use crate::utils::*;

const ARG_PATH: &str = "PATH";

/// Constructs a `fmt` subcommand.
pub fn make_fmt<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_FMT)
        .about("Format a package")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_PATH)
                .long("path")
                .takes_value(true)
                .help("Specifies the package dir.")
                .required(false),
        )
}

/// Handles a `fmt` request.
pub fn handle_fmt(matches: &ArgMatches) -> Result<(), Error> {
    let pkg_path = matches
        .value_of(ARG_PATH)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    fmt_package(pkg_path).map(|_| ()).map_err(Error::CargoError)
}
