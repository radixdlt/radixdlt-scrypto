use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::transaction::*;

use crate::ledger::*;
use crate::resim::*;

const ARG_AMOUNT: &str = "AMOUNT";
const ARG_RESOURCE_DEF: &str = "RESOURCE_DEF";
const ARG_RECIPIENT: &str = "RECIPIENT";

const ARG_TRACE: &str = "TRACE";
const ARG_SIGNERS: &str = "SIGNERS";

/// Constructs a `transfer` subcommand.
pub fn make_transfer<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_TRANSFER)
        .about("Transfers resource to another account")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_AMOUNT)
                .help("Specify the amount to transfer.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_RESOURCE_DEF)
                .help("Specify the resource definition address.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_RECIPIENT)
                .help("Specify the recipient address.")
                .required(true),
        )
        // options
        .arg(
            Arg::with_name(ARG_TRACE)
                .long("trace")
                .help("Turn on tracing."),
        )
        .arg(
            Arg::with_name(ARG_SIGNERS)
                .long("signers")
                .takes_value(true)
                .help("Specify the transaction signers, separated by comma."),
        )
}

/// Handles a `transfer` request.
pub fn handle_transfer(matches: &ArgMatches) -> Result<(), Error> {
    let amount = match_amount(matches, ARG_AMOUNT)?;
    let resource_def = match_address(matches, ARG_RESOURCE_DEF)?;
    let recipient = match_address(matches, ARG_RECIPIENT)?;
    let trace = matches.is_present(ARG_TRACE);
    let signers = match_signers(matches, ARG_SIGNERS)?;

    let mut configs = get_configs()?;
    let account = configs.default_account.ok_or(Error::NoDefaultAccount)?;
    let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
    let mut executor = TransactionExecutor::new(&mut ledger, configs.current_epoch, configs.nonce);
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(amount, resource_def, account.0)
        .drop_all_bucket_refs()
        .deposit_all_buckets(recipient)
        .build(signers)
        .map_err(Error::TransactionConstructionError)?;
    let receipt = executor.run(transaction, trace).unwrap();

    println!("{:?}", receipt);
    if receipt.success {
        configs.nonce = executor.nonce();
        set_configs(configs)?;
        Ok(())
    } else {
        Err(Error::TransactionFailed)
    }
}
