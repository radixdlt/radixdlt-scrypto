use crate::api::ObjectModuleId;
use crate::blueprints::package::TypePointer;
use crate::types::BlueprintId;
use crate::ScryptoSbor;
use radix_engine_common::address::AddressDisplayContext;
use radix_engine_common::types::NodeId;
use sbor::rust::fmt;
use utils::ContextualDisplay;

/// Identifies a specific event schema emitter by some emitter RENode.
///
/// This type is an identifier uses to identify the schema of events emitted by an RENode of some
/// [`NodeId`]. With this identifier, the schema for an event can be queried, obtained, and with
/// it, the SBOR encoded event data can be decoded and understood.
///
/// It is important to note that application events are always emitted by an RENode, meaning that
/// there is always an emitter of some [`NodeId`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub struct EventTypeIdentifier(pub Emitter, pub TypePointer);

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum Emitter {
    Function(BlueprintId),
    Method(NodeId, ObjectModuleId),
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for Emitter {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        match self {
            Self::Function(blueprint_id) => {
                write!(
                    f,
                    "Function {{ package: {}, blueprint_name: {} }}",
                    blueprint_id.package_address.display(*context),
                    blueprint_id.blueprint_name,
                )
            }
            Self::Method(node_id, module_id) => {
                write!(
                    f,
                    "Method {{ node: {}, module_id: {:?} }}",
                    node_id.display(*context),
                    module_id,
                )
            }
        }
    }
}
