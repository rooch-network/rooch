## Rooch indexer cli

A tool to handle rooch indexer.

### Usage

1. rooch indexer rebuild

```shell
rooch indexer rebuild --input ~/indexer.csv -d ~/.rooch -n main
```

Step 1, cleanup database files

```shell
rooch server clean -n main -d  {your rooch data dir} 
```

Step 2, start server to initialization genesis

```shell
rooch server start -n main -d  {your rooch data dir} 
```

Step 3, stop server

```shell
kill {server pid} or Ctrl-C
```

Step 4, export indexer data

```shell
rooch statedb export --output {your file} -d {your rooch data dir} -n main -m 3
```

Step 5 run indexer rebuild command

```shell
rooch indexer rebuild --input {your indexer file} -d {your rooch data dir} -n main
```

### Config

For better import throughput, please increase block cache size for RocksdbConfig of moveos. 16GiB is a good start.