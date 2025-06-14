# Rooch WebAuthn Validator Development Guide

## 1. Overview

To enhance user experience and leverage the secure hardware of modern devices (such as Touch ID, Windows Hello, YubiKey, etc.), the Rooch framework has introduced support for WebAuthn (FIDO2). This allows users to directly use their device's built-in biometrics or security keys to sign Rooch transactions.

This guide aims to help developers understand how the WebAuthn validation process works in Rooch and to instruct them on how to correctly build and submit a transaction signed with WebAuthn on the client side (dApp/SDK).

The core of this feature is based on the **P-256 (secp256r1)** elliptic curve signature algorithm, and therefore also depends on the newly added `ecdsa_r1` crypto module in the framework.

## 2. Architecture Overview

The entire process is divided into two parts: **Client-side (off-chain)** and **Rooch Node (on-chain)**.

-   **Client**: Responsible for interacting with the browser's WebAuthn API (`navigator.credentials.get()`), obtaining the signature and related data, then packaging (serializing) this data in a specific format, and finally sending it as the transaction's authenticator payload to the Rooch node.
-   **On-chain Validator (`webauthn_validator.move`)**: A dedicated Move module responsible for parsing the payload from the client, reconstructing the standard WebAuthn verification message, calling the `ecdsa_r1` module for signature verification, performing a series of security checks, and finally confirming the signer's identity.

## 3. On-chain Implementation Details

Before diving into client-side development, it is crucial to understand the requirements of the on-chain validator.

### 3.1. Validator ID and Signature Scheme

-   **Validator ID**: The built-in ID for the WebAuthn validator in the Rooch system is `3`.
    -   `rooch_framework::webauthn_validator::WEBAUTHN_AUTH_VALIDATOR_ID: u64 = 3;`
-   **Signature Scheme ID**: The ID for the P-256 (secp256r1) signature scheme is `2`.
    -   `rooch_framework::session_key::SIGNATURE_SCHEME_ECDSAR1: u8 = 2;`

### 3.2. Data Payload Structure (WebauthnAuthPayload)

The data submitted by the client **must** be structured as the following Move struct and then serialized using **BCS (Binary Canonical Serialization)**.

```move
// in rooch_framework::webauthn_validator
struct WebauthnAuthPayload has copy, store, drop {
    scheme: u8,                   // Must be 2 (SIGNATURE_SCHEME_ECDSAR1)
    signature: vector<u8>,        // 64 bytes (r || s)
    public_key: vector<u8>,       // 33-byte compressed P-256 public key
    authenticator_data: vector<u8>, // authenticatorData from WebAuthn API
    client_data_json: vector<u8>,   // clientDataJSON from WebAuthn API
}
```

### 3.3. Core Validation Process

The `validate` function of the `webauthn_validator.move` module performs the following steps:

1.  **BCS Decode**: Deserializes the received `authenticator_payload` (a `vector<u8>`) into the `WebauthnAuthPayload` struct.
2.  **Reconstruct Verification Message**: The WebAuthn signature is not on the transaction hash itself, but on a combination of `authenticatorData` and a hash of `clientDataJSON`. The validator reconstructs this message on-chain:
    ```
    message_to_verify = authenticatorData || SHA-256(clientDataJSON)
    ```
3.  **ECDSA (P-256) Signature Verification**: Uses the `ecdsa_r1::verify` function, passing the `signature`, `public_key` parsed from the payload, and the `message_to_verify` reconstructed in the previous step to verify the signature's validity.
4.  **Challenge Verification (Critical Security Step)**:
    -   Parses `clientDataJSON` (which is a JSON string).
    -   Extracts the `challenge` field. The value of this field should be the **Base64URL-encoded transaction hash**.
    -   On-chain, the validator Base64 decodes the `challenge` string and asserts that its result must be identical to the current transaction's hash (`tx_context::tx_hash()`). This step ensures that the signature is bound to a specific transaction, preventing replay attacks.
5.  **Authentication Key Generation**:
    -   After successful verification, it calls the `session_key::secp256r1_public_key_to_authentication_key` function to generate a chain-recognizable `authentication_key` from the `public_key`.
    -   **Derivation Logic**: `auth_key = vector::singleton(scheme_byte) || hash::sha2_256(public_key)`
        -   i.e.: `[0x02, ...32_byte_hash...]`
