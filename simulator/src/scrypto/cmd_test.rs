use std::path::PathBuf;

use clap::{crate_version, App, Arg, ArgMatches, SubCommand};

use crate::scrypto::*;
use crate::utils::*;

const ARG_ARGS: &str = "ARGS";
const ARG_PATH: &str = "PATH";

/// Constructs a `test` subcommand.
pub fn make_test<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_TEST)
        .about("Tests a package")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_ARGS)
                .help("Specify additional arguments to cargo.")
                .multiple(true),
        )
        .arg(
            Arg::with_name(ARG_PATH)
                .long("path")
                .takes_value(true)
                .help("Specifies the package dir.")
                .required(false),
        )
}

/// Handles a `test` request.
pub fn handle_test(matches: &ArgMatches) -> Result<(), Error> {
    let mut args = Vec::new();
    if let Some(x) = matches.values_of(ARG_ARGS) {
        x.for_each(|a| args.push(a));
    }

    let pkg_path = matches
        .value_of(ARG_PATH)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    test_package(pkg_path, args)
        .map(|_| ())
        .map_err(Error::CargoError)
}
