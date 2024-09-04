// We used to use automod, but it breaks various tools
// such as cargo fmt, so let's just list them explicitly.
mod frame;
mod kernel;
mod kernel_open_substate;
mod panics;
mod test_environment;
mod transaction_executor;
mod transaction_multi_threaded;
