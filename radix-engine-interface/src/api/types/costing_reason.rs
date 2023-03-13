#[derive(Clone, Copy, Debug)]
pub enum ClientCostingReason {
    RunWasm,
    RunNative,
    RunSystem,
}
