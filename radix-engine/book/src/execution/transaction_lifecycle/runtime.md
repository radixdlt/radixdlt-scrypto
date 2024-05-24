# Transaction Runtime

Once transaction bootup has finished, the [Transaction Processor Blueprint](../../native/transaction_processor/blueprint.md)
function `run` is then executed with transaction data as its argument. It executes on top of the initial call frame created
during kernel initialization in a standard [application environment](../environment/README.md).

Once the `run` function has finished executing transaction shutdown begins.