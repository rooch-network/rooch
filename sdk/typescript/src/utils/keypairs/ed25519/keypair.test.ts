// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { Ed25519Keypair } from './keypair'
import { fromB64 } from '../../../types/bcs'
import nacl from 'tweetnacl'

describe('create', () => {
  it('should new ed25519 keypair', () => {
    const kp = Ed25519Keypair.generate()

    expect(kp.getPublicKey().toBytes()).toHaveLength(32)
  })

  it('should create ed25519 keypair from secret key', () => {
    // valid secret key is provided by rooch keystore
    const validSecretKey = 'AM4KesRCz7SzQt+F9TK0IvznFGxjUWGgRNlJxbTLW0Ol'

    const secretKey = fromB64(validSecretKey)

    const keypair = Ed25519Keypair.fromSecretKey(secretKey.slice(1))

    expect(keypair.getPublicKey().toRoochPublicKey()).toEqual(
      'AD69FribWlJzoVMZP3hpppmwkVYYz4c2DZw+PKi2MsvA',
    )
  })

  it('should invalid mnemonics to derive ed25519 keypair', () => {
    expect(() => {
      Ed25519Keypair.deriveKeypair('rooch')
    }).toThrow('Invalid mnemonic')
  })

  it('should recover ed25519 keypair by mnemonics', () => {
    // mnemonics is provided by rooch cli
    const mnemonics = 'main south wonder traffic identify two baby job doctor eye betray sniff'
    const address = '0xe3e66642fee3090f5518fc2412af42f3b27a26af3dd7bf0436a5604680a654f6'

    const k1 = Ed25519Keypair.deriveKeypair(mnemonics)

    expect(k1.toRoochAddress()).toBe(address)
  })
})

describe('sign', () => {
  it('should sign data', () => {
    const keypair = new Ed25519Keypair()
    const signData = new TextEncoder().encode('hello world')
    const signature = keypair.signData(signData)
    const isValid = nacl.sign.detached.verify(signData, signature, keypair.getPublicKey().toBytes())
    expect(isValid).toBeTruthy()
  })

  // TODO: more test
})
