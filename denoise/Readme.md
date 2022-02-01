# Denoise test with odin

* download odin https://github.com/OpenImageDenoise/oidn/releases
* extract it to: `lib/odin`
* may adjust version numbers in `build.rs`

## Mac (arm based)

```bash
rustup target add x86_64-apple-darwin
#rustup target add aarch64-apple-darwin

OIDN_DIR=lib/oidn cargo run --target x86_64-apple-darwin
```

## Mac (intel based)

```bash

OIDN_DIR=lib/oidn cargo run --release
```