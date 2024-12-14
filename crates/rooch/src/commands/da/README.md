# Rooch DA tool

Toolkits for RoochDA.

## Usage

### namespace

Derive DA namespace from genesis file:

```shell
rooch da namespace --genesis-file-path {genesis-file}
```

### unpack

Unpack tx list to human-readable format:

```shell
rooch da unpack --segment-dir {segment-dir} --batch-dir {batch-dir}
```

If you want to verify tx list order, you can use `--verify-order` flag:

```shell
rooch da unpack --segment-dir {segment-dir} --batch-dir {batch-dir} --verify-order
```

### exec

Execute tx list with state root verification(compare with Rooch Network).
It's a tool built for verification at development stage, not a full feature tool for sync states in production.

#### Prepare tx_order:state_root list file

using [order_state.sh](https://github.com/popcnt1/roh/blob/main/scripts/order_state.sh) to generate tx_order:state_root
list file:

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

genesis init(add correct genesis back into db):

```shell
rooch genesis init -d {data_dir} -n {network}
```