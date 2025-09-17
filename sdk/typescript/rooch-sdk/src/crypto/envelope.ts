// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Bytes } from '../types/index.js'
import { bytes, sha256, toHEX, concatBytes, varintByteNum } from '../utils/index.js'
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
