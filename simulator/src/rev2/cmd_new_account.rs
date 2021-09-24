use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::engine::*;
use radix_engine::execution::*;
use radix_engine::model::*;
use scrypto::args;
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

    let mut ledger = FileBasedLedger::new(get_data_dir()?);
    let mut runtime = Runtime::new(sha256(Uuid::new_v4().to_string()), &mut ledger);

    // create XRD native token
    if runtime.get_resource_def(Address::RadixToken).is_none() {
        let mut metadata = HashMap::new();
        metadata.insert("symbol".to_owned(), "xrd".to_owned());
        metadata.insert("name".to_owned(), "Radix".to_owned());
        metadata.insert("description".to_owned(), "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.".to_owned());
        metadata.insert("url".to_owned(), "https://tokens.radixdlt.com".to_owned());
        runtime.put_resource_def(
            Address::RadixToken,
            ResourceDef {
                metadata,
                minter: Some(Address::System),
                auth: Some(Address::System),
                supply: 1_000_000.into(),
            },
        );
    }

    // publish smart account blueprint
    let package = Address::Package([1u8; 26]);
    if runtime.get_package(package).is_none() {
        runtime.put_package(
            package,
            Package::new(include_bytes!("../../../assets/account.wasm").to_vec()),
        );
    }

    // Create new account component with test XRD
    let mut proc = runtime.start_process(trace);
    let account: Address = proc
        .call_function((package, "Account".to_owned()), "new", args!())
        .and_then(decode_return)
        .map_err(Error::TxnExecutionError)?;
    let bucket =
        scrypto::resource::Bucket::from(proc.create_bucket(1_000_000.into(), Address::RadixToken));
    proc.call_method(account, "deposit", vec![scrypto_encode(&bucket)])
        .map_err(Error::TxnExecutionError)?;
    proc.finalize().map_err(Error::TxnExecutionError)?;
    runtime.commit();

    println!("New account: {}", account);

    // set as default config if not set
    if get_config(CONF_DEFAULT_ACCOUNT)?.is_none() {
        set_config(CONF_DEFAULT_ACCOUNT, &account.to_string())?;
        println!("No default account configured. This will be used as the default account.")
    }

    Ok(())
}
