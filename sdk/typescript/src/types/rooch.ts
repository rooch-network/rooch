// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
export type Identifier = string
export type AccountAddress = string
export type HashValue = string
export type Bool = boolean
export type U8 = number
export type U16 = number
export type U64 = bigint
export type U128 = number
export type U256 = string
export type I64 = number
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
  | 'U64'
  | 'U128'
  | 'Address'
  | 'Signer'
  | { Vector: TypeTag }
  | { Struct: StructTag }

export type ArgType = Bool | U8 | U64 | U128 | AccountAddress
export type Arg = {
  type: TypeTag
  value: ArgType
}
