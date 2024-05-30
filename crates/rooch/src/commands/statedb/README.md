## Rooch Statedb tool

A tool to export and import rooch statedb.

### Usage

1. rooch statedb genesis-utxo

```shell
rooch statedb genesis-utxo --input ~/utxo.txt -d ~/.rooch -n main
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

Step 4 run statedb genesis-utxo command

```shell
rooch statedb genesis-utxo --input {your utxo file} -d {your rooch data dir} -n main
```

2. rooch statedb export

```shell
rooch statedb export --output {your file} -d {your rooch data dir} -n main -m {export mode}
```

3. rooch statedb import

```shell
rooch statedb statedb --input {your file} -d {your rooch data dir} -n main
```

### Config

For better import throughput, please increase block cache size for RocksdbConfig of moveos. 16GiB is a good start.