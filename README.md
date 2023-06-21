# eray

Simple TUI shader graph editor and OpenGL viewer with raytraced screenshot
capabilities.

## Best-case scenario objectives

ISIM:
- [x] Object loading
- [x] Generic graph-based computation pipeline
- [~] Raytracer using the shader graph (texture mapping is currently bugged)

POGL:
- [~] Scene conversion for OpenGL
- [ ] Shaderlib port to OpenGL shaders
- [ ] Live OpenGL view

TIFO:
- [~] Post-processing pipeline setup
- [ ] Library of post-processing effects

Bonus:
- [ ] .eray parsing and dumping
- [ ] TUI editor

## Running

An example graph is already defined in `src/main.rs`, just run the following
command in your shell (with the rust toolchain installed) and the output color
texture will be saved as `color.ppm`.

```sh
cargo run
```

## .eray shader files

Shader graphs are meant to be fully representable as (and therefore storable to
and loadable from) .eray files following the grammar defined in
`src/lib/pest/grammar.pest`.
