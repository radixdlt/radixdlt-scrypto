# Transaction Runtime

Once transaction bootup has finished, the `TRANSACTION_PROCESSOR` blueprint function `run` is then
executed with transaction data as its argument. It executes on top of the initial call frame created
during kernel initialization.

Once the `run` function has finished executing transaction shutdown begins.