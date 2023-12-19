// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { Ed25519Keypair } from './keypair'
import { fromB64 } from '../../b64'
import nacl from 'tweetnacl'
import { fromHexString, toHexString } from '../../hex'

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

  // Publish modules to address: 0x7194e6bf0860250491496174e7f7d7a9a9424d41734830656b9466787c04480c
  // wallet_context sign, hash:"0xfb4163fdb9ab1f9fc31c13d6f2d2ea91b411bb0a80a204f2c0ceb68f6b35e172", sign:"0x00b49c1ee8d28e55e7f526f33cff9ec51cea1bd0bad8a7d92a30d1f59f84d0043da3409e680dc719e8f0cd5aaf22b01e5f72433bdaaeed6ab3ae476fcf5e593907056f1378f1c31a98b7be133d43d242ecb1b8b418b2090f90c6febb49a417200e"
  it('should sign data same as rooch cli', async () => {
    const mnemonics = 'nose aspect organ harbor move prepare raven manage lamp consider oil front'
    const keypair = Ed25519Keypair.deriveKeypair(mnemonics)
    const roochAddress = keypair.toRoochAddress()
    expect(roochAddress).toBe('0x7194e6bf0860250491496174e7f7d7a9a9424d41734830656b9466787c04480c')

    const signHash = fromHexString(
      '0xfb4163fdb9ab1f9fc31c13d6f2d2ea91b411bb0a80a204f2c0ceb68f6b35e172',
    )
    const signature = await keypair.signMessageWithHashed(signHash)
    const signatureHex = toHexString(signature.signature)
    expect(signatureHex).toBe(
      '0x00b49c1ee8d28e55e7f526f33cff9ec51cea1bd0bad8a7d92a30d1f59f84d0043da3409e680dc719e8f0cd5aaf22b01e5f72433bdaaeed6ab3ae476fcf5e593907056f1378f1c31a98b7be133d43d242ecb1b8b418b2090f90c6febb49a417200e',
    )
  })

  // TODO: more test
})
