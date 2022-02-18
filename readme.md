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