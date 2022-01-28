use clap::{crate_version, App, Arg, ArgMatches};
use radix_engine::transaction::*;

use crate::ledger::*;
use crate::resim::*;

const ARG_AMOUNT: &str = "AMOUNT";
const ARG_RESOURCE_ADDRESS: &str = "RESOURCE_ADDRESS";
const ARG_MINT_BADGE_ADDR: &str = "MINT_BADGE_ADDRESS";

const ARG_TRACE: &str = "TRACE";
const ARG_SIGNERS: &str = "SIGNERS";

/// Constructs a `mint` subcommand.
pub fn make_mint<'a>() -> App<'a> {
    App::new(CMD_MINT)
        .about("Mints resource")
        .version(crate_version!())
        .arg(
            Arg::new(ARG_AMOUNT)
                .help("Specify the amount to mint.")
                .required(true),
        )
        .arg(
            Arg::new(ARG_RESOURCE_ADDRESS)
                .help("Specify the resource definition address.")
                .required(true),
        )
        .arg(
            Arg::new(ARG_MINT_BADGE_ADDR)
                .help("Specify the mint auth resource definition address.")
                .required(true),
        )
        // options
        .arg(Arg::new(ARG_TRACE).long("trace").help("Turn on tracing."))
        .arg(
            Arg::new(ARG_SIGNERS)
                .long("signers")
                .takes_value(true)
                .help("Specify the transaction signers, separated by comma."),
        )
}

/// Handles a `mint` request.
pub fn handle_mint(matches: &ArgMatches) -> Result<(), Error> {
    let amount = match_amount(matches, ARG_AMOUNT)?;
    let resource_address = match_address(matches, ARG_RESOURCE_ADDRESS)?;
    let mint_badge_addr = match_address(matches, ARG_MINT_BADGE_ADDR)?;
    let trace = matches.is_present(ARG_TRACE);
    let signers = match_signers(matches, ARG_SIGNERS)?;

    let mut configs = get_configs()?;
    let account = configs.default_account.ok_or(Error::NoDefaultAccount)?;
    let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
    let mut executor =
        TransactionExecutor::new(&mut ledger, configs.current_epoch, configs.nonce, trace);
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(
            &ResourceAmount::Fungible {
                amount: 1.into(),
                resource_address: mint_badge_addr,
            },
            account.0,
        )
        .mint(amount, resource_address, mint_badge_addr)
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
