# Nostr In Move

Nostr in Move, a Nostr referential implementation in Move programming language for reference of on-chain persistent storage of Nostr.

## Protocol Implementation

Nostr in Move implements [NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md).

## Usage

Nostr in Move consists of base smart contracts written in Move programming language that depends on Rooch Framework's verification package for Schnorr signature. It is based on Rooch, but could be rebuilt and reused in other Move oriented blockchains.

The base smart contracts currently function:

- [Save Nostr event with verification](#save).
- [Save Nostr event without verification](#save).
- [Create Nostr event](#create).

## Prerequisites

- [Rooch command line interface](https://rooch.network/build/reference/rooch-cli).
- Knowledge of [Move on Rooch](https://rooch.network/learn/core-concepts/move-contracts/move-on-rooch).

## Install

Follow the [installation guide](https://rooch.network/build/getting-started/installation) to install Rooch command line interface.

## Build

Build the smart contracts by using a default Rooch address:

```zsh
rooch move build --named-addresses nostr=default
```

Or assign a specific one:

```zsh
rooch move build --named-addresses nostr=<contract_address>
```

## Publish

Publish the smart contracts to Rooch:

```zsh
rooch move publish ./build/nostr/package.rpd
```

## Run

### Save

Saves Nostr events into the Move event smart contract.

- Save a Nostr event to Move store with verification of id and signature
```zsh
rooch move run --function <contract_address>::event::save_event_entry --args "string:<x_only_public_key>" --args "u64:<created_at>" --args "u16:<kind>" --args "vector<string>:<tags>" --args "string:<content>" --args "string:<signature>"
```
Tags as command line arguments only accept **vector\<string\>**. For **vector\<vector\<string\>\>**, it should be supported in the future.
- Save a Nostr event to Move store without verification of id
```zsh
rooch move run --function <contract_address>::event::save_event_plaintext_entry --args "string:<id>" --args "string:<x_only_public_key>" --args "u64:<created_at>" --args "u16:<kind>" --args "vector<string>:<tags>" --args "string:<content>" --args "string:<signature>"
```
This could be met with the need of saving a draft of or unpublished Nostr note. For example, save with varying content of Nostr event in the Move store.

### Create

Creates Nostr events in the Move event smart contract and moves to the owner of the X Only public key as unpublished events.

- Create a Nostr event natively in Move and store in Move's state
1. Create a pre event of Nostr for signing
```zsh
rooch move run --function <contract_address>::event::create_pre_event_entry --args "string:<x_only_public_key>" --args "u16:<kind>" --args "vector<string>:<tags>" --args "string:<content>"
```
The pre event of Nostr is used with offline signing environment for Schnorr signature to generate a signature for a Nostr event. To view the generated id of the pre event of Nostr used for signing, run the following command:
```zsh
rooch object -i <pre_event_object_id>
```
2. Sign the id of the Nostr pre event with Schnorr signature

This step could be done with Rooch TypeScript, Rust, Python or Go SDK, or command-line interfaces that support signing sha256 hashed message with Schnorr signature.

When signing id of Nostr pre event with Schnorr signature, make sure the leading `0x` of the id is stripped.

3. Create an event of Nostr using the previously generated signature

> **Note:** to create an event of Nostr natively in Move, the following steps must be done.
> 1. Import Nostr private key in hex to Rooch account using `rooch account import` command:
> ```zsh
> rooch account import -k <nostr_private_key> --json
> ```
> The Nostr private key in hex could be retrieved from third party applications.
>
> 2. Get the Nostr public key from `rooch account list` command:
> ```zsh
> rooch account list --json
> ```
>
> 3. Switch to the Nostr public key returned by the second step using `rooch account switch` command:
> ```zsh
> rooch account switch -a <nostr_public_key> --json
> ```

Once switched to the Nostr public key on Rooch, run the following command to insert the signature into the pre event of Nostr and construct an event of Nostr:

```zsh
rooch move run --function <contract_address>::event::create_event_entry --args "string:<signature>"
```

The signer of the pre event of Nostr is also required to generate an event of Nostr under the same context to be shared with Nostr public key.

4. View the Nostr event in Move's state
```zsh
rooch object -i <event_object_id>
```

## Interoperability

Since the X Only public key used in Nostr could be converted to Bitcoin address and Rooch address, there are plenty of scenarios to consider of.

### Bitcoin

[NIP-47](https://github.com/nostr-protocol/nips/blob/master/47.md) defines a basic usage between the lightning wallet and the Nostr apps. Pull the Nostr event type of Bitcoin payments stored in the Move's state, and make use of it.

### Rooch

Rooch for Bitcoin payments could be used to persist Nostr event type of Bitcoin payments on-chain and provide necessary information to third party inquirers.

## Extendability

Developers may extend other NIP capabilities to the base smart contracts in Move programming language.

## Terms

- Nostr: notes and other stuff transmitted by relays.
- Move: a smart contract language from Diem.
- Diem: the author of the Move programming language.
- Rooch Framework: a Move library provided by Rooch.
- Schnorr signature: a signature over secp256k1 elliptic curve.
- X Only public key: a 32-byte hex strings without the leading `0x`.
- Rooch: verifiable app container with Move language for Bitcoin ecosystem.
- SDK: software development kit.
- Nostr public key: a Nostr public key in bech32 starting from `npub`.
- Bitcoin: a digital cash system and a decentralized digital currency.
- NIP: Nostr implementation possibilities.

## Related Links

- https://github.com/nostr-protocol/nips/blob/master/01.md.
- https://rooch.network/build/reference/rooch-cli.
- https://rooch.network/learn/core-concepts/move-contracts/move-on-rooch.
- https://rooch.network/build/getting-started/installation.
- https://github.com/nostr-protocol/nips/blob/master/47.md.

## License

Nostr in Move is licensed Apache-2.0 license inherited from the parent project.
