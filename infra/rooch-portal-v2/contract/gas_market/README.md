# GasMarket and GasFaucet

1. build package

```bash
rooch move build -p infra/rooch-portal-v2/contract/gas_market --named-addresses gas_market=bc1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4sugkfjt
```

2. build tx

```bash
rooch tx build --sender bc1prcajaj9n7e29u4dfp33x3hcf52yqeegspdpcd79pqu4fpr6llx4sugkfjt --function 0x2::module_store::publish_package_entry --args file:infra/rooch-portal-v2/contract/gas_market/build/gas_market/package.rpd
``` 

3. sign tx
4. submit tx