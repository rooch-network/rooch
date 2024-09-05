## Rooch Statedb tool

A tool to export and import rooch statedb.

### Usage

#### Prerequisite for genesis-* command

1. bitcoind synced with `-txindex=1` and `-server=1` option:

```shell
bitcoind -datadir=<datadir> -txindex=1 -server=1
```

2. set block height to genesis block height:

expect height: `<h>`

get `<h+1>` block hash:

```shell
bitcoin-cli -datadir=<datadir> -conf=<datadir/bitcoin.conf> -rpccookiefile=<datadir/.cookie> getblockhash <h+1>
<h+1 block hash>
```

invalid `<h+1>` block:

```shell
bitcoin-cli -datadir=<datadir> -conf=<datadir/bitcoin.conf> -rpccookiefile=<datadir/.cookie> invalidateblock <h+1 block hash>
```

check block height is expected:

```shell
bitcoin-cli -datadir=<datadir> -conf=<datadir/bitcoin.conf> -rpccookiefile=<datadir/.cookie> getblockcount
<h>
```

3. prepare utxo source file:

> - stop bitcoind first
> - clone chainstate:

```shell
rsync --delete -av <datadir/chainstate/> <chainstate_clone_path>
```

> - dump utxo source file(each line is a utxo record, format
    is `count,txid,vout,height,coinbase,amount,script,type,address`) by
    [bitcoin-utxo-dump](https://github.com/in3rsha/bitcoin-utxo-dump):

```shell
bitcoin-utxo-dump -f count,txid,vout,height,coinbase,amount,script,type,address -db <chainstate_clone_path> -o <output>
```

> - check max height of utxo dump file is <h> by python script:

```shell
awk -F, 'NR > 1 { if ($4 > max) max = $4 } END { print max }' <utxo_list_path>
```

4. prepare ord source file(if needed):

> - start bitcoind again
> - dump ord source file by
    [ord](https://github.com/popcnt1/ord):

5. prepare genesis env:

```shell
rooch genesis init -n main -d <rooch_datadir>
```

#### Subcommands

**genesis-utxo**:

```shell
rooch statedb genesis-utxo --input <utxo_src_path> -d <rooch_datadir> -n main --batch-size <utxo_batch_size>
```

**genesis**:

```shell
rooch statedb genesis --utxo-source <utxo_src_path> --ord-source <ord_src_path> -d <rooch_datadir> -n main --outpoint-inscriptions-map-dump-path <dump_path> --utxo-batch-size <utxo_batch_size> --ord-batch-size <ord_batch_size>
```

***tips***:

> - `--utxo-ord-map` is redb database file path. We could reuse it in `genesis-utxo` command.
> - `--batch-size`/`--utxo-batch-size` is optional, default is 2M. Set it smaller if memory is limited.
> - `--ord-batch-size` is optional, default is 1M. Set it smaller if memory is limited.

**genesis-verify**:

```shell
rooch statedb genesis --utxo-source <utxo_src_path> --ord-source <ord_src_path> -d <rooch_datadir> -n main --outpoint-inscriptions-map-dump-path <dump_path>
```

**tips**:

> - `--random-mode` is optional, default is false. Set it true if you want a fast check. Highly recommend passing it if
    you have a big data set.

**rooch statedb export**:

```shell
rooch statedb export --output {your file} -d {your rooch data dir} -n main -m {export mode}
```

**rooch statedb import(WIP)**:

```shell
rooch statedb import --input {your file} -d {your rooch data dir} -n main
```
