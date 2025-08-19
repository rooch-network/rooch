# Rooch Move Framework v23

- [rooch-framework] Introduce payment channel modules and docs; implement DID-based channel flows, SubRAV, CLI integration, and tests (#3648)
- [rooch-framework] Introduce ecdsa_r1 crypto and WebAuthn validator; refactor multibase and DID; update docs, natives, and tests (#3620)
- [rooch-framework] Introduce rs256 crypto primitive with natives, docs, and test cases; add additional checks for pre-hash function (#3624)
- [rooch-framework] Re-add Schnorr (secp256k1) support and merge with ecdsa_k1; adjust gas parameters; add docs (#3550)
- [rooch-framework] DID session scopes: support parsing custom session scopes; add `add_verification_method_with_scopes_entry`; update docs and tests (#3655)
- [rooch-framework] DID on Move: foundational DID changes and tests (#3595)
- [moveos_std] Add `address::from_string`; update `string_utils` docs used by session scopes (#3655)
- [rooch-framework] Fix `ecdsa_r1` debug_assert; initialize genesis gas v6 parameters (#3625)
- [rooch-framework] Coin migration: add migration module and docs; scripts for coin store migration; change init function to manual execute (#3572, #3573, #3568)
- [frameworks] Minor typo fixes across stdlib and framework sources (#3603, #3663)
 - [rooch-framework] Transfer enhancements and testnet switch: support batch transfer of objects; update transfer docs and core modules; switch testnet to Bitcoin testnet4; regenerate testnet genesis (#3679)
