# Raytracer build with Rust

## Screenshots

<sub>`cargo run --release -- scene/helmet.json cmd no-animation samples=64 1280x720 monte_carlo=1`</sub>
<img src="data/renderings/output_2022-5-16_14-48-33_00000000.png" width="720">
<br>

<sub>`cargo run --release -- scene/room.json scene/kbert.json cmd samples=64 1280x720 monte_carlo=1`</sub>
<img src="data/renderings/output_2022-5-16_15-41-8_00000000.png" width="720">
<br>

<sub>`cargo run --release -- scene/latern.json cmd samples=64 1280x720 monte_carlo=1`</sub>
<img src="data/renderings/output_2022-5-16_15-50-6_00000000.png" width="720">
<br>

<sub>`cargo run --release -- scene/corset.json samples=256 1280x720 monte_carlo=1`</sub>
<img src="data/renderings/output_2022-5-16_15-52-24_00000000.png" width="720">
<br>

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
cargo watch -s "cargo run --release -- scene/helmet.json" -w src/

#run without watch
cargo run --release SCENE_NAME
cargo run --release -- scene/helmet.json
```

## command line args

* `PATH_TO_SCENE.json` -- set the path to a scene.json file (you can set multiple scene files)
* `no-animation` -- disable animation
* `cmd` -- cmd version without window
* `samples=1234` -- set samples amount
* `800x600` -- set render resolution
* `monte_carlo=1` -- enable monte carlo rendering


```bash
#example
cargo run --release -- scene/helmet.json no-animation samples=32 800x600 monte_carlo=1
```


## Linux (Ubuntu) requirements
```bash
sudo apt install cmake pkg-config libssl-dev build-essential cmake xorg-dev
```