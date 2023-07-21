## NoOp Auth Validator

NoOpValidator is an auth validator that does not validate anything. It is used for testing purposes, and should not be used in production.


1. Start a local server: 
```bash
rooch server start
```
2. Open another terminal, publish the `noop_auth_validator` modules: 

```bash
rooch move publish -p ./examples/noop_auth_validator --sender-account default --named-addresses noop_auth_validator=default

```
The module is published to my default address, the address is `0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01`.

3. Run `0x3::account_authentication::install_auth_validator_entry` to install the auth validator to my default account: 
```bash
rooch move run --function 0x3::account_authentication::install_auth_validator_entry --type-args 0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01::noop_validator::NoOpValidator --sender-account default
```

4. Run the `state` command to view the `InstalledAuthValidator`` resource:
```bash
rooch state --access-path /resource/0xbbfc33692c7d57839fde9643681fb64c83b377e4c70b1e4b76aa35ff1e410d01/0x3::account_authentication::InstalledAuthValidator
```
```json
[
  {
    "state": {
      "value": "0x010300000000000000",
      "value_type": "0x3::account_authentication::InstalledAuthValidator"
    },
    "move_value": {
      "abilities": 8,
      "type": "0x3::account_authentication::InstalledAuthValidator",
      "value": {
        "validators": [
          "3"
        ]
      }
    }
  }
]
```
The validateor id is `3`, it also is the authenticator's scheme.

5. Use the new authenticator to run a function:

```bash
rooch move run --function 0x3::empty::empty --authenticator 3:0x12 --sender-account default
```

we can see the transaction is executed successfully.

The authenticator is `3:0x12`, the first part is the scheme, the second part is the payload in hex string.

The `NoOpValidator` accepts any payload, only fails when the payload is empty. 

The following command will fail:

```bash
rooch move run --function 0x3::empty::empty --authenticator 3: --sender-account default
```