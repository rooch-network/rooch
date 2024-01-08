// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { BaseWallet } from './baseWallet'
import { RoochMultiChainID, SerializedSignature } from '@roochnetwork/rooch-sdk'
import { Buffer } from 'buffer'
import { MultiChainAddress, MultiChainAddressLength } from '../address'

export abstract class BitcoinWallet extends BaseWallet {
  protected toSerializedSignature(signature: string, fromAddress: string): SerializedSignature {
    let signBuffer = Buffer.from(signature, 'base64')

    const normalizeSignBuffer = Buffer.concat([
      signBuffer.subarray(1),
      Buffer.from([this.normalize_recovery_id(signBuffer[0])]),
    ])

    let multiAddress = new MultiChainAddress(RoochMultiChainID.Bitcoin, fromAddress)

    const serializedSignature = new Uint8Array(normalizeSignBuffer.length + MultiChainAddressLength)
    serializedSignature.set(normalizeSignBuffer)
    serializedSignature.set(multiAddress.toBytes(), normalizeSignBuffer.length)

    return serializedSignature
  }
  normalize_recovery_id(v: number) {
    let normalizeV = v - 27 - 4

    if (normalizeV < 0) {
      normalizeV = normalizeV + 4
    }

    return normalizeV
  }
}
