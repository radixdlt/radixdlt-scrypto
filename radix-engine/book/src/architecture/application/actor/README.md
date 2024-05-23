# Actor

An actor is the acting entity currently executing and determines what state can be directly read.

There are five types of actors:

| Actor Name    | Description                                                                                           |
|---------------|-------------------------------------------------------------------------------------------------------|
| Root          | The initial application of all transactions.                                                          |
| Method        | A call on an object. Has direct access to state of the running object.                                |
| Function      | A stateless function call. Has no direct access to any state.                                         |
| Method Hook   | A callback call on an object defined by the system. Has direct access to state of the running object. |
| Function Hook | A callback stateless function call defined by the system. Has no direct access to any state.          |

