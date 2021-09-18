use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::execution::*;
use radix_engine::model::*;
use scrypto::buffer::*;
use scrypto::rust::collections::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

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
                .short("t")
                .long("trace")
                .help("Turns on tracing."),
        )
}

/// Handles a `new-account` request.
pub fn handle_new_account(matches: &ArgMatches) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);

    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_data_dir()?);
    let mut runtime = Runtime::new(tx_hash, &mut ledger);

    // create XRD native token
    if runtime.get_resource(Address::RadixToken).is_none() {
        let mut metadata = HashMap::new();
        metadata.insert("symbol".to_owned(), "xrd".to_owned());
        metadata.insert("name".to_owned(), "Radix".to_owned());
        metadata.insert("description".to_owned(), "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.".to_owned());
        metadata.insert("url".to_owned(), "https://tokens.radixdlt.com".to_owned());
        let xrd = Resource {
            metadata,
            minter: Some(Address::System),
            supply: None,
        };
        runtime.put_resource(Address::RadixToken, xrd);
    }

    // publish smart account blueprint
    let package = Address::Package([1u8; 26]);
    if runtime.get_package(package).is_none() {
        runtime.put_package(
            package,
            Package::new(include_bytes!("../../../assets/account.wasm").to_vec()),
        );
    }

    // create new account
    let mut process = Process::new(0, trace, &mut runtime);
    let output = process
        .prepare_call_function(package, "Account", "new".to_owned(), Vec::new())
        .and_then(|invocation| process.run(invocation))
        .map_err(Error::TxnExecutionError)?;
    process.finalize().map_err(Error::TxnExecutionError)?;
    let component: Address = scrypto_decode(&output).map_err(Error::DataError)?;

    // allocate free XRD
    let mut buckets = HashMap::new();
    let bid = runtime.new_bucket_id();
    let bucket = Bucket::new(1_000_000.into(), Address::RadixToken);
    buckets.insert(bid, bucket);

    // deposit
    let mut process2 = Process::new(0, trace, &mut runtime);
    process2.put_resources(buckets, HashMap::new());
    process2
        .prepare_call_method(
            component,
            "deposit".to_owned(),
            vec![scrypto_encode(&scrypto::resource::Bucket::from(bid))],
        )
        .and_then(|invocation| process2.run(invocation))
        .map_err(Error::TxnExecutionError)?;
    process2.finalize().map_err(Error::TxnExecutionError)?;

    // flush
    runtime.flush();
    println!("New account: {}", component);

    // set as default config if not set
    if get_config(CONF_DEFAULT_ACCOUNT)?.is_none() {
        set_config(CONF_DEFAULT_ACCOUNT, &component.to_string())?;
        println!("No default account configured. This will be used as the default account.")
    }

    Ok(())
}
