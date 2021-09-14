use std::fs::remove_dir_all;

use clap::{crate_version, App, ArgMatches, SubCommand};

use crate::rev2::*;
/// Constructs a `reset` subcommand.
pub fn make_reset_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_RESET)
        .about("Resets the data directory")
        .version(crate_version!())
}

/// Handles a `reset` request.
pub fn handle_reset<'a>(_matches: &ArgMatches<'a>) -> Result<(), Error> {
    let dir = get_data_dir()?;
    remove_dir_all(dir).map_err(|e| Error::IOError(e))?;
    println!("Data directory deleted.");
    Ok(())
}
