use colored::*;
use radix_engine::model::*;

use crate::transaction::*;

pub fn print_receipt(receipt: TransactionReceipt) {
    println!("{}", "Instructions:".bold().green());
    for inst in receipt.transaction.instructions {
        println!("|- {:02x?}", inst);
    }
    println!();

    println!("{}", "Results:".bold().green());
    for result in receipt.results {
        println!("|- {:02x?}", result);
    }
    println!();

    println!("{}", "Logs:".bold().green());
    for (level, msg) in receipt.logs {
        let (l, m) = match level {
            Level::Error => ("ERROR".red(), msg.red()),
            Level::Warn => ("WARN".yellow(), msg.yellow()),
            Level::Info => ("INFO".green(), msg.green()),
            Level::Debug => ("DEBUG".cyan(), msg.cyan()),
            Level::Trace => ("TRACE".normal(), msg.normal()),
        };
        println!("|- [{:5}] {}", l, m);
    }
}
