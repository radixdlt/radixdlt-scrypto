use clap::{App, Arg, ArgMatches, SubCommand};

const ARG_ADDRESS: &'static str = "ADDRESS";

pub fn prepare_show<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(ARG_ADDRESS)
        .about("Show the content of an address.")
        .version("1.0")
        .arg(
            Arg::with_name("ADDRESS")
                .help("Specify the address.")
                .required(true),
        )
}

pub fn handle_show<'a>(_matches: &ArgMatches<'a>) {
    todo!()
}
