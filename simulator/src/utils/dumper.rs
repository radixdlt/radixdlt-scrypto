use colored::*;
use radix_engine::model::*;
use radix_engine::transaction::*;
use radix_engine::utils::*;
use scrypto::types::*;

use crate::utils::*;

pub fn dump_receipt(transaction: &Transaction, receipt: &Receipt) {
    println!(
        "{} {}",
        "Transaction Status:".bold().green(),
        if receipt.success {
            "SUCCESS".blue()
        } else {
            "FAILURE".red()
        }
        .bold()
    );

    println!("{}", "Instructions:".bold().green());
    for (last, inst) in transaction.instructions.iter().identify_last() {
        println!("{} {:?}", list_item_prefix(last), inst);
    }

    println!("{}", "Results:".bold().green());
    for (last, result) in receipt.results.iter().identify_last() {
        let msg = match result {
            Ok(r) => match r {
                Some(rtn) => {
                    format!("Ok({})", format_sbor(rtn).unwrap())
                }
                None => "Ok".to_string(),
            },
            Err(err) => format!("Err({:?})", err),
        };
        println!("{} {}", list_item_prefix(last), msg);
    }

    println!("{} {}", "Logs:".bold().green(), receipt.logs.len());
    for (last, (level, msg)) in receipt.logs.iter().identify_last() {
        let (l, m) = match level {
            Level::Error => ("ERROR".red(), msg.red()),
            Level::Warn => ("WARN".yellow(), msg.yellow()),
            Level::Info => ("INFO".green(), msg.green()),
            Level::Debug => ("DEBUG".cyan(), msg.cyan()),
            Level::Trace => ("TRACE".normal(), msg.normal()),
        };
        println!("{} [{:5}] {}", list_item_prefix(last), l, m);
    }

    println!(
        "{} {}",
        "New Addresses:".bold().green(),
        receipt.new_addresses.len()
    );
    for (last, address) in receipt.new_addresses.iter().identify_last() {
        let ty = match address {
            Address::Package(_) => "Package",
            Address::Component(_) => "Component",
            Address::ResourceDef(_) => "ResourceDef",
        };
        println!("{} {}: {}", list_item_prefix(last), ty, address);
    }
}
