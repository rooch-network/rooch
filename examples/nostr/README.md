# Nostr Move Example

A Nostr (Notes and Other Stuff Transmitted by Relays) example written in Move programming language for reference of on-chain persistant storage for Nostr.

## Protocol Implementation

The Nostr in Move example implements [NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md).

## Build

```zsh
rooch move build --named-addresses nostr=default
```

## Publish

```zsh
rooch move publish ./build/nostr/package.rpd
```

## Run

- Save a Nostr Event to Move store with verification of id and signature
```zsh
rooch move run --function <contract_address>::event::save_event_entry --args "string:<public_key>" --args "u64:<created_at>" --args "u16:<kind>" --args "vector<string>:<tags>" --args "string:<content>" --args "string:<signature>"
```
Tags as command line arguments only accept **vector\<string\>**. For **vector\<vector\<string\>\>**, it should be supported in the future.
- Save a Nostr Event to Move store without verification of id
```zsh
rooch move run --function <contract_address>::event::save_event_plaintext_entry --args "string:<id>" --args "string:<public_key>" --args "u64:<created_at>" --args "u16:<kind>" --args "vector<string>:<tags>" --args "string:<content>" --args "string:<signature>"
```
This could be met with the need of saving a draft of or unpublished Nostr Note. For example, save with varying content of Nostr Event in the Move store.
- Create a Nostr Event natively in Move and store in Move's state
1. Create a Pre Event of Nostr for signing
```zsh
rooch move run --function <contract_address>::event::create_pre_event_entry --args "string:<public_key>" --args "u16:<kind>" --args "vector<string>:<tags>" --args "string:<content>"
```
The Pre Event of Nostr is used with schnorr offline signing environment to generate a signature for a Nostr Event. To view the generated id of the Pre Event of Nostr used for signing, run the following command:
```zsh
rooch object -i <pre_event_object_id>
```
2. Sign the id of the Nostr Pre Event with schnorr signature

This step could be done with Rooch TypeScript, Rust or Go SDK, or with `rooch account sign` command.

Firstly, import Nostr private key hex to Rooch account using `rooch account import` command:
```zsh
rooch account import -k <nostr_private_key> --json
```

Secondly, switch to the Rooch address returned by the first step:
```zsh
rooch account switch -a <rooch_address> --json
```

Lastly, sign the Nostr Pre Event id with Rooch SDKs or using `rooch account sign`:
```zsh
rooch account sign -a <nostr_public_key> -m <id> --json
```
Here, **nostr_public_key** should start with `npub` and **id** should align with the id of the Pre Event of Nostr without the leading `0x`.

3. Create an Event of Nostr using the previously generated signature
```zsh
rooch move run --function <contract_address>::event::create_event_entry --args "string:<public_key>" --args "string:<signature>"
```
The public key is required to derive the owner's rooch address for generating an Event of Nostr.

4. View the Nostr Event in Move's state
```zsh
rooch object -i <event_object_id>
```

## Interoperate Nostr with Bitcoin and Rooch

Since the Nostr public key could be converted to Bitcoin address and Rooch address, there are plenty of scenarios to consider of.

## Related Links

- https://github.com/nostr-protocol/nips/blob/master/01.md

## License

This example is licensed public domain and Apache-2.0 license inherited from the parent project.
