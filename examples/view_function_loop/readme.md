**A full workflow example:**

1. add fixtures/config.yml to the ROOCH_CONFIG environment variable
2. Start a local server: 
```
    rooch server start
```
3. Open another terminal, publish the `view_function_loop` module: 
```
    rooch move publish -p ../../examples/view_function_loop/ --sender-account 0x123
```
4. Run `0x123::view_function_loop::out_of_gas` to execute a view function
```
    rooch move view --function 0x123::view_function_loop::out_of_gas 
```
