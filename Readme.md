# Raytracer build with Rust

## Features
* Shadow (it can be controled via `cast_shadow` and `receive_shadow`)
* Reflection
* Refraction
* Phong shading (sort of)
* Alpha/Opacity/Transparency via material setting or alpha map
* Supported shapes
  * Sphere
  * Triangle-Mesh
* Texture mapping
* Normal mapping (bump mapping)
* Wavefront (obj) object loading
* Anti-Aliasing
* DOF (Depth of field)
* Different light types (directional, point, spot)
* Monte Carlo raytracing (sort of)
* Fog
* Matrix based transformations
* Json based scenes
* GLTF based scenes
* PBR (sort of)
* Basic animation support

## usage
use cargo watch to run release version:
```bash
#install:
cargo install cargo-watch

#run with watch:
cargo watch -s "cargo run --release" -w src/

#run without watch
cargo run --release
```

## Linux (Ubuntu) requirements
```
sudo apt install cmake pkg-config libssl-dev build-essential cmake xorg-dev
```