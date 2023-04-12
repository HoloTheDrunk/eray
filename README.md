# eray

Simple TUI shader editor and OpenGL viewer with raytraced screenshot
capabilities.

# Shader files (.eray)

.eray files are split into sections:

## Node type signature

An .eray file represents a fragment shader, which is also a node.
As such, it needs to provide information on its inputs and outputs.

## Custom node definition (import)

Shaders defined in other .eray files can be required in this section, causing a
controlled error if no loaded custom shader matches the name and inputs/outputs.

## Node declaration

Nodes are instantiated with a name and a shader name.

## Node linking

Nodes' sockets have to be linked to create the graph.
This section also allows instantiation of standard type values.
