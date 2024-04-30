set -ex

cargo test --package radix-transaction-scenarios --lib -- runners::dumper::test::update_all_generated_scenarios --exact --nocapture