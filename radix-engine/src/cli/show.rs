use clap::{App, Arg, ArgMatches, SubCommand};

pub fn prepare_show<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("account")
        .about("Show the content of an address.")
        .version("1.0")
        .arg(
            Arg::with_name("ADDRESS")
                .help("Specify the address.")
                .required(true),
        )
}

pub fn handle_show<'a>(matches: &ArgMatches<'a>) {
    todo!()
}
