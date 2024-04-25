# Application Layer

Applications in Radix Engine are organized in a format called a *Package* which is uniquely
identified through a `package_address`.

Each package defines zero or more *Blueprints* each with a unique name within the package. A
blueprint is globally identifiable by `<package_address> + <blueprint_name>`.

Each blueprint defines type information and logic which can then create/manipulate/destroy *objects* 
which are instantiations of specific blueprints.
