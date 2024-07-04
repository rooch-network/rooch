// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { sha3_256 } from '../utils/index.js'
import { normalizeRoochAddress } from '../address/index.js'
import { Args, bcs, Serializer } from '../bcs/index.js'
import { address, Bytes, identifier, u8, u64 } from '../types/index.js'
import { CallFunctionArgs, CallScript } from './types.js'

const DEFAULT_GAS = BigInt(50000000)

export class CallFunction {
  address: string
  module: identifier
  function: identifier
  args: Args[]
  typeArgs: string[]

  constructor(input: CallFunctionArgs) {
    const [pkg, mod, fn] =
      'target' in input ? input.target.split('::') : [input.address, input.module, input.function]

    this.address = pkg
    this.module = mod
    this.function = fn

    this.args = input.args || []
    this.typeArgs = input.typeArgs?.map((item) => Serializer.typeTagToString(item)) || []
  }

  functionId(): string {
    return `${normalizeRoochAddress(this.address)}::${this.module}::${this.function}`
  }

  encodeArgs(): string[] {
    return this.args?.map((item) => item.encodeWithHex())
  }

  encodeArgsWithUtf8(): string {
    return ''
  }

  encodeArgsToByteArrays(): u8[][] {
    return this.args.map((item) => item.encode()).map((item) => Array.from(item))
  }
}

type MoveActionType = CallFunction | CallScript

export class MoveAction {
  scheme: number
  val: MoveActionType

  private constructor(scheme: number, val: MoveActionType) {
    this.scheme = scheme
    this.val = val
  }

  static newCallFunction(input: CallFunctionArgs) {
    return new MoveAction(1, new CallFunction(input))
  }

  static newCallScript(input: CallScript) {
    return new MoveAction(2, input)
  }
}

export class TransactionData {
  sender?: address
  sequenceNumber?: u64
  chainId?: u64
  maxGas: u64
  action: MoveAction

  constructor(
    action: MoveAction,
    sender?: string,
    sequenceNumber?: bigint,
    chainId?: bigint,
    maxGas?: bigint,
  ) {
    this.sender = sender
    this.sequenceNumber = sequenceNumber
    this.chainId = chainId
    this.action = action
    this.maxGas = maxGas || DEFAULT_GAS
  }

  encode(): Bytes {
    const call = this.action.val as CallFunction

    return bcs.RoochTransactionData.serialize({
      sender: this.sender!,
      sequenceNumber: this.sequenceNumber!,
      chainId: this.chainId!,
      maxGas: this.maxGas,
      action: {
        kind: 'CallFunction',
        functionId: {
          moduleId: {
            address: call.address,
            name: call.module,
          },
          name: call.function,
        },
        args: Array.from(call.encodeArgsToByteArrays()),
        typeArgs: call.typeArgs,
      },
    }).toBytes()
  }

  hash(): Bytes {
    return sha3_256(this.encode())
  }
}
