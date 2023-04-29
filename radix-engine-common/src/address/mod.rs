mod decoder;
mod display;
mod encoder;
mod errors;
mod hrpset;

pub use decoder::Bech32Decoder;
pub use display::*;
pub use encoder::Bech32Encoder;
pub use errors::*;
pub use hrpset::HrpSet;

pub mod test_addresses {
    use crate::types::{NodeId, ResourceAddress};

    // The system addresses are defined in `radix-engine-interface`, but some
    // tests have a need for some placeholder addresses - so define them here so
    // we only need to update one place if they change in future.

    pub const FUNGIBLE_RESOURCE: ResourceAddress = ResourceAddress::new_or_panic([
        93, 166, 99, 24, 198, 49, 140, 97, 245, 166, 27, 76, 99, 24, 198, 49, 140, 247, 148, 170,
        141, 41, 95, 20, 230, 49, 140, 99, 24, 198,
    ]);
    pub const FUNGIBLE_RESOURCE_NODE_ID: NodeId = FUNGIBLE_RESOURCE.into_node_id();
    pub const FUNGIBLE_RESOURCE_SIM_ADDRESS: &'static str =
        "resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3";
    pub const FUNGIBLE_RESOURCE_NO_NETWORK_STRING: &'static str =
        "NodeId(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6)";
}
