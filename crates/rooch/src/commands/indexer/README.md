## Rooch indexer cli

A tool to handle rooch indexer.

### Usage

1. prepare genesis env:

```shell
rooch genesis init -n main -d <rooch_datadir>
```

2. rebuild indexer:

```shell
rooch indexer rebuild --input {your indexer file} -d {your rooch data dir} -n main
```

input: output of `rooch indexer export` command with `indexer` mode.

3. bench indexer built by `rooch indexer rebuild`:

```shell
rooch indexer bench -d {your rooch data dir} -n main
```