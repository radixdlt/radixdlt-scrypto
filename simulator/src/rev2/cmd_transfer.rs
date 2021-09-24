use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::execution::*;
use scrypto::args;
use scrypto::rust::str::FromStr;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::ledger::*;
use crate::rev2::*;

const ARG_TRACE: &str = "TRACE";
const ARG_AMOUNT: &str = "AMOUNT";
const ARG_RESOURCE: &str = "RESOURCE";
const ARG_RECIPIENT: &str = "RECIPIENT";

/// Constructs a `transfer` subcommand.
pub fn make_transfer<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_TRANSFER)
        .about("Transfers resources")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_TRACE)
                .short("t")
                .long("trace")
                .help("Turns on tracing."),
        )
        .arg(
            Arg::with_name(ARG_AMOUNT)
                .help("Specify the amount to transfer.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_RESOURCE)
                .help("Specify the resource address.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_RECIPIENT)
                .help("Specify the recipient address.")
                .required(true),
        )
}

/// Handles a `transfer` request.
pub fn handle_transfer(matches: &ArgMatches) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);
    let amount = Amount::from_str(
        matches
            .value_of(ARG_AMOUNT)
            .ok_or_else(|| Error::MissingArgument(ARG_AMOUNT.to_owned()))?,
    )
    .map_err(|_| Error::InvalidAmount)?;
    let resource: Address = matches
        .value_of(ARG_RESOURCE)
        .ok_or_else(|| Error::MissingArgument(ARG_RESOURCE.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)?;
    let recipient: Address = matches
        .value_of(ARG_RECIPIENT)
        .ok_or_else(|| Error::MissingArgument(ARG_RECIPIENT.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)?;

    match get_config(CONF_DEFAULT_ACCOUNT)? {
        Some(a) => {
            let account: Address = a.as_str().parse().map_err(Error::InvalidAddress)?;

            let mut ledger = FileBasedLedger::new(get_data_dir()?);
            let mut runtime = Runtime::new(sha256(Uuid::new_v4().to_string()), &mut ledger);
            let mut process = runtime.start_process(trace);
            let bid = process.reserve_bucket_id();
            process
                .call_method(account, "withdraw", args!(amount, resource))
                .map_err(Error::TxnExecutionError)?;
            process
                .move_to_bucket(amount, resource, bid)
                .map_err(Error::TxnExecutionError)?;
            process
                .call_method(
                    recipient,
                    "deposit",
                    args!(scrypto::resource::Bucket::from(bid)),
                )
                .map_err(Error::TxnExecutionError)?;
            process.finalize().map_err(Error::TxnExecutionError)?;
            runtime.commit();

            println!("Resource transferred!");
            Ok(())
        }
        None => Err(Error::NoDefaultAccount),
    }
}
