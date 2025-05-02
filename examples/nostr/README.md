# Nostr Move Example

A Nostr (Notes and Other Stuff Transmitted by Relays) example written in Move programming language for reference of on-chain persistant storage.

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

- Save a Nostr Event to Move store with verification of id and signature:
```zsh
rooch move run --function <contract_address>::event::save_event_entry --args "string:<public_key>" --args "u64:<created_at>" --args "u16:<kind>" --args "vector<string>:<tags>" --args 'string:<content>' --args "string:<signature>"
```
Tags as command line arguments only accept **vector\<string\>**. For **vector\<vector\<string\>\>**, it should be supported in the future.
- Save a Nostr Event to Move store without verification of id:
```zsh
rooch move run --function <contract_address>::event::save_event_plaintext_entry --args "string:<id>" --args "string:<public_key>" --args "u64:<created_at>" --args "u16:<kind>" --args "vector<string>:<tags>" --args 'string:<content>' --args "string:<signature>"
```
This could be met with the need of saving a draft of or unpublished Nostr Note. For example, save with varying content of Nostr Event in the Move store.
- Create a Pre Event of Nostr for signing:
```zsh
rooch move run --function <contract_address>::event::create_pre_event_entry --args "string:<public_key>" --args "u16:<kind>" --args "vector<string>:<tags>" --args 'string:<content>'
```
The pre event of Nostr is used with schnorr offline signing environment to generate a signature for a Nostr Event. To view the generated id of the Nostr Event used for signing, run the following command:
```zsh
rooch object -i <pre_event_object_id>
```
- Sign the id of the Nostr Event with schnorr



## Troubleshooting



## Related Links

- https://github.com/nostr-protocol/nips/blob/master/01.md

## License

This example is licensed public domain and Apache-2.0 license inherited from the parent project.
