## Rooch DB tool

A tool to repair DB data.

### Usage

1. Revert tx when only write TransactionSequenceInfo succ:

```shell
rooch db revert-tx  --tx-order {tx_order}   -d {data_dir} -n {network}
```