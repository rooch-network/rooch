// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Bytes } from '../types/index.js'
import { bytes, sha256, toHEX, concatBytes, varintByteNum, fromB64 } from '../utils/index.js'
import { bcs } from '../bcs/index.js'

/**
 * Session signing envelope types
 */
export enum SigningEnvelope {
  RawTxHash = 0x00,
  BitcoinMessageV0 = 0x01,
  WebAuthnV0 = 0x02,
}

/**
 * Base interface for envelope message builders
 */
export interface EnvelopeMessageBuilder {
  buildMessage(txHash: Bytes): Bytes
  getEnvelopeType(): SigningEnvelope
}

/**
 * Raw transaction hash envelope (default)
 * No message transformation, signs directly over tx_hash
 */
export class RawTxHashEnvelope implements EnvelopeMessageBuilder {
  buildMessage(txHash: Bytes): Bytes {
    return txHash
  }

  getEnvelopeType(): SigningEnvelope {
    return SigningEnvelope.RawTxHash
  }
}

/**
 * Bitcoin message envelope
 * Signs over Bitcoin message format: "Bitcoin Signed Message:\n" + message
 */
export class BitcoinMessageEnvelope implements EnvelopeMessageBuilder {
  private readonly messagePrefix = '\u0018Bitcoin Signed Message:\n'
  private readonly roochPrefix = 'Rooch Transaction:\n'

  buildMessage(txHash: Bytes): Bytes {
    // Build canonical template: "Rooch Transaction:\n" + hex(tx_hash)
    const template = this.roochPrefix + toHEX(txHash)
    return bytes('utf8', template)
  }

  /**
   * Build the full Bitcoin message for signing
   */
  buildBitcoinMessage(txHash: Bytes): Bytes {
    const message = this.buildMessage(txHash)
    const messageLength = message.length

    // Bitcoin message format: prefix + varint(len) + message
    const prefixBytes = bytes('utf8', this.messagePrefix)
    const lengthBytes = varintByteNum(messageLength)

    return concatBytes(prefixBytes, lengthBytes, message)
  }

  /**
   * Compute Bitcoin message digest (double SHA256)
   */
  computeDigest(txHash: Bytes): Bytes {
    const bitcoinMessage = this.buildBitcoinMessage(txHash)
    return sha256(sha256(bitcoinMessage))
  }

  getEnvelopeType(): SigningEnvelope {
    return SigningEnvelope.BitcoinMessageV0
  }
}

export class WebauthnEnvelopeData {
  authenticator_data: Uint8Array
  client_data_json: Uint8Array

  constructor(authenticator_data: Uint8Array, client_data_json: Uint8Array) {
    this.authenticator_data = authenticator_data
    this.client_data_json = client_data_json
  }

  encode(): Bytes {
    return WebauthnEnvelopeDataSchema.serialize({
      authenticator_data: this.authenticator_data,
      client_data_json: this.client_data_json,
    }).toBytes()
  }
}

export const WebauthnEnvelopeDataSchema = bcs.struct('WebauthnEnvelopeData', {
  authenticator_data: bcs.vector(bcs.u8()),
  client_data_json: bcs.vector(bcs.u8()),
})

/**
 * WebAuthn envelope
 * Signs over WebAuthn message format: authenticator_data || SHA256(client_data_json)
 */
export class WebAuthnEnvelope implements EnvelopeMessageBuilder {
  private readonly authenticatorData: Bytes
  private readonly clientDataJson: Bytes

  constructor(authenticatorData: Bytes, clientDataJson: Bytes) {
    this.authenticatorData = authenticatorData
    this.clientDataJson = clientDataJson
  }

  buildMessage(_txHash: Bytes): Bytes {
    // For WebAuthn, we return the BCS-encoded WebauthnAuthPayload
    // This will be handled specially in the authenticator
    throw new Error('WebAuthn envelope requires special handling in authenticator')
  }

  /**
   * Compute WebAuthn digest: authenticator_data || SHA256(client_data_json)
   */
  computeDigest(_txHash: Bytes): Bytes {
    const clientDataHash = sha256(this.clientDataJson)
    return concatBytes(this.authenticatorData, clientDataHash)
  }

  getEnvelopeType(): SigningEnvelope {
    return SigningEnvelope.WebAuthnV0
  }

  getAuthenticatorData(): Bytes {
    return this.authenticatorData
  }

  getClientDataJson(): Bytes {
    return this.clientDataJson
  }
}

/**
 * WebAuthn assertion data parsed from AuthenticatorAssertionResponse
 */
export interface WebAuthnAssertionData {
  signature: Bytes // Original DER signature
  rawSignature: Bytes // Canonicalized raw signature
  authenticatorData: Bytes
  clientDataJSON: Bytes
  credentialId?: string
}

/**
 * WebAuthn utilities for parsing and validating WebAuthn data
 */
