// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, beforeEach } from 'vitest'
import { BitcoinSignMessage, Authenticator } from './authenticator.js'
import {
  SigningEnvelope,
  RawTxHashEnvelope,
  BitcoinMessageEnvelope,
  WebAuthnEnvelope,
} from './envelope.js'
import { Ed25519Keypair } from '../keypairs/ed25519/index.js'

describe('BitcoinSignMessage', () => {
  it('should correctly construct with valid txData and messageInfo', () => {
    const txData = new Uint8Array([1, 2, 3, 4])
    const messageInfo = 'Test Message Info'
    const bitcoinSignMessage = new BitcoinSignMessage(txData, messageInfo)

    expect(bitcoinSignMessage.messagePrefix).toBe('\u0018Bitcoin Signed Message:\n')
    expect(bitcoinSignMessage.messageInfo).toBe('Rooch Transaction:\nTest Message Info\n')
    expect(bitcoinSignMessage.txHash).toEqual(txData)
  })

  it('should correctly generate raw message string', () => {
    const txData = new Uint8Array([1, 2, 3, 4])
    const messageInfo = 'Test Message Info'
    const bitcoinSignMessage = new BitcoinSignMessage(txData, messageInfo)

    expect(bitcoinSignMessage.txHash).toEqual(txData)
    expect(bitcoinSignMessage.raw()).toBe('Rooch Transaction:\nTest Message Info\n01020304')
  })

  it('should handle empty messageInfo gracefully', () => {
    const txData = new Uint8Array([])
    const messageInfo = ''
    const bitcoinSignMessage = new BitcoinSignMessage(txData, messageInfo)

    expect(bitcoinSignMessage.messageInfo).toBe('Rooch Transaction:\n')
    expect(bitcoinSignMessage.raw()).toBe('Rooch Transaction:\n')
  })

  it('should correctly encode message with valid txHash and messageInfo', () => {
    const txData = new Uint8Array([0x01, 0x02, 0x03, 0x04])
    const messageInfo = 'Example message info'
    const bitcoinSignMessage = new BitcoinSignMessage(txData, messageInfo)
    const encodedData = bitcoinSignMessage.encode()
    expect(encodedData).toBeInstanceOf(Uint8Array)
    expect(encodedData.length).toBeLessThanOrEqual(255)
  })
})

describe('Envelope', () => {
  const mockTxHash = new Uint8Array([
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
  ])

  describe('RawTxHashEnvelope', () => {
    it('should return tx_hash as message', () => {
      const envelope = new RawTxHashEnvelope()
      const message = envelope.buildMessage(mockTxHash)

      expect(message).toEqual(mockTxHash)
      expect(envelope.getEnvelopeType()).toBe(SigningEnvelope.RawTxHash)
    })
  })

  describe('BitcoinMessageEnvelope', () => {
    it('should build canonical template message', () => {
      const envelope = new BitcoinMessageEnvelope()
      const message = envelope.buildMessage(mockTxHash)

      // Should start with "Rooch Transaction:\n"
      const messageStr = new TextDecoder().decode(message)
      expect(messageStr.startsWith('Rooch Transaction:\n')).toBe(true)
      expect(messageStr).toContain(
        '0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20',
      )
      expect(envelope.getEnvelopeType()).toBe(SigningEnvelope.BitcoinMessageV0)
    })

    it('should compute Bitcoin message digest', () => {
      const envelope = new BitcoinMessageEnvelope()
      const digest = envelope.computeDigest(mockTxHash)

      expect(digest).toBeInstanceOf(Uint8Array)
      expect(digest.length).toBe(32) // SHA256 output length
    })

    it('should build Bitcoin message with correct format', () => {
      const envelope = new BitcoinMessageEnvelope()
      const bitcoinMessage = envelope.buildBitcoinMessage(mockTxHash)

      // Should start with Bitcoin message prefix
      expect(bitcoinMessage[0]).toBe(0x18) // Length of "Bitcoin Signed Message:\n"
      const messageStart = new TextDecoder().decode(bitcoinMessage.slice(1, 25))
      expect(messageStart).toBe('Bitcoin Signed Message:\n')
    })
  })

  describe('WebAuthnEnvelope', () => {
    const mockAuthData = new Uint8Array([0xa1, 0xa2, 0xa3, 0xa4])
    const mockClientData = new Uint8Array([0xb1, 0xb2, 0xb3, 0xb4])

    it('should store authenticator data and client data', () => {
      const envelope = new WebAuthnEnvelope(mockAuthData, mockClientData)

      expect(envelope.getAuthenticatorData()).toEqual(mockAuthData)
      expect(envelope.getClientDataJson()).toEqual(mockClientData)
      expect(envelope.getEnvelopeType()).toBe(SigningEnvelope.WebAuthnV0)
    })

    it('should throw error when building message directly', () => {
      const envelope = new WebAuthnEnvelope(mockAuthData, mockClientData)

      expect(() => envelope.buildMessage(mockTxHash)).toThrow(
        'WebAuthn envelope requires special handling in authenticator',
      )
    })

    it('should compute WebAuthn digest', () => {
      const envelope = new WebAuthnEnvelope(mockAuthData, mockClientData)
      const digest = envelope.computeDigest(mockTxHash)

      expect(digest).toBeInstanceOf(Uint8Array)
      expect(digest.length).toBe(mockAuthData.length + 32) // auth_data + SHA256(client_data)

      // Should start with authenticator data
      expect(digest.slice(0, mockAuthData.length)).toEqual(mockAuthData)
    })
  })
})

