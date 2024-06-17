// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Bytes } from '@/types'

export const ROOCH_ADDRESS_LENGTH = 32

export interface Address {
  toBytes(): Bytes
  toStr(): string
}
