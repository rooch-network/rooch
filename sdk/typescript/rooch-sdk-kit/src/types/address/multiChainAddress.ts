// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { getAddressInfo } from './bitcoinAddress'
import { bech32 } from 'bech32'
import { RoochMultiChainID, RoochMultiChainIDToString } from '@roochnetwork/rooch-sdk'

export const MultiChainAddressLength = 31

export class MultiChainAddress {
  private multiChainId: RoochMultiChainID
  private rawAddress: Uint8Array

  // TODO: support all Chain
  constructor(multiChainId: RoochMultiChainID, address: string) {
    this.multiChainId = multiChainId

    this.rawAddress = getAddressInfo(address).bytes
  }

  toBytes(): Uint8Array {
    // TODO: use bcs
    const bitcoinSer = [0, 0, 0, 0, 0, 0, 0, 0, 22, 2]

    const tmp = new Uint8Array(this.rawAddress.length + bitcoinSer.length)
    tmp.set(bitcoinSer)
    tmp.set(this.rawAddress, bitcoinSer.length)
    return tmp
  }

  toBech32(): string {
    // discard ...
    const data = [1].concat(bech32.toWords(this.rawAddress))

    const address = bech32.encode(RoochMultiChainIDToString(this.multiChainId), data)

    return address
  }
}
