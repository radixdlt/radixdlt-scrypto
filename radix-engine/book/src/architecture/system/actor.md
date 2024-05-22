# Actor

An actor is the acting entity currently executing. The system layer exposes an Api for retrieving
state and additional info of the current actor.

There are five types of actors:

| Actor Name    | Description                                                                                           |
|---------------|-------------------------------------------------------------------------------------------------------|
| Root          | The initial application of all transactions.                                                          |
| Method        | A call on an object. Has direct access to state of the running object.                                |
| Function      | A stateless function call. Has no direct access to any state.                                         |
| Method Hook   | A callback call on an object defined by the system. Has direct access to state of the running object. |
| Function Hook | A callback stateless function call defined by the system. Has no direct access to any state.          |

## Implementation

The state of the current actor is stored per call frame as `CallFrameData`. The system exposes an
interface which can access the state of the currently acting object (if there is one). Thus, the system
prevents higher layers from accessing state of call frame objects which aren't the actor.