// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'

import { RoochMultiChainID } from '@roochnetwork/rooch-sdk'
import { MultiChainAddress } from './multiChainAddress'

describe('multiChainAddress', () => {
  it('should parse multi chain address is ok', () => {
    // native swgwit p2wpkh
    let p2wpkhAddress = 'bc1pq5ttgyqu5pmfn9aqt09d978mky2fndxr3ed3ntszta75g9q6xrlqlwyl0r'
    let expectMultiChainAddressBytesWithP2wpkh = new Uint8Array([
      0, 0, 0, 0, 0, 0, 0, 0, 34, 2, 1, 5, 22, 180, 16, 28, 160, 118, 153, 151, 160, 91, 202, 210,
      248, 251, 177, 20, 153, 180, 195, 142, 91, 25, 174, 2, 95, 125, 68, 20, 26, 48, 254,
    ])
    let p2wpkhMulticChainAddress = new MultiChainAddress(RoochMultiChainID.Bitcoin, p2wpkhAddress)
    expect(p2wpkhMulticChainAddress.toBytes()).toEqual(expectMultiChainAddressBytesWithP2wpkh)

    // nestd segwit p2sh-p2wpkh
    let p2shAddress = '39fVbRM2TNBdNZPYeAJM7sHCPKczaZL7LV'
    let expectMultiChainAddressBytesWithP2SH = new Uint8Array([
      0, 0, 0, 0, 0, 0, 0, 0, 21, 1, 87, 119, 69, 184, 41, 233, 101, 189, 166, 156, 217, 192, 62, 9,
      151, 234, 162, 97, 49, 192,
    ])
    let p2shMulticChainAddress = new MultiChainAddress(RoochMultiChainID.Bitcoin, p2shAddress)
    expect(p2shMulticChainAddress.toBytes()).toEqual(expectMultiChainAddressBytesWithP2SH)

    // taproot p2tr
    let p2trAddress = 'bc1pq5ttgyqu5pmfn9aqt09d978mky2fndxr3ed3ntszta75g9q6xrlqlwyl0r'
    let expectMultiChainAddressBytesWithP2TR = new Uint8Array([
      0, 0, 0, 0, 0, 0, 0, 0, 34, 2, 1, 5, 22, 180, 16, 28, 160, 118, 153, 151, 160, 91, 202, 210,
      248, 251, 177, 20, 153, 180, 195, 142, 91, 25, 174, 2, 95, 125, 68, 20, 26, 48, 254,
    ])
    let p2TRMulticChainAddress = new MultiChainAddress(RoochMultiChainID.Bitcoin, p2trAddress)
    expect(p2TRMulticChainAddress.toBytes()).toEqual(expectMultiChainAddressBytesWithP2TR)

    // legacy p2pkh
    let p2pkhAddress = '15MJa2Jx2yA5iERwTKENY2WdWF3vnN6KVe'
    let expectMultiChainAddressBytesWithP2PKH = new Uint8Array([
      0, 0, 0, 0, 0, 0, 0, 0, 21, 0, 47, 183, 125, 16, 71, 244, 105, 179, 253, 132, 178, 184, 60, 5,
      68, 57, 97, 253, 162, 187,
    ])
    let p2pkhMulticChainAddress = new MultiChainAddress(RoochMultiChainID.Bitcoin, p2pkhAddress)
    expect(p2pkhMulticChainAddress.toBytes()).toEqual(expectMultiChainAddressBytesWithP2PKH)
  })
})
