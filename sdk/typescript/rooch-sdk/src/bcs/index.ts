// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bcs } from '@mysten/bcs'

import {
  Address,
  Authenticator,
  BitcoinAuthPayload,
  CallFunction,
  FunctionId,
  ModuleId,
  MoveAction,
  MultiChainAddress,
  ObjectId,
  raw,
  RoochTransaction,
  RoochTransactionData,
  ScriptCall,
} from './bcs.js'

export { BcsType, type BcsTypeOptions } from '@mysten/bcs'
export { Args, type ArgType } from './args.js'
export { Serializer } from './serializer.js'
export { type StructTag, type TypeTag } from './types.js'

export const RoochBcs = {
  U8: bcs.u8(),
  U16: bcs.u16(),
  U32: bcs.u32(),
  U64: bcs.u64(),
  U128: bcs.u128(),
  U256: bcs.u256(),
  Bool: bcs.bool(),
  Raw: raw,
  String: bcs.string(),
  Address,
  MultiChainAddress,
  ObjectId,
  BitcoinAuthPayload,
  ModuleId,
  FunctionId,
  ScriptCall,
  CallFunction,
  MoveAction,
  RoochTransactionData,
  Authenticator,
  RoochTransaction,
  ...bcs,
}

export { RoochBcs as bcs }
