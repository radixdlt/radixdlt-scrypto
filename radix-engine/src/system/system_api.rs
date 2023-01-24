use crate::kernel::SubstateApi;

/// APIs for accessing functionalities provided by system.
///
///
/// For now, `SystemApi` completely follows the semantics a subset of `KernelApi`.
///
pub trait SystemApi: SubstateApi {}
