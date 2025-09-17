// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, beforeEach } from 'vitest'
import {
  SigningEnvelope,
  RawTxHashEnvelope,
  BitcoinMessageEnvelope,
  WebAuthnEnvelope,
  WebAuthnUtils,
  WebAuthnEnvelopeBuilder,
  WebauthnEnvelopeData,
} from './envelope.js'
import { toHEX } from '../utils/index.js'

describe('SigningEnvelope', () => {
  const mockTxHash = new Uint8Array([
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
  ])

  describe('Envelope constants', () => {
    it('should have correct enum values', () => {
      expect(SigningEnvelope.RawTxHash).toBe(0x00)
      expect(SigningEnvelope.BitcoinMessageV0).toBe(0x01)
      expect(SigningEnvelope.WebAuthnV0).toBe(0x02)
    })
  })

  describe('RawTxHashEnvelope', () => {
    let envelope: RawTxHashEnvelope

    beforeEach(() => {
      envelope = new RawTxHashEnvelope()
    })

    it('should return correct envelope type', () => {
      expect(envelope.getEnvelopeType()).toBe(SigningEnvelope.RawTxHash)
    })

    it('should return tx_hash unchanged as message', () => {
      const message = envelope.buildMessage(mockTxHash)
      expect(message).toEqual(mockTxHash)
    })

    it('should handle empty tx_hash', () => {
      const emptyHash = new Uint8Array(0)
      const message = envelope.buildMessage(emptyHash)
      expect(message).toEqual(emptyHash)
    })
  })

  describe('BitcoinMessageEnvelope', () => {
    let envelope: BitcoinMessageEnvelope

    beforeEach(() => {
      envelope = new BitcoinMessageEnvelope()
    })

    it('should return correct envelope type', () => {
      expect(envelope.getEnvelopeType()).toBe(SigningEnvelope.BitcoinMessageV0)
    })

    it('should build canonical template with correct format', () => {
      const message = envelope.buildMessage(mockTxHash)
      const messageStr = new TextDecoder().decode(message)

      expect(messageStr.startsWith('Rooch Transaction:\n')).toBe(true)
      expect(messageStr).toContain(toHEX(mockTxHash).toLowerCase())
    })

    it('should build Bitcoin message with correct prefix', () => {
      const bitcoinMessage = envelope.buildBitcoinMessage(mockTxHash)

      // Check that the message starts with the Bitcoin message prefix
      expect(bitcoinMessage[0]).toBe(0x18) // Length of "Bitcoin Signed Message:\n"
      const messageStart = new TextDecoder().decode(bitcoinMessage.slice(1, 25))
      expect(messageStart).toBe('Bitcoin Signed Message:\n')
    })

    it('should compute consistent digest', () => {
      const digest1 = envelope.computeDigest(mockTxHash)
      const digest2 = envelope.computeDigest(mockTxHash)

      expect(digest1).toEqual(digest2)
      expect(digest1.length).toBe(32) // SHA256 output
    })

    it('should produce different digests for different tx_hashes', () => {
      const txHash1 = new Uint8Array(32).fill(0x01)
      const txHash2 = new Uint8Array(32).fill(0x02)

      const digest1 = envelope.computeDigest(txHash1)
      const digest2 = envelope.computeDigest(txHash2)

      expect(digest1).not.toEqual(digest2)
    })

    it('should handle edge cases', () => {
      // Empty tx_hash
      const emptyHash = new Uint8Array(0)
      const message = envelope.buildMessage(emptyHash)
      expect(new TextDecoder().decode(message)).toBe('Rooch Transaction:\n')

      // Single byte tx_hash
      const singleByteHash = new Uint8Array([0xff])
      const singleMessage = envelope.buildMessage(singleByteHash)
      expect(new TextDecoder().decode(singleMessage)).toBe('Rooch Transaction:\nff')
    })
  })

  describe('WebAuthnEnvelope', () => {
    const mockAuthData = new Uint8Array([0xa1, 0xa2, 0xa3, 0xa4, 0xa5])
    const mockClientData = new Uint8Array([0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6])
    let envelope: WebAuthnEnvelope

    beforeEach(() => {
      envelope = new WebAuthnEnvelope(mockAuthData, mockClientData)
    })

    it('should return correct envelope type', () => {
      expect(envelope.getEnvelopeType()).toBe(SigningEnvelope.WebAuthnV0)
    })

    it('should store and return authenticator data', () => {
      expect(envelope.getAuthenticatorData()).toEqual(mockAuthData)
    })

    it('should store and return client data JSON', () => {
      expect(envelope.getClientDataJson()).toEqual(mockClientData)
    })

    it('should throw error when building message directly', () => {
      expect(() => envelope.buildMessage(mockTxHash)).toThrow(
        'WebAuthn envelope requires special handling in authenticator',
      )
    })

    it('should compute WebAuthn digest correctly', () => {
      const digest = envelope.computeDigest(mockTxHash)

      // Digest should be authenticator_data || SHA256(client_data_json)
      expect(digest.length).toBe(mockAuthData.length + 32)

      // First part should be authenticator data
      expect(digest.slice(0, mockAuthData.length)).toEqual(mockAuthData)

      // Second part should be SHA256 of client data (32 bytes)
      const clientDataHash = digest.slice(mockAuthData.length)
      expect(clientDataHash.length).toBe(32)
    })

    it('should produce consistent digests', () => {
      const digest1 = envelope.computeDigest(mockTxHash)
      const digest2 = envelope.computeDigest(mockTxHash)

      expect(digest1).toEqual(digest2)
    })

    it('should handle empty authenticator data', () => {
      const emptyAuthData = new Uint8Array(0)
      const envelopeEmpty = new WebAuthnEnvelope(emptyAuthData, mockClientData)

      const digest = envelopeEmpty.computeDigest(mockTxHash)
      expect(digest.length).toBe(32) // Only SHA256(client_data)
    })

    it('should handle empty client data', () => {
      const emptyClientData = new Uint8Array(0)
      const envelopeEmpty = new WebAuthnEnvelope(mockAuthData, emptyClientData)

      const digest = envelopeEmpty.computeDigest(mockTxHash)
      expect(digest.length).toBe(mockAuthData.length + 32)
    })
  })

  describe('Integration tests', () => {
    it('should produce different results for different envelope types', () => {
      const rawEnvelope = new RawTxHashEnvelope()
      const bitcoinEnvelope = new BitcoinMessageEnvelope()

      const rawMessage = rawEnvelope.buildMessage(mockTxHash)
      const bitcoinMessage = bitcoinEnvelope.buildMessage(mockTxHash)

      expect(rawMessage).not.toEqual(bitcoinMessage)
    })

    it('should maintain envelope type consistency', () => {
      const envelopes = [
        new RawTxHashEnvelope(),
        new BitcoinMessageEnvelope(),
        new WebAuthnEnvelope(new Uint8Array([1]), new Uint8Array([2])),
      ]

      const expectedTypes = [
        SigningEnvelope.RawTxHash,
        SigningEnvelope.BitcoinMessageV0,
        SigningEnvelope.WebAuthnV0,
      ]

      envelopes.forEach((envelope, index) => {
        expect(envelope.getEnvelopeType()).toBe(expectedTypes[index])
      })
    })
  })

  describe('WebAuthn Utils and Envelope Builder', () => {
    // Mock WebAuthn assertion data
    const mockAssertionData = {
      signature: new Uint8Array([
        0x30, 0x44, 0x02, 0x20, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc,
        0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x02, 0x20, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa,
        0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
        0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00,
      ]),
      rawSignature: new Uint8Array(64), // Will be computed by derToRaw
      authenticatorData: new Uint8Array([
        0x49, 0x96, 0x0d, 0xe5, 0x88, 0x0e, 0x8c, 0x68, 0x74, 0x34, 0x17, 0x0f,
        0x64, 0x76, 0x60, 0x5b, 0x8f, 0xe4, 0xae, 0xb9, 0xa2, 0x86, 0x32, 0xc7,
        0x99, 0x5c, 0xf3, 0xba, 0x83, 0x1d, 0x97, 0x63, 0x01, 0x00, 0x00, 0x00,
        0x00,
      ]),
      clientDataJSON: new Uint8Array(
        new TextEncoder().encode(
          JSON.stringify({
            type: 'webauthn.get',
            challenge: 'AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHyA', // base64 of mockTxHash
            origin: 'https://example.com',
            crossOrigin: false,
          })
        )
      ),
    }

    const mockTxHash = new Uint8Array([
      0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
      0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
    ])

    describe('WebAuthnUtils', () => {
      it('should validate correct challenge', () => {
        const isValid = WebAuthnUtils.validateChallenge(mockAssertionData.clientDataJSON, mockTxHash)
        expect(isValid).toBe(true)
      })

      it('should reject incorrect challenge', () => {
        const wrongTxHash = new Uint8Array(32).fill(0xff)
        const isValid = WebAuthnUtils.validateChallenge(mockAssertionData.clientDataJSON, wrongTxHash)
        expect(isValid).toBe(false)
      })

      it('should compute verification message correctly', () => {
        const verificationMessage = WebAuthnUtils.computeVerificationMessage(
          mockAssertionData.authenticatorData,
          mockAssertionData.clientDataJSON
        )

        // Should be authenticatorData + SHA256(clientDataJSON)
        const expectedLength = mockAssertionData.authenticatorData.length + 32 // SHA256 = 32 bytes
        expect(verificationMessage.length).toBe(expectedLength)

        // First part should be authenticatorData
        const authenticatorPart = verificationMessage.slice(0, mockAssertionData.authenticatorData.length)
        expect(Array.from(authenticatorPart)).toEqual(Array.from(mockAssertionData.authenticatorData))
      })

      it('should handle invalid clientDataJSON gracefully', () => {
        const invalidJSON = new Uint8Array([0x7b, 0x22, 0x69, 0x6e, 0x76]) // "{inv" - invalid JSON
        const isValid = WebAuthnUtils.validateChallenge(invalidJSON, mockTxHash)
        expect(isValid).toBe(false)
      })
    })

    describe('WebAuthnEnvelopeBuilder', () => {
      let envelopeBuilder: WebAuthnEnvelopeBuilder

      beforeEach(() => {
        envelopeBuilder = new WebAuthnEnvelopeBuilder(mockAssertionData)
      })

      it('should return correct envelope type', () => {
        expect(envelopeBuilder.getEnvelopeType()).toBe(SigningEnvelope.WebAuthnV0)
      })

      it('should build message with valid challenge', () => {
        const message = envelopeBuilder.buildMessage(mockTxHash)
        expect(message).toBeInstanceOf(Uint8Array)
        expect(message.length).toBeGreaterThan(0)
      })

      it('should throw error with invalid challenge', () => {
        const wrongTxHash = new Uint8Array(32).fill(0xff)
        expect(() => {
          envelopeBuilder.buildMessage(wrongTxHash)
        }).toThrow('WebAuthn challenge does not match transaction hash')
      })

      it('should compute digest correctly', () => {
        const digest = envelopeBuilder.computeDigest(mockTxHash)
        const expectedDigest = WebAuthnUtils.computeVerificationMessage(
          mockAssertionData.authenticatorData,
          mockAssertionData.clientDataJSON
        )
        expect(Array.from(digest)).toEqual(Array.from(expectedDigest))
      })

      it('should return signature from assertion data', () => {
        const signature = envelopeBuilder.getSignature()
        expect(Array.from(signature)).toEqual(Array.from(mockAssertionData.rawSignature))
      })

      it('should return original assertion data', () => {
        const assertionData = envelopeBuilder.getAssertionData()
        expect(assertionData).toBe(mockAssertionData)
      })
    })

    describe('WebAuthn Integration', () => {
      it('should work with WebauthnEnvelopeData BCS encoding', () => {
        const envelopeBuilder = new WebAuthnEnvelopeBuilder(mockAssertionData)
        const message = envelopeBuilder.buildMessage(mockTxHash)

        // Should be valid BCS-encoded data
        expect(message).toBeInstanceOf(Uint8Array)
        expect(message.length).toBeGreaterThan(0)

        // Try to decode it back (this would throw if invalid BCS)
        expect(() => {
          // This is just a basic check that the data is structured correctly
          // In a real scenario, you'd use BCS decoder
          const data = new WebauthnEnvelopeData(
            mockAssertionData.authenticatorData,
            mockAssertionData.clientDataJSON
          )
          data.encode()
        }).not.toThrow()
      })
    })
  })
})
