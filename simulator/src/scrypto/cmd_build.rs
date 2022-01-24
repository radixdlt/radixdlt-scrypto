use std::path::PathBuf;

use clap::{crate_version, App, Arg, ArgMatches};

use crate::scrypto::*;
use crate::utils::*;

const ARG_PATH: &str = "PATH";
const ARG_TRACE: &str = "TRACE";

/// Constructs a `build` subcommand.
pub fn make_build<'a>() -> App<'a> {
    App::new(CMD_BUILD)
        .about("Builds a package")
        .version(crate_version!())
        .arg(
            Arg::new(ARG_PATH)
                .long("path")
                .takes_value(true)
                .help("Specifies the package dir.")
                .required(false),
        )
        .arg(Arg::new(ARG_TRACE).long("trace").help("Turn on tracing."))
}

/// Handles a `build` request.
pub fn handle_build(matches: &ArgMatches) -> Result<(), Error> {
    let pkg_path = matches
        .value_of(ARG_PATH)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    let trace = matches.is_present(ARG_TRACE);

    build_package(pkg_path, trace)
        .map(|_| ())
        .map_err(Error::CargoError)
}
