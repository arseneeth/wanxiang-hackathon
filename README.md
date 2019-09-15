# PokeSubs

## Overview

PokeSubs are born from the mixing of "Pokemon-go" and "Cryptokitties".

PokeSubs are non fungible tokens that are cute ! You can play with them, look for them and exchange them with other PokeSubs fans.

There are two types of PokeSub users:
* PokeFarmer
* PokeHunter

The PokeFarmer are priviledged users who can generate PokeSubs for PokeHunters. They get a PokeCoin reward from transaction fee everytime the PokeSub will be exchanged in the future.

PokeFarmers can generate a PokeSub for a given PokeHunter only once.

The more a PokeHunter is active in the chain the more he is likely to get super cute PokeSubs.

PokeHunter activity increases with:
* the number of PokeSubs he owns
* how much he interacts with his PokeSubs
* how often does he exchange PokeSubs with other PokeHunters
* how many PokeFarmers he met

## Getting started

To run PokeSubs you would need `Rust` and `Substrate` being installed on your machine.

To run pokesubs clone the repo, then:

`cd pokesubs`

Build webassembly binary `./scripts/build.sh`

Build bytecode `cargo build --release`

Run substrate node `./target/release/pokesubs --dev`

