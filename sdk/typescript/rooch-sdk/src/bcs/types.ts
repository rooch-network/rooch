// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { address } from '../types/index.js'

/**
 * TypeTag object. A decoupled `0x...::module::Type<???>` parameter.
 */
export type TypeTag =
  | 'u8'
  | 'u16'
  | 'u32'
  | 'u64'
  | 'u128'
  | 'u256'
  | 'bool'
  | 'address'
  | 'signer'
  | { Vector: TypeTag }
  | { Struct: StructTag }
  | string

/**
 * Kind of a TypeTag which is represented by a Move type identifier.
 */
export interface StructTag {
  address: string
  module: string
  name: string
  typeParams?: TypeTag[]
}

export interface BcsStructTag {
  address: address
  module: string
  name: string
  typeParams: BcsTypeTag[]
}

export type BcsTypeTag =
  | { bool: null | true }
  | { u8: null | true }
  | { u64: null | true }
  | { u128: null | true }
  | { address: null | true }
  | { signer: null | true }
  | { vector: BcsTypeTag }
  | { struct: BcsStructTag }
  | { u16: null | true }
  | { u32: null | true }
  | { u256: null | true }
