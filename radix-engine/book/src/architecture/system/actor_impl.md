# Actor Implementation

The system layer is responsible for defining the [Actor](../application/actor/README.md) abstraction.

The state of the current actor is stored per call frame as `CallFrameData`. The system exposes an
interface which can access the state of the currently acting object (if there is one). Thus, the system
prevents higher layers from accessing state of call frame objects which aren't the actor.