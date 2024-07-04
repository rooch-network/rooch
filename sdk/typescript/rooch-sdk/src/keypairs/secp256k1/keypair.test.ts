// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { Secp256k1Keypair } from './keypair.js'
import { decodeRoochSercetKey, ROOCH_SECRET_KEY_PREFIX } from '../../crypto/index.js'

const TEST_CASES: [{ pk: string; sk: string }] = [
  {
    sk: 'roochsecretkey1q9rc3ryrp644d33yy4d2c9mg7wnuuxag7mqs0uq6yp7nmv6yd7usu2j6v3z',
    pk: 'Au4i1I9dB6BvAQ+aX8mt4f/wVKjYLhOkD6LEcgB/WBjq',
  },
]

describe('Secp256k1 keypair', () => {
  it('Create secp256k1 keypair', () => {
    const kp = Secp256k1Keypair.generate()
    expect(kp.getPublicKey().toBytes()).toHaveLength(33)
  })

  it('Export secp256k1 keypair', () => {
    const kp = Secp256k1Keypair.generate()
    const secret = kp.getSecretKey()

    expect(secret.startsWith(ROOCH_SECRET_KEY_PREFIX)).toBeTruthy()
  })

  it('Create secp256k1 keypair from secret key', () => {
    // valid secret key is provided by rooch keystore
    const { sk, pk } = TEST_CASES[0]

    const key = decodeRoochSercetKey(sk)
    const keypair = Secp256k1Keypair.fromSecretKey(key.secretKey)

    expect(keypair.getPublicKey().toBase64()).toEqual(pk)

    const keypair1 = Secp256k1Keypair.fromSecretKey(sk)

    expect(keypair1.getPublicKey().toBase64()).toEqual(pk)
  })

  describe('sign', () => {
    it('should sign data', async () => {
      const keypair = new Secp256k1Keypair()
      const message = new TextEncoder().encode('hello world')
      const signature = await keypair.sign(message)
      const isValid = await keypair.getPublicKey().verify(message, signature)
      expect(isValid).toBeTruthy()
    })

    it('Sign data same as rooch cli', async () => {
      // TODO:
    })
  })
})
