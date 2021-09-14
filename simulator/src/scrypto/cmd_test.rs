use clap::{crate_version, App, Arg, ArgMatches, SubCommand};

use crate::scrypto::*;
use crate::utils::*;

const ARG_ARGS: &str = "ARGS";

/// Constructs a `test` subcommand.
pub fn make_test_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_TEST)
        .about("Tests a package")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_ARGS)
                .help("Specify additional arguments to cargo.")
                .multiple(true),
        )
}

/// Handles a `test` request.
pub fn handle_test<'a>(matches: &ArgMatches<'a>) -> Result<(), Error> {
    let mut args = Vec::new();
    if let Some(x) = matches.values_of(ARG_ARGS) {
        x.for_each(|a| args.push(a));
    }

    test_package(std::env::current_dir().unwrap(), args)
        .map(|_| ())
        .map_err(Error::CargoError)
}
