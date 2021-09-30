use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::transaction::*;
use scrypto::rust::str::FromStr;
use scrypto::types::*;

use crate::ledger::*;
use crate::rev2::*;
use crate::utils::*;

const ARG_TRACE: &str = "TRACE";
const ARG_AMOUNT: &str = "AMOUNT";
const ARG_RESOURCE_ADDRESS: &str = "RESOURCE_ADDRESS";
const ARG_RECIPIENT_ADDRESS: &str = "RECIPIENT_ADDRESS";

/// Constructs a `transfer` subcommand.
pub fn make_transfer<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_TRANSFER)
        .about("Transfers resource")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_TRACE)
                .long("trace")
                .help("Turns on tracing."),
        )
        .arg(
            Arg::with_name(ARG_AMOUNT)
                .help("Specify the amount to transfer.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_RESOURCE_ADDRESS)
                .help("Specify the resource definition address.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_RECIPIENT_ADDRESS)
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
    let resource_address: Address = matches
        .value_of(ARG_RESOURCE_ADDRESS)
        .ok_or_else(|| Error::MissingArgument(ARG_RESOURCE_ADDRESS.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)?;
    let recipient_address: Address = matches
        .value_of(ARG_RECIPIENT_ADDRESS)
        .ok_or_else(|| Error::MissingArgument(ARG_RECIPIENT_ADDRESS.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)?;

    match get_config(CONF_DEFAULT_ACCOUNT)? {
        Some(a) => {
            let account: Address = a.as_str().parse().map_err(Error::InvalidAddress)?;

            let mut ledger = FileBasedLedger::new(get_data_dir()?);
            let mut executor = TransactionExecutor::new(&mut ledger, 0, 0); // TODO: fix nonce and epoch.

            let abi = executor
                .export_abi_by_component(account, trace)
                .map_err(Error::TxnExecutionError)?;

            let transaction = TransactionBuilder::new()
                .call_method(
                    &abi,
                    account,
                    "withdraw",
                    vec![&amount.to_string(), &resource_address.to_string()],
                )
                .call_method(
                    &abi,
                    recipient_address,
                    "deposit",
                    vec![&format!("{},{}", amount, resource_address)],
                )
                .build_with(Some(account))
                .map_err(Error::TxnConstructionErr)?;

            let receipt = executor.execute(&transaction, trace);
            dump_receipt(&receipt);

            if receipt.success {
                Ok(())
            } else {
                Err(Error::TransactionFailed)
            }
        }
        None => Err(Error::NoDefaultAccount),
    }
}
