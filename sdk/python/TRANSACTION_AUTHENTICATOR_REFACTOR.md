# Python SDK Transaction Authenticator Refactor Plan

## Background
The current Python SDK implementation of transaction authenticators and their BCS serialization is not fully compatible with the Rust implementation. The main issues are:
- The Python `TransactionAuthenticator` conflates the authenticator validator ID and the signature type.
- The BCS serialization format does not match Rust: Rust expects `[auth_validator_id: u64][payload: Vec<u8>]`, where the payload's structure depends on the validator type.
- The payload for each authenticator type (Session, Bitcoin, etc.) is not handled distinctly in Python.

## Rust Reference
- `Authenticator` struct:
  - `auth_validator_id: u64` (validator type, e.g., Session, Bitcoin, etc.)
  - `payload: Vec<u8>` (the authenticator payload, format depends on validator type)
- Each authenticator type (Session, Bitcoin, BitcoinMultisign) has its own payload structure.
- Serialization: BCS encodes `auth_validator_id` as u64, then the payload as bytes (already BCS-encoded if needed).

## Python Refactor Plan

### 1. Redefine TransactionAuthenticator
- Add `auth_validator_id: int` (u64) field.
- Add `payload: bytes` field (or a serializable object, depending on type).
- Remove direct use of `AuthenticatorType` as the validator ID.
- Implement classmethods/factories for each authenticator type (Session, Bitcoin, etc.) to construct the correct payload and set the correct validator ID.

### 2. Serialization/Deserialization
- Serialization: Write `auth_validator_id` as u64, then the payload as bytes.
- Deserialization: Read `auth_validator_id`, then parse the payload according to the validator type.
- For SessionAuthenticator, the payload is just the signature bytes.
- For BitcoinAuthenticator, the payload is a BCS-encoded struct (see Rust's `AuthPayload`).

### 3. Transaction Signing Logic
- When signing, select the correct authenticator type based on the key type.
- Use the correct factory/classmethod to create the authenticator with the right payload and validator ID.

### 4. Update Tests
- Update/fix tests in `test_transaction_bcs.py` and related files to use the new authenticator structure and serialization.
- Ensure round-trip BCS serialization/deserialization matches Rust.

### 5. Migration/Compatibility
- Remove or deprecate the old `auth_type` and `AuthenticationKey` fields from transaction serialization.
- Update all usages of `TransactionAuthenticator` in the SDK and tests.

## Example (SessionAuthenticator)
```python
class TransactionAuthenticator(Serializable, Deserializable):
    def __init__(self, auth_validator_id: int, payload: bytes):
        self.auth_validator_id = auth_validator_id
        self.payload = payload

    @classmethod
    def session(cls, signature: bytes):
        return cls(auth_validator_id=SESSION_ID, payload=signature)

    def serialize(self, serializer: BcsSerializer):
        serializer.u64(self.auth_validator_id)
        serializer.bytes(self.payload)
```

## Next Steps
- Refactor the Python SDK as described above.
- Update all transaction signing and serialization logic.
- Fix and validate all related tests.
