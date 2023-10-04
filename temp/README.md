## IDEAS:

- "map" a JSON file to a set of properties or an object
- distro-specific read/write actions
- generate UI baed on document schema
- composability of properties: define a schema/property once and use it in multiple places
- read "style" (read once, read on demand, read on change)
- write "validation"
- action aliases
- chaining actions for a single read/write

## QUESTIONS:

- should this module schema be binplaced on a device OR should it be used to generate code that is binplaced on a device?
  - binplacing the schema
    - smaller footprint
    - devices can "self describe" using the schema
    -
  - generating code

- what about modules that cannot be represented this way (like tpm)
  - some modules might NEED to be implmented as code or are too complicated
  - ideally this would be a "last resort" option but may end up being common for lower level system components
  - in this case, the implmentation should be as simple as possible (generating alot of code) and sharing code with the action runner

- what about modules that are more proceedural (like commandrunner)
  - some modules might NEED to maintain an internal state

- what about modules that are expensive (like deviceinfo)
  - some modules always report the same value and can be optimized to load (perform the read action) once at startup