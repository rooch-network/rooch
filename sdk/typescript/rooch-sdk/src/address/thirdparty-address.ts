// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Bytes } from '../types/index.js'

import { Address } from './address.js'
import { RoochAddress } from './rooch.js'

export abstract class ThirdPartyAddress extends Address {
  abstract genMultiChainAddress(): Bytes
  abstract genRoochAddress(): RoochAddress
  abstract toBytes(): Bytes
  protected abstract decode(): any
}