export class WebAuthnUtils {
  /**
   * Parse AuthenticatorAssertionResponse into standardized data
   */
  static parseAssertionResponse(response: AuthenticatorAssertionResponse): WebAuthnAssertionData {
    const signature = new Uint8Array(response.signature)
    const authenticatorData = new Uint8Array(response.authenticatorData)
    const clientDataJSON = new Uint8Array(response.clientDataJSON)

    return {
      signature,
      rawSignature: this.derToRaw(signature),
      authenticatorData,
      clientDataJSON,
    }
  }

  /**
   * Validate that the challenge in clientDataJSON matches the expected value
   */
  static validateChallenge(clientDataJSON: Bytes, expectedChallenge: Bytes): boolean {
    try {
      const clientData = JSON.parse(new TextDecoder().decode(clientDataJSON))
      const actualChallenge = fromB64(clientData.challenge)
      // Compare byte arrays manually
      if (actualChallenge.length !== expectedChallenge.length) {
        return false
      }
      return actualChallenge.every((byte, index) => byte === expectedChallenge[index])
    } catch (error) {
      return false
    }
  }

  /**
   * Compute WebAuthn verification message: authenticator_data || SHA256(client_data_json)
   */
  static computeVerificationMessage(authenticatorData: Bytes, clientDataJSON: Bytes): Bytes {
    const clientDataHash = sha256(clientDataJSON)
    return concatBytes(authenticatorData, clientDataHash)
  }

  /**
   * Convert DER signature to raw format and canonicalize (low-S form)
   */
  private static derToRaw(der: Bytes): Bytes {
    // Parse DER sequence: 0x30 len 0x02 lenR R 0x02 lenS S
    let offset = 0
    if (der[offset++] !== 0x30) throw new Error('Invalid DER signature')

    const seqLen = der[offset++]
    if (seqLen + 2 !== der.length) {
      // For signatures longer than 127 bytes, length encoding could be multi-byte
      // but for ECDSA signatures this is typically not the case
    }

    if (der[offset++] !== 0x02) throw new Error('Invalid DER signature')
    const rLen = der[offset++]
    let r = der.slice(offset, offset + rLen)
    offset += rLen

    if (der[offset++] !== 0x02) throw new Error('Invalid DER signature')
    const sLen = der[offset++]
    let s = der.slice(offset, offset + sLen)

    // Strip leading zero padding
    if (r.length === 33 && r[0] === 0x00) {
      r = r.slice(1)
    }
    if (s.length === 33 && s[0] === 0x00) {
      s = s.slice(1)
    }

    // Canonicalize S to low-S form (for secp256r1)
    const SECP256R1_N = BigInt('0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551')
    const HALF_N = SECP256R1_N >> BigInt(1)

    let sBig = BigInt(
      '0x' +
        Array.from(s)
          .map((b) => b.toString(16).padStart(2, '0'))
          .join(''),
    )
    if (sBig > HALF_N) {
      sBig = SECP256R1_N - sBig
      // Convert back to bytes
      let sHex = sBig.toString(16)
      if (sHex.length % 2 === 1) sHex = '0' + sHex
      s = new Uint8Array(sHex.match(/.{1,2}/g)!.map((byte) => parseInt(byte, 16)))
    }

    // Pad to 32 bytes
    const rPad = new Uint8Array(32)
    rPad.set(r, 32 - r.length)
    const sPad = new Uint8Array(32)
    sPad.set(s, 32 - s.length)

    const raw = new Uint8Array(64)
    raw.set(rPad, 0)
    raw.set(sPad, 32)
    return raw
  }
}

/**
 * WebAuthn envelope builder - constructs envelope from parsed assertion data
 */
export class WebAuthnEnvelopeBuilder implements EnvelopeMessageBuilder {
  constructor(private assertionData: WebAuthnAssertionData) {}

  getEnvelopeType(): SigningEnvelope {
    return SigningEnvelope.WebAuthnV0
  }

  buildMessage(txHash: Bytes): Bytes {
    // Validate that the challenge matches the transaction hash
    if (!WebAuthnUtils.validateChallenge(this.assertionData.clientDataJSON, txHash)) {
      throw new Error('WebAuthn challenge does not match transaction hash')
    }

    // Return BCS-encoded envelope data
    return new WebauthnEnvelopeData(
      this.assertionData.authenticatorData,
      this.assertionData.clientDataJSON,
    ).encode()
  }

  computeDigest(_txHash: Bytes): Bytes {
    // This is used for verification - compute the message that was actually signed
    return WebAuthnUtils.computeVerificationMessage(
      this.assertionData.authenticatorData,
      this.assertionData.clientDataJSON,
    )
  }

  /**
   * Get the signature from the assertion data
   */
  getSignature(): Bytes {
    return this.assertionData.rawSignature
  }

  /**
   * Get the original assertion data
   */
  getAssertionData(): WebAuthnAssertionData {
    return this.assertionData
  }
}
