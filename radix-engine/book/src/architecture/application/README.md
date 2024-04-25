# Application Layer

The application layer is responsible for defining high level logic which manipulates 
object state.

Applications in Radix Engine are organized in a format called a *Package* which contain zero or
more *Blueprints*, each of which has a unique name within the package. Packages are uniquely identified
through a `<package_address>`. Blueprints are globally identifiable by `<package_address> + <blueprint_name>`.

Each blueprint defines type information and logic which can create/manipulate/destroy *objects*,
which are instantiations of blueprints.


