# Rooch standard library

Rooch stdlib include the MoveOS standard library and Rooch framework.

This crate is used to compile the moveos-stdlib and rooch-framework, and then generate the compiled Move bytecode, documentation, and error descriptions for use by the Move Explain tool.

## Compile and save the latest stdlib 

1. Compile 

```bash
cargo run --package rooch-framework-release --bin rooch-framework-release
```

This command will compile the moveos-stdlib and rooch-framework, and then check the compatibility with previous one (if exists), and finally save the new compiled stdlib.

## Release a new version

1. Compile with given version number

```bash
cargo run --package rooch-framework-release --bin rooch-framework-release -- --version 5
```

All modified source files and generated files should be committed.