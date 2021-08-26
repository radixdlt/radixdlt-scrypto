use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::rust::collections::*;
use scrypto::types::*;
use std::fs;

use crate::cli::*;
use crate::ledger::*;

pub const CONFIG_DEFAULT_ACCOUNT: &'static str = "default.account";

pub fn get_default_account() -> Address {
    let path = get_config_json();
    let config: HashMap<String, String> =
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    config
        .get(CONFIG_DEFAULT_ACCOUNT)
        .expect("Default account not set")
        .as_str()
        .into()
}

const ARG_NAME: &'static str = "NAME";
const ARG_VALUE: &'static str = "VALUE";

/// Constructs a `config` subcommand.
pub fn make_config_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_CONFIG)
        .about("Config this simulator")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_NAME)
                .help("Specify the name, e.g. `default.account`")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_VALUE)
                .help("Specify the value.")
                .required(true),
        )
}

/// Handles a `config` request.
pub fn handle_config<'a>(matches: &ArgMatches<'a>) {
    let name = matches.value_of(ARG_NAME).unwrap().to_owned();
    let value = matches.value_of(ARG_VALUE).unwrap().to_owned();

    let path = get_config_json();
    let mut config = if path.exists() {
        serde_json::from_str(fs::read_to_string(&path).unwrap().as_str()).unwrap()
    } else {
        HashMap::<String, String>::new()
    };
    config.insert(name, value);
    fs::write(path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

    println!("{}", serde_json::to_string_pretty(&config).unwrap());
}
