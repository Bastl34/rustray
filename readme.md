# some fun with rust

go to a specific directory and run:
```bash
cargo watch -x run
```

use cargo watch to run release version:
```bash
#install:
cargo install cargo-watch

#run:
cargo watch -s "cargo run --release" -w src/
```

## Linux (Ubuntu) requirements
```
sudo apt install cmake pkg-config libssl-dev build-essential cmake xorg-dev
```