6.  **Session Key and DID Integration Verification**:
    -   `session_key::contains_session_key`: Checks if the derived `authentication_key` has been registered as a session key under the user's account.
    -   `did::find_verification_method_by_session_key`: Checks if this `authentication_key` is associated with a verification method in the user's DID document (this verification method must belong to the `authentication` relationship).
    -   **Important**: This means the user must **first** add their WebAuthn device's public key to their DID document before they can use it to sign transactions.

## 4. Client-side (dApp/SDK) Development Guide

Based on the on-chain validation logic, the client needs to perform the following steps:

1.  **Calculate Transaction Hash**: First, just like with a regular transaction, construct the full transaction data and calculate its final hash `tx_hash` (32 bytes).

2.  **Call WebAuthn API**: Call `navigator.credentials.get()` and provide the `tx_hash` calculated in the previous step in the `publicKey.challenge` field.
    -   **Note**: The `challenge` field requires an `ArrayBuffer` type, and according to the WebAuthn specification, the raw data usually needs to be Base64URL encoded. The client library should handle the conversion from `tx_hash` (raw bytes) -> `challenge` (encoded ArrayBuffer).

    ```javascript
    const credential = await navigator.credentials.get({
      publicKey: {
        // ... other options like rpId
        challenge: arrayBufferFromTxHash, // ArrayBuffer of the 32-byte transaction hash
        allowCredentials: [/*...*/],
      },
    });
    ```

3.  **Collect Data**: Obtain the following raw byte data (`ArrayBuffer`) from the `credential` object:
    -   `signature`: `credential.response.signature`
    -   `authenticatorData`: `credential.response.authenticatorData`
    -   `clientDataJSON`: `credential.response.clientDataJSON`
    -   `publicKey`: The user's P-256 public key (usually obtained and saved during the registration phase). Must be in 33-byte compressed format.

4.  **Construct and Serialize Payload**:
    -   Assemble the above data along with `scheme: 2` into an object that corresponds to the `WebauthnAuthPayload` struct in Move.
    -   Use a BCS library (e.g., `@mysten/bcs`) to serialize this object into a byte array.

    ```javascript
    // Example using a BCS library
    const bcs = new BCS(getSuiMoveConfig()); // Assuming a compatible BCS config

    const payload = {
      scheme: 2,
      signature: new Uint8Array(signature),
      public_key: new Uint8Array(compressedPublicKey),
      authenticator_data: new Uint8Array(authenticatorData),
      client_data_json: new Uint8Array(clientDataJSON),
    };

    // This is the final payload to be used in the authenticator
    const serializedPayload = bcs.ser('WebauthnAuthPayload', payload).toBytes();
    ```
    *(Note: The specific BCS serialization requires registering the `WebauthnAuthPayload` struct type. Please refer to the Rooch SDK implementation.)*

5.  **Send Transaction**:
    -   Construct an `Authenticator` object where `auth_validator_id` is `3` and `payload` is the BCS-serialized byte array from the previous step.
    -   Attach this `Authenticator` to your transaction and send it.

## 5. Security Considerations

-   **Phishing Protection (`rpId`)**: It is strongly recommended to set the `rpId` field when calling `navigator.credentials.get()` and to keep it consistent with the `rpId` used during registration. This can prevent phishing websites from tricking users into signing malicious transactions. The on-chain validator can also optionally check the `rpIdHash` in `authenticatorData` for enhanced security.
-   **Challenge Binding**: The strict on-chain binding of `challenge` and `tx_hash` is the core mechanism to prevent signature replay attacks. The client must ensure this field is set correctly.
-   **User Verification (`uv`)**: For high-value transactions, it is recommended to set `userVerification: 'required'` in the WebAuthn options to force user biometric or PIN verification. The on-chain validator can check the `uv` flag in `authenticatorData` to enforce this policy.

## 6. Summary

Rooch's WebAuthn support provides a highly secure and user-friendly transaction signing scheme. When integrating, the key for developers is to understand and correctly implement the client-side **Payload construction and BCS serialization** process, and to ensure that the user's WebAuthn public key has been pre-registered into their Rooch DID. 