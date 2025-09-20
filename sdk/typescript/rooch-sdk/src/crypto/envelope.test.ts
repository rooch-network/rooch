// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { SigningEnvelope, WebAuthnUtils, WebauthnEnvelopeData } from './envelope.js'
import { toB64 } from '../utils/index.js'

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

  describe('WebauthnEnvelopeData', () => {
    it('should create and encode WebAuthn envelope data', () => {
      const authenticatorData = new Uint8Array([0xa1, 0xa2, 0xa3, 0xa4])
      const clientDataJSON = new Uint8Array([0xb1, 0xb2, 0xb3, 0xb4])

      const envelopeData = new WebauthnEnvelopeData(authenticatorData, clientDataJSON)

      expect(envelopeData.authenticator_data).toEqual(authenticatorData)
      expect(envelopeData.client_data_json).toEqual(clientDataJSON)

      const encoded = envelopeData.encode()
      expect(encoded).toBeInstanceOf(Uint8Array)
      expect(encoded.length).toBeGreaterThan(0)
    })
  })

  describe('WebAuthnUtils', () => {
    describe('validateChallenge', () => {
      it('should validate matching challenge', () => {
        const clientDataJSON = new Uint8Array(
          Buffer.from(
            JSON.stringify({
              challenge: toB64(mockTxHash),
              origin: 'https://example.com',
              type: 'webauthn.get',
            }),
          ),
        )

        const result = WebAuthnUtils.validateChallenge(clientDataJSON, mockTxHash)
        expect(result).toBe(true)
      })

      it('should reject non-matching challenge', () => {
        const differentHash = new Uint8Array(32).fill(0xff)
        const clientDataJSON = new Uint8Array(
          Buffer.from(
            JSON.stringify({
              challenge: toB64(differentHash),
              origin: 'https://example.com',
              type: 'webauthn.get',
            }),
          ),
        )

        const result = WebAuthnUtils.validateChallenge(clientDataJSON, mockTxHash)
        expect(result).toBe(false)
      })

      it('should handle invalid JSON gracefully', () => {
        const invalidJSON = new Uint8Array(Buffer.from('invalid json'))

        const result = WebAuthnUtils.validateChallenge(invalidJSON, mockTxHash)
        expect(result).toBe(false)
      })
    })

    describe('computeVerificationMessage', () => {
      it('should compute correct verification message', () => {
        const authenticatorData = new Uint8Array([0xa1, 0xa2, 0xa3, 0xa4])
        const clientDataJSON = new Uint8Array([0xb1, 0xb2, 0xb3, 0xb4])

        const result = WebAuthnUtils.computeVerificationMessage(authenticatorData, clientDataJSON)

        expect(result).toBeInstanceOf(Uint8Array)
        expect(result.length).toBe(authenticatorData.length + 32) // auth_data + SHA256(client_data)

        // Should start with authenticator data
        expect(result.slice(0, authenticatorData.length)).toEqual(authenticatorData)
      })
    })

    describe('parseAssertionResponse', () => {
      it('should parse WebAuthn assertion response', () => {
        // Mock a simple DER signature (this is a simplified example)
        const mockDERSignature = new Uint8Array([
          0x30,
          0x44, // SEQUENCE, length 68
          0x02,
          0x20, // INTEGER, length 32 (r)
          ...new Array(32).fill(0x01), // r value
          0x02,
          0x20, // INTEGER, length 32 (s)
          ...new Array(32).fill(0x02), // s value
        ])

        const mockResponse = {
          signature: mockDERSignature.buffer,
          authenticatorData: new Uint8Array([0xa1, 0xa2, 0xa3, 0xa4]).buffer,
          clientDataJSON: new Uint8Array([0xb1, 0xb2, 0xb3, 0xb4]).buffer,
        } as AuthenticatorAssertionResponse

        const result = WebAuthnUtils.parseAssertionResponse(mockResponse)

        expect(result.signature).toEqual(mockDERSignature)
        expect(result.rawSignature).toBeInstanceOf(Uint8Array)
        expect(result.rawSignature.length).toBe(64) // Raw signature should be 64 bytes
        expect(result.authenticatorData).toEqual(new Uint8Array([0xa1, 0xa2, 0xa3, 0xa4]))
        expect(result.clientDataJSON).toEqual(new Uint8Array([0xb1, 0xb2, 0xb3, 0xb4]))
      })
    })
  })
})
