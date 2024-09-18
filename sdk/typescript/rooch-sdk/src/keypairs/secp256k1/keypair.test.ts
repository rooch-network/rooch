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

  it('Create secp256k1 keypair from CLI secret key', () => {
    const testKey = 'roochsecretkey1q969zv4rhqpuj0nkf2e644yppjf34p6zwr3gq0633qc7n9luzg6w6lycezc'
    const expectRoochHexAddress =
      '0xf892b3fd5fd0e93436ba3dc8d504413769d66901266143d00e49441079243ed0'
    const expectRoochBech32Address =
      'rooch1lzft8l2l6r5ngd468hyd2pzpxa5av6gpyes585qwf9zpq7fy8mgqh9npj5'
    const expectNoStrddress = 'npub1h54r2zvulk96qjmfnyy83mtry0pp5acnz6uvk637typxtvn90c8s0lrc0g'
    const expectBitcoinAddress = 'bcrt1pw9l5h7vepq8cnpugwm848x3at34gg5eq0mamdrjw0krunfjm0zfq65gjzz'

    const sk = Secp256k1Keypair.fromSecretKey(testKey)
    const addrView = sk.getSchnorrPublicKey().toAddress()

    expect(addrView.roochAddress.toHexAddress()).eq(expectRoochHexAddress)
    expect(addrView.roochAddress.toBech32Address()).eq(expectRoochBech32Address)
    expect(addrView.noStrAddress.toStr()).eq(expectNoStrddress)
    expect(addrView.bitcoinAddress.toStr()).eq(expectBitcoinAddress)
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
