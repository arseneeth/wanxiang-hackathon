# PokeSubs

To run PokeSubs you would need `Rust` and `Substrate` being installed on your machine.

To run pokesubs clone the repo, then:

`cd pokesubs`

Build webassembly binary `./scripts/build.sh`

Build bytecode `cargo build --release`

Run substrate node `./target/release/pokesubs --dev`

