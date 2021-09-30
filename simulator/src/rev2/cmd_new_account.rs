use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::transaction::*;

use crate::ledger::*;
use crate::rev2::*;

const ARG_TRACE: &str = "TRACE";

/// Constructs a `new-account` subcommand.
pub fn make_new_account<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_NEW_ACCOUNT)
        .about("Creates an account")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_TRACE)
                .long("trace")
                .help("Turns on tracing."),
        )
}

/// Handles a `new-account` request.
pub fn handle_new_account(matches: &ArgMatches) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);

    let mut ledger = FileBasedLedger::new(get_data_dir()?);

    let mut executor = TransactionExecutor::new(&mut ledger, 0, 0);

    let account = executor.new_account( trace);

    println!("New account: {}", account);

    // set as default config if not set
    if get_config(CONF_DEFAULT_ACCOUNT)?.is_none() {
        set_config(CONF_DEFAULT_ACCOUNT, &account.to_string())?;
        println!("No default account configured. This will be used as the default account.")
    }

    Ok(())
}
