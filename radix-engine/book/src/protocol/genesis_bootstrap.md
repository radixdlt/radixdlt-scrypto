# Genesis Bootstrap

Bootstrapping a Radix Engine requires flashing several system substates and then the execution
of several genesis transactions.

Specifically, the substates of the `Package` blueprint and object module blueprints are flashed.

Once flashed, `Package::publish` calls may now be called to create the rest of the native
blueprints.
