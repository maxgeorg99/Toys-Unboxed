#!/usr/bin/env just --justfile

release:
  cargo build --release

run:
  cargo run

generate:
    spacetime generate --lang rust --project-path spacetimedb --out-dir crates/game/module_bindings

start-local:
    spacetime start

deploy-local:
    spacetime publish --project-path spacetimedb --server local

deploy-maincloud:
    spacetime publish --project-path spacetimedb --server maincloud