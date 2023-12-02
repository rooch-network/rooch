# Rooch standard library

Rooch stdlib include the MoveOS standard library and Rooch framework.

This crate is used to compile the moveos-stdlib and rooch-framework, and then generate the compiled Move bytecode, documentation, and error descriptions for use by the Move Explain tool.

## Compile, check compatibility with previous version

1. Compile 

```bash
cargo run --package rooch-framework-release --bin rooch-framework-release
```

This command will compile the latest moveos-stdlib and rooch-framework, and then check the compatibility with previous one (if exists).

## Release a new version

1. Compile with given version number

The version number must start from 1 and increase continuously, or the command will abort.

```bash
cargo run --package rooch-framework-release --bin rooch-framework-release -- --version 1
```

All modified source files and generated files should be committed.