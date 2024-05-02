# Application Environment

## Call Frame

On function entry a new call frame is created for the execution of the function. The call frame
contains all owned and referenced objects usable by the running function.

The System layer exposes an API which only allows state to be read from and written to by the current
*actor*, or object which is being called.


The application layer
also accepts an SBOR encoded value representing the argument passed into the function. All owned objects
and referenced objects in the argument is guaranteed to be valid (i.e. owned/referenced in the current
call frame).


All references in the
frame are 