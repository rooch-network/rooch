## Rooch indexer cli

A tool to handle rooch indexer.

### Usage

1. rooch indexer rebuild

```shell
rooch indexer rebuild --input ~/utxo.txt -d ~/.rooch -n main
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

Step 4 run indexer genesis-utxo command

```shell
rooch indexer genesis-utxo --input {your utxo file} -d {your rooch data dir} -n main
```

### Config

For better import throughput, please increase block cache size for RocksdbConfig of moveos. 16GiB is a good start.