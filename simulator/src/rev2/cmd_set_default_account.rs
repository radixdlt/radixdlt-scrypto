use clap::{crate_version, App, Arg, ArgMatches, SubCommand};

use crate::rev2::*;

const ARG_ADDRESS: &str = "ADDRESS";

/// Constructs a `config` subcommand.
pub fn make_set_default_account<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_SET_DEFAULT_ACCOUNT)
        .about("Sets the default account")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_ADDRESS)
                .help("Specify the address.")
                .required(true),
        )
}

/// Handles a `config` request.
pub fn handle_set_default_account(matches: &ArgMatches) -> Result<(), Error> {
    let address: Address = matches
        .value_of(ARG_ADDRESS)
        .ok_or_else(|| Error::MissingArgument(ARG_ADDRESS.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)?;

    set_config(CONF_DEFAULT_ACCOUNT, &address.to_string())?;

    println!("Done!");
    Ok(())
}
