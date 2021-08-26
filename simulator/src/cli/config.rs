use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use scrypto::rust::collections::*;
use std::fs;

use crate::cli::*;
use crate::ledger::*;

pub const CONFIG_DEFAULT_ACCOUNT: &'static str = "default.account";

pub fn get_configs() -> HashMap<String, String> {
    let path = get_config_json();
    if path.exists() {
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    } else {
        HashMap::new()
    }
}

pub fn set_configs(config: HashMap<String, String>) {
    let path = get_config_json();
    fs::write(path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
}

pub fn get_config(key: &str) -> Option<String> {
    get_configs().get(key).map(ToOwned::to_owned)
}

pub fn set_config(key: &str, value: &str) {
    let mut configs = get_configs();
    configs.insert(key.to_owned(), value.to_owned());
    set_configs(configs);
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
    let name = matches.value_of(ARG_NAME).unwrap();
    let value = matches.value_of(ARG_VALUE).unwrap();

    set_config(name, value);

    println!("{}", serde_json::to_string_pretty(&get_configs()).unwrap());
}
