use std::fs::remove_dir_all;

use clap::{crate_version, App, ArgMatches, SubCommand};

use crate::*;

/// Constructs a `reset` subcommand.
pub fn make_reset_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_RESET)
        .about("Resets data directory.")
        .version(crate_version!())
}

/// Handles a `reset` request.
pub fn handle_reset<'a>(_matches: &ArgMatches<'a>) {
    let file = get_data_dir();
    if file.exists() {
        remove_dir_all(file).unwrap();
    }
    println!("Data directory emptied.");
}
