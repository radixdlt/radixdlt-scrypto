use crate::api::ModuleId;
use crate::internal_prelude::*;
use crate::types::BlueprintId;
use radix_common::address::AddressDisplayContext;
use radix_common::types::NodeId;
use radix_rust::ContextualDisplay;
use sbor::rust::fmt;
use sbor::rust::string::String;

/// Identifies a specific event schema emitter by some emitter RENode.
///
/// This type is an identifier uses to identify the schema of events emitted by an RENode of some
/// [`NodeId`]. With this identifier, the schema for an event can be queried, obtained, and with
/// it, the SBOR encoded event data can be decoded and understood.
///
/// It is important to note that application events are always emitted by an RENode, meaning that
/// there is always an emitter of some [`NodeId`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub struct EventTypeIdentifier(pub Emitter, pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum Emitter {
    Function(BlueprintId),
    Method(NodeId, ModuleId),
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
                    "Function {{ blueprint_id: {} }}",
                    blueprint_id.display(*context),
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

impl EventTypeIdentifier {
    pub fn len(&self) -> usize {
        let emitter_size = match &self.0 {
            Emitter::Function(blueprint_id) => blueprint_id.len(),
            Emitter::Method(node_id, _module_1) => node_id.len() + 1,
        };
        emitter_size + self.1.len()
    }
}
