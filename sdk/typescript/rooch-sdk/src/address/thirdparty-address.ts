// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Bytes } from '../types/index.js'

import { Address } from './address.js'
import { RoochAddress } from './rooch.js'

export abstract class ThirdPartyAddress implements Address {
  protected rawAddress: string

  protected constructor(input: string) {
    this.rawAddress = input
  }

  abstract genMultiChainAddress(): Bytes
  abstract genRoochAddress(): RoochAddress
  abstract toBytes(): Bytes
  protected abstract decode(): any

  toStr(): string {
    return this.rawAddress
  }

  toSortStr(address: string | null | undefined, start = 6, end = 4): string {
    try {
      if (!address) {
        return ''
      }
      if (address.length <= start + end) {
        return address
      }
      return `${address.substring(0, start)}...${address.substring(
        address.length - end,
        address.length,
      )}`
    } catch (error) {
      return ''
    }
  }
}
