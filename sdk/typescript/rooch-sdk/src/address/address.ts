// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Bytes } from '../types/index.js'
export const ROOCH_BECH32_PREFIX = 'rooch'

export const ROOCH_ADDRESS_LENGTH = 32

export interface Address {
  toBytes(): Bytes
  toStr(): string
}