describe('Authenticator with Envelope', () => {
  let keypair: Ed25519Keypair
  const mockTxHash = new Uint8Array([
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
  ])

  beforeEach(() => {
    keypair = Ed25519Keypair.generate()
  })

  describe('sessionWithEnvelope', () => {
    it('should create authenticator with default RawTxHash envelope', async () => {
      const authenticator = await Authenticator.sessionWithEnvelope(mockTxHash, keypair)

      expect(authenticator).toBeInstanceOf(Authenticator)
      expect(authenticator.authValidatorId).toBe(0) // SESSION validator
      expect(authenticator.payload).toBeInstanceOf(Uint8Array)

      // Should be v1 format (scheme + signature + public_key)
      const expectedLength = 1 + 64 + 32 // Ed25519: 1 + 64 + 32
      expect(authenticator.payload.length).toBe(expectedLength)
    })

    it('should create authenticator with explicit RawTxHash envelope', async () => {
      const envelope = new RawTxHashEnvelope()
      const authenticator = await Authenticator.sessionWithEnvelope(mockTxHash, keypair, envelope)

      expect(authenticator).toBeInstanceOf(Authenticator)

      // Should be v1 format since RawTxHash uses v1
      const expectedLength = 1 + 64 + 32
      expect(authenticator.payload.length).toBe(expectedLength)
    })

    it('should create authenticator with BitcoinMessage envelope', async () => {
      const envelope = new BitcoinMessageEnvelope()
      const authenticator = await Authenticator.sessionWithEnvelope(mockTxHash, keypair, envelope)

      expect(authenticator).toBeInstanceOf(Authenticator)

      // Should be v2 format with message
      // Format: scheme(1) + envelope(1) + signature(64) + public_key(32) + message_len + message
      expect(authenticator.payload.length).toBeGreaterThan(1 + 1 + 64 + 32)

      // Check envelope type in payload
      expect(authenticator.payload[1]).toBe(SigningEnvelope.BitcoinMessageV0)
    })

    it('should create authenticator with WebAuthn envelope', async () => {
      const mockAuthData = new Uint8Array([0xa1, 0xa2, 0xa3, 0xa4])
      const mockClientData = new Uint8Array([0xb1, 0xb2, 0xb3, 0xb4])
      const envelope = new WebAuthnEnvelope(mockAuthData, mockClientData)

      const authenticator = await Authenticator.sessionWithEnvelope(mockTxHash, keypair, envelope)

      expect(authenticator).toBeInstanceOf(Authenticator)

      // Should be v2 format with WebAuthn data
      expect(authenticator.payload.length).toBeGreaterThan(1 + 1 + 64 + 32)

      // Check envelope type in payload
      expect(authenticator.payload[1]).toBe(SigningEnvelope.WebAuthnV0)
    })

    it('should throw error for unsupported envelope type', async () => {
      // Mock an unsupported envelope
      const mockEnvelope = {
        buildMessage: () => new Uint8Array(),
        getEnvelopeType: () => 0xff as SigningEnvelope, // Unsupported type
      }

      await expect(
        Authenticator.sessionWithEnvelope(mockTxHash, keypair, mockEnvelope),
      ).rejects.toThrow('Unsupported envelope type: 255')
    })
  })

  describe('backward compatibility', () => {
    it('should maintain compatibility between session() and sessionWithEnvelope()', async () => {
      const auth1 = await Authenticator.session(mockTxHash, keypair)
      const auth2 = await Authenticator.sessionWithEnvelope(mockTxHash, keypair)

      // Both should have same structure (v1 format)
      expect(auth1.authValidatorId).toBe(auth2.authValidatorId)
      expect(auth1.payload.length).toBe(auth2.payload.length)

      // Scheme should be the same
      expect(auth1.payload[0]).toBe(auth2.payload[0])
    })
  })
})
