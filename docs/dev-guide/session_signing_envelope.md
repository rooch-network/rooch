## Session Signing Envelope for Bitcoin Wallet Compatibility

This document specifies a backward-compatible extension to Rooch session authentication that decouples the signature algorithm from the message formatting ("signing envelope"). It enables Bitcoin wallets that only support message signing (e.g., UniSat, OKX) to sign Rooch transactions without changing the underlying signature algorithm (e.g., Secp256k1).

## Goals

- Preserve existing session signature schemes: Ed25519, Secp256k1, Secp256r1
- Remain backward-compatible with existing authenticator payloads
- Introduce an extensible, versioned "signing envelope" that describes how the message to be signed is constructed from the Rooch transaction hash
- Provide a secure, replay-resistant template that binds the signature to a specific Rooch transaction (and optionally to a chain/domain)

## Terminology

- **Signature Scheme**: The asymmetric signature algorithm and key type. Existing values: Ed25519, Secp256k1 (K1), Secp256r1 (R1)
- **Signing Envelope**: A small, explicit descriptor for how the message is derived from `tx_hash` before signature verification

## Envelope Types

- **0x00 RawTxHash (default)**: The signature is verified directly over `tx_hash`
- **0x01 BitcoinMessageV0**: The signature is verified over the standard Bitcoin message digest:
  - `digest = SHA256(SHA256("Bitcoin Signed Message:\n" + VarInt(len(message)) + message))`
  - `message` is an ASCII string carried in the authenticator payload and must equal a canonical template built from the current `tx_hash` (see Template Binding)
- **0x02 WebAuthnV0**: The signature is verified over the WebAuthn message digest:
  - `digest = authenticator_data || SHA256(client_data_json)`
  - `message` contains a BCS-encoded `WebauthnAuthPayload` structure
  - The `challenge` field in `client_data_json` must equal the base64-encoded `tx_hash`
- **Reserved for future extensions**:
  - 0x03 Bip322Simple
  - 0x10.. Vendor-specific envelopes

Unknown envelope values MUST be rejected.

## Authenticator Payload Formats

The legacy v1 payload remains supported for strict backward compatibility. A new v2 payload introduces a one-byte `envelope` after the existing `scheme`.

### v1 (Legacy, implicit RawTxHash)

```
| scheme (1) | signature | public_key |
```

- **Ed25519**: `signature(64) | public_key(32)`
- **Secp256k1**: `signature(64) | public_key(33)` (compressed)
- **Secp256r1**: `signature(64) | public_key(33)`
- Envelope is implicitly `0x00 RawTxHash`

### v2 (Envelope-aware)

```
| scheme (1) | envelope (1) | signature | public_key | [message_len | message] |
```

- For envelopes that do not require external data (e.g., RawTxHash), `message_len | message` are omitted
- For `BitcoinMessageV0 (0x01)`, `message` MUST be included
  - `message_len`: either VarInt (Bitcoin style) or a fixed-width length (e.g., u32). SDK and on-chain validator MUST agree on one encoding. Recommendation: use VarInt for wire compatibility; on-chain can initially only implement the single-byte path because expected messages are shorter than 253 bytes

## Template Binding (Anti-Replay)

To prevent replay and phishing, the `message` for `BitcoinMessageV0` MUST be equal to a canonical template derived from the transaction under verification:

```
message = "Rooch Transaction:\n" + hex_lowercase(tx_hash)
```

This format is consistent with the existing `auth_payload.move` module to maintain compatibility with Bitcoin authenticators.

- `hex_lowercase(tx_hash)` is the lowercase hex encoding of the 32-byte Rooch transaction hash
- The template MUST be exactly reproduced on-chain for equality comparison; messages deviating from the template MUST be rejected

## On-chain Verification Flow

Applies to `frameworks/rooch-framework/sources/auth_validator/session_validator.move`.

1. Parse `scheme`
2. If the payload is v2, parse `envelope`; if missing, set `envelope = 0x00 RawTxHash`
3. Extract `(signature, public_key)` and optional `(message)` according to `scheme` and `envelope`
4. Compute `digest` based on `envelope`:
  - RawTxHash (0x00): `digest = tx_hash`
  - BitcoinMessageV0 (0x01):
    - Ensure `message` is present and equals the canonical template derived from `tx_hash` (and optional `ChainId`)
    - Compute `digest = sha256(sha256("Bitcoin Signed Message:\n" + VarInt(len(message)) + message))`
  - WebAuthnV0 (0x02):
    - Ensure `message` is present and contains a valid BCS-encoded `WebauthnAuthPayload`
    - Deserialize the payload to extract `authenticator_data` and `client_data_json`
    - Verify that the `challenge` in `client_data_json` equals the base64-encoded `tx_hash`
    - Compute `digest = authenticator_data || SHA256(client_data_json)`
5. Verify signature using the `scheme`-appropriate verifier:
   - Ed25519: `ed25519::verify(signature, public_key, digest)`
   - Secp256k1: `ecdsa_k1::verify(signature, public_key, digest, ecdsa_k1::sha256())`
   - Secp256r1: `ecdsa_r1::verify(signature, public_key, digest)`
6. Derive the session authentication key from the `public_key` as done today

Unknown or unsupported combinations MUST abort with `error_validate_invalid_authenticator()`.

### Implementation Notes (Move)

- Hex encoding: implement a simple nibble-to-ASCII function to produce `hex_lowercase(tx_hash)` deterministically
- VarInt: initially support the single-byte branch (`len(message) < 253`) since the canonical template length is small; extend to full VarInt if/when needed
- Ensure existing parsing loops and length checks are kept strict to avoid out-of-bounds reads

## SDK Responsibilities

- Introduce a `SigningEnvelope` enum mirroring on-chain values
  - Default: `RawTxHash`
  - For Bitcoin wallets that only support `signMessage`, select `BitcoinMessageV0`
- Construct the canonical template from `tx_hash` (and optional `chainId`)
- When using `BitcoinMessageV0`:
  - Call the wallet `signMessage(message)` API
  - Assemble v2 payload: `scheme | envelope | signature | public_key | message_len | message`
- When possible, prefer `RawTxHash` for wallets that can sign raw digests

## Backward Compatibility

- Existing clients using v1 payloads continue to work unchanged (implicit `RawTxHash`)
- New clients can opt into v2 payloads with an explicit `envelope`
- Validators MUST accept both v1 and v2 payloads

## Security Considerations

- Strong binding to `tx_hash` via the canonical template prevents cross-transaction replay
- Optional chain/domain separation strengthens cross-chain replay resistance
- Reject unknown envelopes and malformed payloads
- Keep message ASCII and bounded length; perform strict length checks
- Rely on existing verifiers to enforce signature canonicalization (e.g., low-S for ECDSA) where applicable

## Testing

- Unit tests in Move (see `make test-move`):
  - Legacy v1 success/failure paths remain unchanged
  - v2 `BitcoinMessageV0`:
    - Success with correct template and signature
    - Failure when `message` mismatches template
    - Failure for unknown `envelope`
    - Failure for length inconsistencies
- SDK tests:
  - Envelope selection logic
  - Payload assembly/round-trip

## Future Extensions

- BIP-322 envelope(s) for more advanced Bitcoin wallet flows
- Additional domain separators (e.g., network name, epoch) if required
- Support for alternative message templates for specialized flows, guarded by explicit envelope values

## References

- Validator: `frameworks/rooch-framework/sources/auth_validator/session_validator.move`
- Developer guide: `docs/dev-guide/rooch_move_guide.md`
