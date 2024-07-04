// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { Ed25519Keypair, Secp256k1Keypair } from '../keypairs/index.js'
import { isValidAddress } from './util.js'

describe('Address util', () => {
  it('Valid Rooch Address', () => {
    const ed25519RoochAddr = new Ed25519Keypair().getRoochAddress()

    expect(isValidAddress(ed25519RoochAddr)).toBeTruthy()
    expect(isValidAddress(ed25519RoochAddr.toHexAddress())).toBeTruthy()
    expect(isValidAddress(ed25519RoochAddr.toBech32Address())).toBeTruthy()

    const btcAddr = new Secp256k1Keypair().getBitcoinAddress()
    const roochAddr = btcAddr.genRoochAddress()

    expect(isValidAddress(btcAddr)).toBeTruthy()
    expect(isValidAddress(roochAddr)).toBeTruthy()
    expect(isValidAddress(roochAddr.toHexAddress())).toBeTruthy()
    expect(isValidAddress(roochAddr.toBech32Address())).toBeTruthy()
  })
})
