// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'

import { StructTag, TypeTag } from './types.js'
import { Serializer } from './serializer.js'

describe('Serializer', () => {
  it('struct tag to string with no type params', () => {
    const testData: StructTag = {
      address: '00000000000000000000000000000001',
      module: 'module1',
      name: 'name1',
    }

    const expectStr =
      '0000000000000000000000000000000000000000000000000000000000000001::module1::name1'
    const resultStr = Serializer.structTagToCanonicalString(testData)
    expect(resultStr).toBe(expectStr)
  })

  it('struct tag to string with type_params', () => {
    const testData: StructTag = {
      address: '00000000000000000000000000000001',
      module: 'module1',
      name: 'name1',
      typeParams: [
        'u8',
        { Vector: 'u64' },
        {
          Struct: {
            address: '0000000000000000000000000000000a',
            module: 'module2',
            name: 'name2',
          },
        },
      ],
    }

    const expectStr =
      '0000000000000000000000000000000000000000000000000000000000000001::module1::name1<u8,vector<u64>,000000000000000000000000000000000000000000000000000000000000000a::module2::name2>'
    const resultStr = Serializer.structTagToCanonicalString(testData)
    expect(resultStr).toBe(expectStr)
  })

  it('type tag to string with vector type', () => {
    const testData: TypeTag = { Vector: 'u64' }

    const expectStr = 'vector<u64>'
    const resultStr = Serializer.typeTagToString(testData)
    expect(resultStr).toBe(expectStr)
  })

  it('type tag to string with struct type', () => {
    const testData: TypeTag = {
      Struct: {
        address: '0000000000000000000000000000000a',
        module: 'module2',
        name: 'name2',
      },
    }

    const resultStr = Serializer.typeTagToString(testData)
    expect(resultStr).toBe(
      '000000000000000000000000000000000000000000000000000000000000000a::module2::name2',
    )
  })

  it('type tag to string with unknown type', () => {
    const testData: any = { Unknown: 'U64' }

    expect(() => Serializer.typeTagToString(testData)).toThrowError()
  })

  it('test named object id', () => {
    const testData: StructTag = {
      address: '0x2',
      module: 'timestamp',
      name: 'Timestamp',
      typeParams: [],
    }

    const expectObjectID = '0x4e8d2c243339c6e02f8b7dd34436a1b1eb541b0fe4d938f845f4dbb9d9f218a2'
    const resultObjectID = Serializer.structTagToObjectID(testData)
    expect(resultObjectID).toBe(expectObjectID)
  })

  it('test_account_named_object_id', () => {
    const testData: StructTag = {
      address: '0x3',
      module: 'coin_store',
      name: 'CoinStore',
      typeParams: [
        {
          Struct: {
            address: '0x3',
            module: 'gas_coin',
            name: 'GasCoin',
            typeParams: [],
          },
        },
      ],
    }
    const expectObjectID = '0x9fe449ea079f937dbc977733d6b0ae450ec44ba22ec8793076026606db1c9f49'
    const resultObjectID = Serializer.structTagToObjectID(testData)
    expect(resultObjectID).toBe(expectObjectID)
  })
})
