## Rooch Statedb tool

A tool to export/import rooch statedb.

### Usage

#### genesis

`genesis` is a subcommand to generate rooch statedb from utxo and ord source files. Run it before starting rooch node.

Source data needed by `genesis` for Rooch MainNet could be found [here](TODO).

For protecting the data integrity, verify checksum file's sha256 before running `genesis` command.

checksum file's sha256:

`cbd0c0a9f4c0f308c29b83dc50b4b4f2684f7eb17df0d66446f8b1f86589dce5`

calculated by:

```shell
sha256sum checksum
```

result:

```shell
sha256sum checksum
cbd0c0a9f4c0f308c29b83dc50b4b4f2684f7eb17df0d66446f8b1f86589dce5  checksum
```

#### Other Subcommands

TODO
