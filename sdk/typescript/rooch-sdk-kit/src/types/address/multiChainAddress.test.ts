// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'

import { RoochMultiChainID } from '@roochnetwork/rooch-sdk'
import { MultiChainAddress } from './multiChainAddress'

describe('multiChainAddress', () => {
  it('should parse multi chain address is ok', () => {
    let expectMultiChainAddress = 'bitcoin1pqrawtj0v3gpl4qqkkcavjdequnj9nswvqu40gx7m'
    let expectMultiChainAddressBytes = new Uint8Array([
      0, 0, 0, 0, 0, 0, 0, 0, 21, 0, 250, 229, 201, 236, 138, 3, 250, 128, 22, 182, 58, 201, 55, 32,
      228, 228, 89, 193, 204, 7,
    ])

    let addressStr = 'bc1qltjunmy2q0agq94k8tynwg8yu3vurnq8h7yc7p'

    let multicChainAddress = new MultiChainAddress(RoochMultiChainID.Bitcoin, addressStr)

    expect(multicChainAddress.toBech32()).toEqual(expectMultiChainAddress)
    expect(multicChainAddress.toBytes()).toEqual(expectMultiChainAddressBytes)
  })
})
