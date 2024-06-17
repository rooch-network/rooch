// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'

import { decodeRoochSercetKey, ROOCH_SECRET_KEY_PREFIX } from '@/crypto'

import { Ed25519Keypair } from './keypair'

const TEST_CASES: [{ pk: string; sk: string }] = [
  {
    sk: 'roochsecretkey1qqzztph49dkdl3vyw5t6fecvtuesrt9f5f2lw8ndpvqael6rr42mwulf8v7',
    pk: '3z8zMjDk70frh7I0ZF269ROuM5PeMKsYxwgFgTZEH9s=',
  },
]

describe('Ed25519 keypair', () => {
  it('create ed25519 keypair', () => {
    const kp = Ed25519Keypair.generate()

    expect(kp.getPublicKey().toBytes()).toHaveLength(32)
  })

  it('export ed25519 keypair', () => {
    const kp = Ed25519Keypair.generate()
    const secret = kp.getSecretKey()

    expect(secret.startsWith(ROOCH_SECRET_KEY_PREFIX)).toBeTruthy()
  })

  it('Create ed25519 keypair from secret key', () => {
    // valid secret key is provided by rooch keystore
    const { sk, pk } = TEST_CASES[0]

    const key = decodeRoochSercetKey(sk)
    const keypair = Ed25519Keypair.fromSecretKey(key.secretKey)

    expect(keypair.getPublicKey().toBase64()).toEqual(pk)

    const keypair1 = Ed25519Keypair.fromSecretKey(sk)

    expect(keypair1.getPublicKey().toBase64()).toEqual(pk)
  })

  it('Invalid mnemonics to derive ed25519 keypair', () => {
    expect(() => {
      Ed25519Keypair.deriveKeypair('rooch')
    }).toThrow('Invalid mnemonic')
  })

  it('Recover ed25519 keypair by mnemonics', () => {
    // mnemonics is provided by rooch cli
    // TODO:
  })

  it('Sign data', async () => {
    const keypair = new Ed25519Keypair()
    const message = new TextEncoder().encode('hello rooch')
    const signature = await keypair.sign(message)
    const isValid = keypair.getPublicKey().verify(message, signature)
    expect(isValid).toBeTruthy()
  })

  it('Sign data same as rooch cli', async () => {
    // TODO:
  })
})
