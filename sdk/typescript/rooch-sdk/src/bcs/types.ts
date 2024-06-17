// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { address } from '@/types'

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

/**
 * Kind of a TypeTag which is represented by a Move type identifier.
 */
export interface StructTag {
  address: address
  module: string
  name: string
  typeParams?: TypeTag[]
}

export interface StructTagA {
  address: address
  module: string
  name: string
  typeParams: TypeTagA[]
}

export type TypeTagA =
  | { bool: null | true }
  | { u8: null | true }
  | { u64: null | true }
  | { u128: null | true }
  | { address: null | true }
  | { signer: null | true }
  | { vector: TypeTagA }
  | { struct: StructTagA }
  | { u16: null | true }
  | { u32: null | true }
  | { u256: null | true }
