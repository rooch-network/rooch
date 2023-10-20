// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { encodeShortString, decodeShortString } from '../utils'

export class Felt {
  static PRIME = BigInt(
    '21888242871839275222246405745257275088548364400416034343698204186575808495617',
  )

  value: bigint

  constructor(val: bigint) {
    this.validate(val)
    this.value = val
  }

  validate(val: bigint) {
    if (val < 0 || val >= Felt.PRIME) {
      throw new Error('Felt value out of range')
    }
  }

  toBigNumber() {
    return this.value
  }

  toHex() {
    return this.value.toString(16)
  }

  toString() {
    return this.value.toString()
  }

  toText() {
    return decodeShortString('0x' + this.toHex())
  }

  static fromString(text: string): Felt {
    return new Felt(BigInt(encodeShortString(text)))
  }
}
