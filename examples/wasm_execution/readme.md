**A full workflow example:**

1. add fixtures/config.yml to the ROOCH_CONFIG environment variable
2. Start a local server: 
```
    rooch server start
```
3. Open another terminal, publish the `Counter` module: 
```
    rooch move publish -p ../../examples/counter/ --sender-account 0x123
```
4. Run `0x123::counter::init` to init a `Counter` resource: 
```
    rooch move run --function 0x123::counter::init --sender-account 0x123
```
5. Run the view function `0x123::counter::value`, you will see the output value:
```
    rooch move view --function 0x123::counter::value
    
    [Number(0)]
```
6. Run `0x123::counter::increase` to increase the value: 
```
    rooch move run --function 0x123::counter::increase --sender-account 0x123
```
7. Run view function again, you will see the value increased:
```
    rooch move view --function 0x123::counter::value
    [Number(1)]
```
8. View the object data of ObjectID `0x123`:
```
    rooch object --id 0x123

    Some("RawObject { id: ObjectID(0000000000000000000000000000000000000000000000000000000000000123), owner: 0000000000000000000000000000000000000000000000000000000000000123, value: [144, 120, 228, 155, 92, 27, 15, 93, 80, 134, 62, 134, 236, 51, 180, 120, 225, 111, 149, 125, 180, 108, 254, 148, 172, 217, 252, 190, 12, 87, 45, 125, 181, 196, 103, 186, 252, 91, 195, 39, 22, 109, 50, 62, 216, 114, 199, 183, 54, 56, 99, 170, 138, 171, 237, 144, 214, 105, 58, 76, 189, 250, 204, 252] }")
```
9. View the resource `0x123::counter::Counter`:
```
    rooch resource --address 0x123 --resource 0x123::counter::Counter
    
    Some("store key 0x123::counter::Counter {\n    value: 1\n}")
```