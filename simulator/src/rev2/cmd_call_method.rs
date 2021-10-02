use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::transaction::*;

use crate::ledger::*;
use crate::rev2::*;

const ARG_COMPONENT: &str = "COMPONENT";
const ARG_METHOD: &str = "METHOD";
const ARG_ARGS: &str = "ARGS";

const ARG_TRACE: &str = "TRACE";

/// Constructs a `call-method` subcommand.
pub fn make_call_method<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_CALL_METHOD)
        .about("Calls a method")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_COMPONENT)
                .help("Specify the component address.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_METHOD)
                .help("Specify the method name.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_ARGS)
            .help("Specify the arguments, e.g. \"5\", \"hello\" or \"amount,resource_def\" (bucket).")
                .multiple(true),
        )
        // options
        .arg(
            Arg::with_name(ARG_TRACE)
                .long("trace")
                .help("Turns on tracing."),
        )
}

/// Handles a `call-method` request.
pub fn handle_call_method(matches: &ArgMatches) -> Result<(), Error> {
    let component = match_address(matches, ARG_COMPONENT)?;
    let method = match_string(matches, ARG_METHOD)?;
    let args = match_args(matches, ARG_ARGS)?;
    let trace = matches.is_present(ARG_TRACE);

    let mut configs = get_configs()?;
    let account = configs.default_account.ok_or(Error::NoDefaultAccount)?;
    let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
    let mut executor = TransactionExecutor::new(&mut ledger, configs.current_epoch, configs.nonce);
    let transaction = TransactionBuilder::new(&executor)
        .call_method(component, &method, args, Some(account))
        .deposit_all(account)
        .build()
        .map_err(Error::TransactionConstructionError)?;
    let receipt = executor.run(transaction, trace);

    println!("{:?}", receipt);
    if receipt.success {
        configs.nonce = executor.nonce();
        set_configs(configs)?;
        Ok(())
    } else {
        Err(Error::TransactionFailed)
    }
}
