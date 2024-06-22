// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { bcs, SerializedBcs } from '@mysten/bcs'

import { Bytes } from '../types/bytes.js'
import { isBytes, toHEX } from '../utils/index.js'
import { u16, u32, u8, u64, u128, u256, bool, address, objectId } from '../types/index.js'

import { Address, ObjectId } from './bcs.js'
import { Serializer } from './serializer.js'
import { StructTag } from './types.js'

export type ArgType =
  | 'u8'
  | 'u16'
  | 'u32'
  | 'u64'
  | 'u128'
  | 'u256'
  | 'bool'
  | 'string'
  | 'object'
  | 'objectId'
  | 'address'

export class Args {
  readonly value: Bytes

  constructor(input: Bytes) {
    this.value = input
  }

  encodeWithHex(): string {
    return toHEX(this.value)
  }

  encode(): Bytes {
    return this.value
  }

  static u8(input: u8) {
    return new Args(bcs.u8().serialize(input).toBytes())
  }

  static u16(input: u16) {
    return new Args(bcs.u16().serialize(input).toBytes())
  }

  static u32(input: u32) {
    return new Args(bcs.u32().serialize(input).toBytes())
  }

  static u64(input: u64) {
    return new Args(bcs.u64().serialize(input).toBytes())
  }

  static u128(input: u128) {
    return new Args(bcs.u128().serialize(input).toBytes())
  }

  static u256(input: u256) {
    return new Args(bcs.u256().serialize(input).toBytes())
  }

  static bool(input: bool) {
    return new Args(bcs.bool().serialize(input).toBytes())
  }

  static string(input: string) {
    return new Args(bcs.string().serialize(input).toBytes())
  }

  static address(input: address) {
    return new Args(Address.serialize(input).toBytes())
  }

  static object(input: StructTag) {
    return this.objectId(Serializer.structTagToObjectID(input))
  }

  static objectId(input: objectId) {
    return new Args(ObjectId.serialize(input).toBytes())
  }

  static struct(input: SerializedBcs<any> | Bytes): Args {
    return new Args(isBytes(input) ? input : input.toBytes())
  }

  static vec(type: ArgType, input: number[] | bigint[] | boolean[] | string[] | StructTag[]) {
    let _value: Bytes
    switch (type) {
      case 'u8':
        _value = bcs
          .vector(bcs.u8())
          .serialize(input as number[])
          .toBytes()
        break
      case 'u16':
        _value = bcs
          .vector(bcs.u16())
          .serialize(input as number[])
          .toBytes()
        break
      case 'u32':
        _value = bcs
          .vector(bcs.u32())
          .serialize(input as number[])
          .toBytes()
        break
      case 'u64':
        _value = bcs
          .vector(bcs.u64())
          .serialize(input as bigint[])
          .toBytes()
        break
      case 'u128':
        _value = bcs
          .vector(bcs.u128())
          .serialize(input as bigint[])
          .toBytes()
        break
      case 'u256':
        _value = bcs
          .vector(bcs.u256())
          .serialize(input as number[])
          .toBytes()
        break
      case 'bool':
        _value = bcs
          .vector(bcs.bool())
          .serialize(input as boolean[])
          .toBytes()
        break
      case 'string':
        _value = bcs
          .vector(bcs.string())
          .serialize(input as string[])
          .toBytes()
        break
      case 'object':
        const tmp = (input as StructTag[]).map(Serializer.structTagToObjectID)
        _value = bcs.vector(ObjectId).serialize(tmp).toBytes()
        break
      case 'objectId':
        _value = bcs
          .vector(ObjectId)
          .serialize(input as string[])
          .toBytes()
        break
      case 'address':
        _value = bcs
          .vector(Address)
          .serialize(input as string[])
          .toBytes()
        break
    }

    return new Args(_value)
  }
}
