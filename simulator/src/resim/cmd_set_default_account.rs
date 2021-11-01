use clap::{crate_version, App, Arg, ArgMatches, SubCommand};

use crate::resim::*;

const ARG_ACCOUNT: &str = "ACCOUNT";

/// Constructs a `set-default-account` subcommand.
pub fn make_set_default_account<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_SET_DEFAULT_ACCOUNT)
        .about("Sets the default account")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_ACCOUNT)
                .help("Specify the account address.")
                .required(true),
        )
}

/// Handles a `set-default-account` request.
pub fn handle_set_default_account(matches: &ArgMatches) -> Result<(), Error> {
    let account = match_address(matches, ARG_ACCOUNT)?;

    let mut configs = get_configs()?;
    configs.default_account = Some(account);
    set_configs(configs)?;

    println!("Default account set!");
    Ok(())
}
