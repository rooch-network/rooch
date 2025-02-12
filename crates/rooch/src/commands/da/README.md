# Rooch DA tool

Toolkits for RoochDA.

## Usage

### namespace

Derive DA namespace from genesis file:

```shell
rooch da namespace --genesis-file-path {genesis-file}
```

### index

Index tx_order:tx_hash:l2_block_number

basic usage:

```shell
rooch da index --segment-dir {segment-dir} -i {index-dir}
```

more options can be found by `rooch da index --help`.

### unpack

#### download segments

the easiest way to download segments is using `getda` tool from [here](https://github.com/popcnt1/roh) for downloading
segments from cloud storage.

```shell
getda --output={segment-dir} --url={da-cloud-storage-path} --last_chunk={max-chunk-id-expected}
```

#### unpack segments

Unpack tx list from segments to human-readable format:

```shell
rooch da unpack --segment-dir {segment-dir} --batch-dir {batch-dir}
```

If you want to get stats only, you can use `--stats-only` flag:

```shell
rooch da unpack --segment-dir {segment-dir} --batch-dir {batch-dir} --stats-only
```

### exec

TODO: update this section with new changes, DO NOT follow this section now.

Execute tx list with state root verification (compare with Rooch Network Mainnet/Testnet).
It's a tool built for verification at the development stage, not a full feature tool for sync states in production.

Features include:

1. Execute tx list from segment dir and saving changes locally
2. Compare state root with Rooch Network Mainnet/Testnet by tx_order:state_root list file
3. We could collect performance data in the execution process by tuning tools like `perf` if needed

#### Prepare tx_order:state_root list file

using [order_state.sh](https://github.com/popcnt1/roh/blob/main/scripts/order_state.sh) to generate tx_order:state_root:
accumulator_root list file:

```shell
rooch env switch -n {network}
./order_state.sh {start_order} {end_order} {interval}
```

#### Prepare genesis

if you just import data by `rooch statedb genesis`, and will execute transaction from tx_order 1 (genesis tx is tx_order
0).

For ensuring everything works as expected, you should:

clean dirty genesis states:

```shell
rooch statedb re-genesis -d {data_dir} -n {network} --mode remove
```

genesis init(add builtin genesis back into db):

```shell
rooch genesis init -d {data_dir} -n {network}
```

we assume it's builtin genesis, because the target we want to verify is Rooch Network Mainnet/Testnet, all the two
Network are using builtin genesis.

#### Execute tx list

```shell
rooch da exec --segment-dir {segment-dir} --order-state-path {order-state-path} -d {data-dir} -n {network} --btc-rpc-url {btc-rpc-url} --btc-rpc-user-name {btc-rpc-user-name} --btc-rpc-password {btc-rpc-password}
```

If everything is ok, you will see this log in the end:

```shell
2024-12-16T05:48:26.924094Z  INFO rooch::commands::da::commands::exec: All transactions execution state root are strictly equal to RoochNetwork: [0, {end_order}]
```