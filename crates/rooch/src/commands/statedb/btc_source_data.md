## BTC Source Data

This doc shows how Rooch gets BTC source data(utxo/inscriptions)

### Sync Bitcoin server at certain height

#### bitcoind synced with `-txindex=1` and `-server=1` option:

```shell
bitcoind -datadir=<datadir> -txindex=1 -server=1
```

#### set block height to genesis block height:

expect height: `<h>` (for Rooch Mainnet, `<h>` is `859000`)

```shell

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

### Dump UTXO source file:

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

### Dump Inscription source file:

[ord tool](https://github.com/popcnt1/ord/tree/feat/rooch/export) is used to dump inscriptions source file.

> - start bitcoind again
> - mapping inscription:address by `ord index map-addr`
> - dump inscriptions by `ord index rooch`

More details could be found [here](https://github.com/popcnt1/ord/tree/feat/rooch/export/src/subcommand/index)
