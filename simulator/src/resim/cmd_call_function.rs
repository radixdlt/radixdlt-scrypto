use clap::{crate_version, App, Arg, ArgMatches};
use radix_engine::transaction::*;

use crate::ledger::*;
use crate::resim::*;

const ARG_PACKAGE: &str = "PACKAGE_ADDRESS";
const ARG_NAME: &str = "BLUEPRINT_NAME";
const ARG_FUNCTION: &str = "FUNCTION";
const ARG_ARGS: &str = "ARGS";

const ARG_TRACE: &str = "TRACE";
const ARG_SIGNERS: &str = "SIGNERS";

/// Constructs a `call-function` subcommand.
pub fn make_call_function<'a>() -> App<'a> {
    App::new(CMD_CALL_FUNCTION)
        .about("Calls a function")
        .version(crate_version!())
        .arg(
            Arg::new(ARG_PACKAGE)
                .help("Specify the blueprint package address.")
                .required(true),
        )
        .arg(
            Arg::new(ARG_NAME)
                .help("Specify the blueprint name.")
                .required(true),
        )
        .arg(
            Arg::new(ARG_FUNCTION)
                .help("Specify the function name.")
                .required(true),
        )
        .arg(
            Arg::new(ARG_ARGS)
            .help("Specify the arguments, e.g. \"5\", \"hello\", \"amount,resource_address\" for Bucket, or \"#id1,#id2,..,resource_address\" for NFT Bucket.")
                .multiple_values(true),
        )
        // options
        .arg(
            Arg::new(ARG_TRACE)
                .long("trace")
                .help("Turn on tracing."),
        )
        .arg(
            Arg::new(ARG_SIGNERS)
                .long("signers")
                .takes_value(true)
                .help("Specify the transaction signers, separated by comma."),
        )
}

/// Handles a `call-function` request.
pub fn handle_call_function(matches: &ArgMatches) -> Result<(), Error> {
    let package = match_address(matches, ARG_PACKAGE)?;
    let name = match_string(matches, ARG_NAME)?;
    let function = match_string(matches, ARG_FUNCTION)?;
    let args = match_args(matches, ARG_ARGS)?;
    let trace = matches.is_present(ARG_TRACE);
    let signers = match_signers(matches, ARG_SIGNERS)?;

    let mut configs = get_configs()?;
    let account = configs.default_account.ok_or(Error::NoDefaultAccount)?;
    let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
    let mut executor =
        TransactionExecutor::new(&mut ledger, configs.current_epoch, configs.nonce, trace);
    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, &name, &function, args, Some(account.0))
        .call_method_with_all_resources(account.0, "deposit_batch")
        .build(signers)
        .map_err(Error::TransactionConstructionError)?;
    let receipt = executor.run(transaction).unwrap();

    println!("{:?}", receipt);
    if receipt.error.is_none() {
        configs.nonce = executor.nonce();
        set_configs(configs)?;
        Ok(())
    } else {
        Err(Error::TransactionFailed)
    }
}
