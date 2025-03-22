**A full workflow example:**

1. add fixtures/config.yml to the ROOCH_CONFIG environment variable
2. Start a local server: 
```
    rooch server start
```
3. Open another terminal, publish the `friend_module_renaming` module: 
```
    rooch move publish -p ../../examples/friend_module_renaming/ --sender-account 0x123
```
