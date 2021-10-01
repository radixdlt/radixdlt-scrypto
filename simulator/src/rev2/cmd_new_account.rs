use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::transaction::*;

use crate::ledger::*;
use crate::rev2::*;
use crate::utils::*;

const ARG_TRACE: &str = "TRACE";

/// Constructs a `new-account` subcommand.
pub fn make_new_account<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_NEW_ACCOUNT)
        .about("Creates an account")
        .version(crate_version!())
        // options
        .arg(
            Arg::with_name(ARG_TRACE)
                .long("trace")
                .help("Turns on tracing."),
        )
}

/// Handles a `new-account` request.
pub fn handle_new_account(matches: &ArgMatches) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);

    let mut configs = get_configs()?;
    let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
    let mut executor = TransactionExecutor::new(&mut ledger, configs.current_epoch, configs.nonce);
    let transaction = TransactionBuilder::new()
        .new_account()
        .build()
        .map_err(Error::TransactionConstructionError)?;
    let receipt = executor.run(&transaction, trace);

    dump_receipt(&transaction, &receipt);
    if receipt.success {
        configs.nonce = executor.nonce();
        configs.default_account = configs
            .default_account
            .or(Some(receipt.nth_component(0).unwrap()));
        set_configs(configs)?;
        Ok(())
    } else {
        Err(Error::TransactionFailed)
    }
}
