// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Address } from '../address/address.js'

import { Bytes } from './bytes.js'

export type u8 = number
export type u16 = number
export type u32 = number
export type u64 = bigint
export type u128 = bigint
export type u256 = bigint
export type bool = boolean
export type objectId = string
export type address = string | Address | Bytes
export type identifier = string
