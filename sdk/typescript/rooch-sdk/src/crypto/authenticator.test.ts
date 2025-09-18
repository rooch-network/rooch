// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect, beforeEach } from 'vitest'
import { BitcoinSignMessage, Authenticator } from './authenticator.js'
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

describe('Authenticator with Envelope', () => {
  let keypair: Ed25519Keypair
  const mockTxHash = new Uint8Array([
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
  ])

  beforeEach(() => {
    keypair = Ed25519Keypair.generate()
  })

  describe('DID authenticator with different signer types', () => {
    it('should auto-select RawTxHash envelope for regular signer', async () => {
      const vmFragment = 'test-vm-fragment'
      const authenticator = await Authenticator.did(mockTxHash, keypair, vmFragment)

      expect(authenticator).toBeInstanceOf(Authenticator)
      expect(authenticator.authValidatorId).toBe(4) // DID validator
      expect(authenticator.payload).toBeInstanceOf(Uint8Array)
    })

    it('should create DID authenticator with explicit BitcoinMessage envelope', async () => {
      const vmFragment = 'test-vm-fragment'
      const authenticator = await Authenticator.didBitcoinMessage(mockTxHash, keypair, vmFragment)

      expect(authenticator).toBeInstanceOf(Authenticator)
      expect(authenticator.authValidatorId).toBe(4) // DID validator
    })

    it('should create DID authenticator with WebAuthn assertion data', async () => {
      const vmFragment = 'test-vm-fragment'
      const mockAssertionData = {
        signature: new Uint8Array(64),
        rawSignature: new Uint8Array(64),
        authenticatorData: new Uint8Array([0xa1, 0xa2, 0xa3, 0xa4]),
        clientDataJSON: new Uint8Array(
          Buffer.from(
            JSON.stringify({
              challenge: Buffer.from(mockTxHash).toString('base64'),
              origin: 'https://example.com',
              type: 'webauthn.get',
            }),
          ),
        ),
      }

      const authenticator = await Authenticator.didWebAuthn(
        mockTxHash,
        vmFragment,
        mockAssertionData,
      )

      expect(authenticator).toBeInstanceOf(Authenticator)
      expect(authenticator.authValidatorId).toBe(4) // DID validator
    })
  })
})
