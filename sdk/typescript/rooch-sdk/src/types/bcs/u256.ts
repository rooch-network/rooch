// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { BcsSerializer } from '../../generated/runtime/bcs/mod'
import { Serializable } from './serializable'

const BIG_128Fs = BigInt('0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF')
const BIG_128 = BigInt(128)

export class U256 implements Serializable {
  private value: bigint | number

  constructor(value: bigint | number) {
    this.value = value
  }

  public serialize(se: BcsSerializer): void {
    const low = BigInt(this.value.toString()) & BIG_128Fs
    const high = BigInt(this.value.toString()) >> BIG_128

    // write little endian number
    se.serializeU128(low)
    se.serializeU128(high)
  }
}
