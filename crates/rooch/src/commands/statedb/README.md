## Rooch Statedb tool

A tool to export and import rooch statedb. 

### Usage

1. rooch  statedb genesis-utxo
```shell
rooch statedb genesis-utxo --input ~/utxo.txt -d ~/.rooch -n local
```

Step 1, cleanup database files
```shell
rooch server clean -n local
```

Step 2, start server to initialization genesis
```shell
rooch server start -n local
```

Step 3, stop server
```shell
kill {server pid} or Ctrl-C
```

Step 4 run statedb genesis-utxo command
```shell
rooch statedb genesis-utxo --input {utxo.file} -d {rooch.database.file} -n {rooch.network}
```

2. rooch  statedb export
```shell
todo!
```

3. rooch  statedb import
```shell
todo!
```