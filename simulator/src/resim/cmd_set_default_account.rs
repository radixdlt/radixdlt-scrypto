use clap::{crate_version, App, Arg, ArgMatches};

use crate::resim::*;

const ARG_ADDRESS: &str = "ADDRESS";
const ARG_PUBLIC_KEY: &str = "PUBLIC_KEY";

/// Constructs a `set-default-account` subcommand.
pub fn make_set_default_account<'a>() -> App<'a> {
    App::new(CMD_SET_DEFAULT_ACCOUNT)
        .about("Sets the default account")
        .version(crate_version!())
        .arg(
            Arg::new(ARG_ADDRESS)
                .help("Specify the account address.")
                .required(true),
        )
        .arg(
            Arg::new(ARG_PUBLIC_KEY)
                .help("Specify the account public key.")
                .required(true),
        )
}

/// Handles a `set-default-account` request.
pub fn handle_set_default_account(matches: &ArgMatches) -> Result<(), Error> {
    let address = match_address(matches, ARG_ADDRESS)?;
    let public_key = match_address(matches, ARG_PUBLIC_KEY)?;
    if !public_key.is_public_key() {
        return Err(Error::InvalidSignerPublicKey);
    }

    let mut configs = get_configs()?;
    configs.default_account = Some((address, public_key));
    set_configs(configs)?;

    println!("Default account set!");
    Ok(())
}
