// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bcs, BcsType, BcsTypeOptions } from '@mysten/bcs'

import { Bytes } from '@/types'
import { bytes, CoderType, fromHEX, toHEX } from '@/utils'
import { isValidRoochAddress, normalizeRoochAddress, ROOCH_ADDRESS_LENGTH } from '@/address'

import { Serializer } from './serializer'
import type { TypeTagA as TypeTagType } from './types'

type Merge<T> = T extends infer U ? { [K in keyof U]: U[K] } : never
type EnumKindTransform<T> = T extends infer U
  ? Merge<(U[keyof U] extends null | boolean ? object : U[keyof U]) & { kind: keyof U }>
  : never

function enumKind<T extends object, Input extends object>(type: BcsType<T, Input>) {
  return type.transform({
    input: ({ kind, ...val }: EnumKindTransform<Input>) =>
      ({
        [kind]: val,
      } as Input),
    output: (val) => {
      const key = Object.keys(val)[0] as keyof T

      return { kind: key, ...val[key] } as EnumKindTransform<T>
    },
  })
}

// TODO: support parse
export const raw = <T, Input>(
  type: BcsType<T, Input>,
  options?: BcsTypeOptions<T[], Iterable<Input> & { length: number }>,
): BcsType<T[], Iterable<Input> & { length: number }> => {
  return new BcsType<T[], Iterable<Input> & { length: number }>({
    name: `vector<${type.name}>`,
    read: (reader) => {
      const result: T[] = []
      for (let i = 0; i < length; i++) {
        result[i] = type.read(reader)
      }
      return result
    },
    write: (value, writer) => {
      for (const item of value) {
        type.write(item, writer)
      }
    },
    ...options,
    validate: (value) => {
      options?.validate?.(value)
      if (!('length' in value)) {
        throw new TypeError(`Expected array, found ${typeof value}`)
      }
    },
  })
}

export const RawBytes = (coder: CoderType = 'hex') => {
  return raw(bcs.u8()).transform({
    input: (input: string | Bytes) => (typeof input === 'string' ? bytes(coder, input) : input),
    output: (input) => input,
  })
}

export const Vector = (coder: CoderType = 'hex') => {
  return bcs.vector(bcs.u8()).transform({
    input: (input: string | Bytes) => (typeof input === 'string' ? bytes(coder, input) : input),
    output: (input) => new Uint8Array(input),
  })
}

export const Address = bcs.bytes(ROOCH_ADDRESS_LENGTH).transform({
  validate: (input) => {
    const address = typeof input === 'string' ? input : toHEX(input)
    if (!address || !isValidRoochAddress(normalizeRoochAddress(address))) {
      throw new Error(`Invalid Rooch address ${address}`)
    }
  },
  input: (input: string | Bytes) =>
    typeof input === 'string' ? fromHEX(normalizeRoochAddress(input)) : input,
  output: (val) => normalizeRoochAddress(toHEX(val)),
})

export const MultiChainAddress = bcs.struct('MultiChainAddress', {
  multiChainId: bcs.u64().transform({
    input: (input: number | bigint) => (typeof typeof input === 'number' ? BigInt(input) : input),
    output: (input) => BigInt(input),
  }),
  rawAddress: bcs.vector(bcs.u8()),
})

export const ObjectId = bcs.vector(Address).transform({
  input: (input: string | Bytes[]) => {
    if (typeof input === 'string') {
      const normalizeId = normalizeRoochAddress(input)
      let bytes = fromHEX(normalizeId)
      let addresses: Uint8Array[] = []
      for (let i = 0; i < bytes.length; i += ROOCH_ADDRESS_LENGTH) {
        let chunk = bytes.slice(i, i + ROOCH_ADDRESS_LENGTH)
        if (chunk.length !== ROOCH_ADDRESS_LENGTH) {
          throw new Error('Invalid chunk size')
        }
        addresses.push(chunk)
      }
      return addresses
    }

    return input
  },
  output: (val) => {
    return val.join('')
  },
})

const InnerTypeTag: BcsType<TypeTagType, TypeTagType> = bcs.enum('TypeTag', {
  bool: null,
  u8: null,
  u64: null,
  u128: null,
  address: null,
  signer: null,
  vector: bcs.lazy(() => InnerTypeTag),
  struct: bcs.lazy(() => StructTag),
  u16: null,
  u32: null,
  u256: null,
}) as BcsType<TypeTagType>

export const StructTag = bcs.struct('StructTag', {
  address: Address,
  module: bcs.string(),
  name: bcs.string(),
  typeParams: bcs.vector(InnerTypeTag),
})

export const TypeTag = InnerTypeTag.transform({
  input: (typeTag: string | TypeTagType) =>
    typeof typeTag === 'string' ? Serializer.typeTagParseFromStr(typeTag, true) : typeTag,
  output: (typeTag: TypeTagType) => Serializer.tagToString(typeTag),
})

export const BitcoinAuthPayload = bcs.struct('AuthPayload', {
  signature: Vector(),
  messagePrefix: Vector('utf8'),
  messageInfo: Vector('utf8'),
  publicKey: Vector('hex'),
  fromAddress: Vector('base64'),
})

export const ModuleId = bcs.struct('ModuleId', {
  address: Address,
  name: bcs.string(),
})

export const FunctionId = bcs.struct('FunctionId', {
  moduleId: ModuleId,
  name: bcs.string(),
})

export const ScriptCall = bcs.struct('ScriptCall', {
  code: RawBytes(),
  args: bcs.vector(bcs.u8()),
  typeArgs: bcs.vector(TypeTag),
})

export const CallFunction = bcs.struct('FunctionCall', {
  functionId: FunctionId,
  typeArgs: bcs.vector(TypeTag),
  args: bcs.vector(bcs.vector(bcs.u8())),
})

export const MoveAction = enumKind(
  bcs.enum('MoveAction', {
    ScriptCall,
    CallFunction,
  }),
)

export const RoochTransactionData = bcs.struct('RoochTransactionData', {
  sender: Address,
  sequenceNumber: bcs.u64(),
  chainId: bcs.u64(),
  maxGas: bcs.u64(),
  action: MoveAction,
})

export const Authenticator = bcs.struct('Authenticator', {
  authValidatorId: bcs.u64(),
  payload: bcs.vector(bcs.u8()),
})

export const RoochTransaction = bcs.struct('RoochTransaction', {
  data: raw(bcs.u8()),
  auth: raw(bcs.u8()),
})
