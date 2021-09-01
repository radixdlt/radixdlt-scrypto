use colored::*;
use radix_engine::model::*;

use crate::transaction::*;
use crate::utils::*;

pub fn dump_receipt(receipt: TransactionReceipt) {
    println!(
        "{} {}\n",
        "Transaction Status:".bold().green(),
        if receipt.success {
            "SUCCESS".blue()
        } else {
            "FAILURE".red()
        }
        .bold()
    );

    println!("{}", "Instructions:".bold().green());
    for (last, inst) in receipt.transaction.instructions.iter().identify_last() {
        println!("{} {:02x?}", item_prefix(last), inst);
    }
    println!();

    println!("{}", "Results:".bold().green());
    for (last, result) in receipt.results.iter().identify_last() {
        println!("{} {:02x?}", item_prefix(last), result);
    }
    println!();

    println!("{}", "Logs:".bold().green());
    for (last, (level, msg)) in receipt.logs.iter().identify_last() {
        let (l, m) = match level {
            Level::Error => ("ERROR".red(), msg.red()),
            Level::Warn => ("WARN".yellow(), msg.yellow()),
            Level::Info => ("INFO".green(), msg.green()),
            Level::Debug => ("DEBUG".cyan(), msg.cyan()),
            Level::Trace => ("TRACE".normal(), msg.normal()),
        };
        println!("{} [{:5}] {}", item_prefix(last), l, m);
    }
    println!();

    println!(
        "{} {} ms\n",
        "Execution Time:".bold().green(),
        receipt.execution_time
    );
}
