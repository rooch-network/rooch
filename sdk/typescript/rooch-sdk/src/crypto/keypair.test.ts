// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { decodeRoochSercetKey, encodeRoochSercetKey, ROOCH_SECRET_KEY_PREFIX } from './keypair.js'
import { bech32 } from 'bech32'
import { SIGNATURE_FLAG_TO_SCHEME } from './signatureScheme.js'

describe('decodeRoochSercetKey function', () => {
  it('should correctly decode a valid Bech32 encoded string', () => {
    const prefix = ROOCH_SECRET_KEY_PREFIX
    const secretKey = new Uint8Array(32).fill(0)
    const signatureFlag = 0x00
    const words = bech32.toWords(new Uint8Array([signatureFlag, ...secretKey]))
    const value = bech32.encode(prefix, words)

    const decoded = decodeRoochSercetKey(value)
    expect(decoded).toEqual({
      schema: SIGNATURE_FLAG_TO_SCHEME[signatureFlag],
      secretKey: secretKey,
    })
  })

  it('should throw an error for invalid prefix', () => {
    const invalidPrefix = 'invalidprefix'
    const secretKey = new Uint8Array(32).fill(0)
    const signatureFlag = 0x00
    const words = bech32.toWords(new Uint8Array([signatureFlag, ...secretKey]))
    const value = bech32.encode(invalidPrefix, words)

    expect(() => decodeRoochSercetKey(value)).toThrow('invalid private key prefix')
  })

  it('should throw an error for invalid Bech32 encoded string', () => {
    const value = 'roochsecretkey1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq'
    expect(() => decodeRoochSercetKey(value)).toThrow()
  })

  it('should have a secretKey of the correct length', () => {
    const prefix = ROOCH_SECRET_KEY_PREFIX
    const secretKey = new Uint8Array(32).fill(0)
    const signatureFlag = 0x00
    const words = bech32.toWords(new Uint8Array([signatureFlag, ...secretKey]))
    const value = bech32.encode(prefix, words)

    const decoded = decodeRoochSercetKey(value)
    expect(decoded.secretKey.length).toBe(32)
  })

  it('should correctly determine the signature scheme', () => {
    const prefix = ROOCH_SECRET_KEY_PREFIX
    const secretKey = new Uint8Array(32).fill(0)
    const signatureFlag = 0x01
    const words = bech32.toWords(new Uint8Array([signatureFlag, ...secretKey]))
    const value = bech32.encode(prefix, words)

    const decoded = decodeRoochSercetKey(value)
    expect(decoded.schema).toBe(SIGNATURE_FLAG_TO_SCHEME[signatureFlag])
  })
})

describe('encodeRoochSercetKey', () => {
  it('should encode correctly when given a valid 32-byte private key with ED25519 scheme', () => {
    const privateKey = new Uint8Array(32).fill(1)
    const scheme = 'ED25519'
    const encoded = encodeRoochSercetKey(privateKey, scheme)
    expect(encoded).toBe(
      bech32.encode(ROOCH_SECRET_KEY_PREFIX, bech32.toWords(new Uint8Array([0x00, ...privateKey]))),
    )
  })

  it('should encode correctly when given a valid 32-byte private key with Secp256k1 scheme', () => {
    const privateKey = new Uint8Array(32).fill(2)
    const scheme = 'Secp256k1'
    const encoded = encodeRoochSercetKey(privateKey, scheme)
    expect(encoded).toBe(
      bech32.encode(ROOCH_SECRET_KEY_PREFIX, bech32.toWords(new Uint8Array([0x01, ...privateKey]))),
    )
  })

  it('should throw an error when the private key length is not 32 bytes', () => {
    const invalidPrivateKey = new Uint8Array(31)
    const scheme = 'ED25519'
    expect(() => encodeRoochSercetKey(invalidPrivateKey, scheme)).toThrow('Invalid bytes length')
  })

  it('should handle edge case where the private key is an array of zeros', () => {
    const privateKey = new Uint8Array(32).fill(0)
    const scheme = 'ED25519'
    const encoded = encodeRoochSercetKey(privateKey, scheme)
    expect(encoded).toBe(
      bech32.encode(ROOCH_SECRET_KEY_PREFIX, bech32.toWords(new Uint8Array([0x00, ...privateKey]))),
    )
  })

  it('should handle edge case where the private key is an array of maximum byte values', () => {
    const privateKey = new Uint8Array(32).fill(255)
    const scheme = 'Secp256k1'
    const encoded = encodeRoochSercetKey(privateKey, scheme)
    expect(encoded).toBe(
      bech32.encode(ROOCH_SECRET_KEY_PREFIX, bech32.toWords(new Uint8Array([0x01, ...privateKey]))),
    )
  })
})
