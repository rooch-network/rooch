// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { BcsSerializer } from '../../generated/runtime/bcs/mod'

const BIG_128Fs = BigInt('0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF')
const BIG_128 = BigInt(128)

export function serializeU256(se: BcsSerializer, value: bigint | number) {
  const low = BigInt(value.toString()) & BIG_128Fs
  const high = BigInt(value.toString()) >> BIG_128

  // write little endian number
  se.serializeU128(low)
  se.serializeU128(high)
}
