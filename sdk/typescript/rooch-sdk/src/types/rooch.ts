// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Serializable } from './bcs/serializable'

export type Identifier = string
export type AccountAddress = string
export type HashValue = string
export type Bool = boolean
export type U8 = number
export type U16 = number
export type U32 = number
export type U64 = bigint
export type U128 = bigint
export type U256 = bigint
export type BlockNumber = number
export type AuthenticationKey = string
export type MultiEd25519PublicKey = string
export type MultiEd25519Signature = string
export type EventKey = string

export type ModuleId = string | { address: AccountAddress; name: Identifier }
export type FunctionId =
  | string
  | { address: AccountAddress; module: Identifier; functionName: Identifier }

export interface StructTag {
  address: string
  module: string
  name: string
  // eslint-disable-next-line no-use-before-define
  type_params?: TypeTag[]
}

export type TypeTag =
  | 'Bool'
  | 'U8'
  | 'U16'
  | 'U32'
  | 'U64'
  | 'U128'
  | 'U256'
  | 'Address'
  | 'Signer'
  | 'Ascii'
  | 'String'
  | 'Object'
  | 'ObjectID'
  | 'Raw'
  | { Vector: TypeTag }
  | { Struct: StructTag }

export type ParsedObjectID = AccountAddress | StructTag

export type ArgType =
  | Bool
  | U8
  | U16
  | U32
  | U64
  | U128
  | U256
  | string
  | Uint8Array
  | AccountAddress
  | Serializable
  | ParsedObjectID
  | ArgType[]

export type Arg = {
  type: TypeTag
  value: ArgType
}
