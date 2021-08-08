use std::fs::remove_dir_all;

use clap::{App, ArgMatches, SubCommand};

use crate::cli::get_root_dir;

pub fn prepare_clear<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("clear")
        .about("Clear ledger state.")
        .version("1.0")
}

pub fn handle_clear<'a>(_args: &ArgMatches<'a>) {
    let file = get_root_dir();
    if file.exists() {
        remove_dir_all(file).unwrap();
    }
    println!("Ledger state cleared.");
}
