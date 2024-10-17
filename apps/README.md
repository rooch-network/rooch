# Rooch DAO Apps

Package address: 0x701c21bf1c8cd5af8c42983890d8ca55e7a820171b8e744c13f2d9998bf76cc3

* app_admin
* gas_faucet
* gas_market
* grow_bitcoin

### Publish packages

```bash
rooch move publish -p apps/app_admin --named-addresses app_admin=default
rooch move publish -p apps/gas_faucet --named-addresses app_admin=default,gas_faucet=default
rooch move publish -p apps/gas_market --named-addresses app_admin=default,gas_market=default
rooch move publish -p apps/grow_bitcoin --named-addresses app_admin=default,grow_bitcoin=default
```
