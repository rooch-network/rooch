**A full workflow example:**

1. add fixtures/config.yml to the ROOCH_CONFIG environment variable
2. Start a local server: 
```
    rooch server start
```
3. Open another terminal, publish the `cosmwasm_vm_execution` module: 
```
    rooch move publish -p ../../examples/cosmwasm_vm_execution/ --sender-account 0x123
```
4. Run `0x123::cosmwasm_vm_execution::run` to execute a wasm function
```
    rooch move run --function 0x123::cosmwasm_vm_execution::run --sender-account 0x123
```
