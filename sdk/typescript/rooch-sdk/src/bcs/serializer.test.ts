// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'

import { StructTag, TypeTag } from './types.js'
import { Serializer } from './serializer.js'
import { Address } from '../address/address.js'
import { RoochAddress } from '../address/rooch.js'

describe('Serializer', () => {
  it('struct tag to string with no type params', () => {
    const testData: StructTag = {
      address: '0x00000000000000000000000000000001',
      module: 'module1',
      name: 'name1',
    }

    const expectStr =
      '0x0000000000000000000000000000000000000000000000000000000000000001::module1::name1'
    const resultStr = Serializer.structTagToCanonicalString(testData)
    expect(resultStr).toBe(expectStr)
  })

  it('struct tag to string with type_params', () => {
    const testData: StructTag = {
      address: '0x00000000000000000000000000000001',
      module: 'module1',
      name: 'name1',
      typeParams: [
        'u8',
        { Vector: 'u64' },
        {
          Struct: {
            address: '0x0000000000000000000000000000000a',
            module: 'module2',
            name: 'name2',
          },
        },
      ],
    }

    const expectStr =
      '0x0000000000000000000000000000000000000000000000000000000000000001::module1::name1<u8,vector<u64>,0x000000000000000000000000000000000000000000000000000000000000000a::module2::name2>'
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
        address: '0x0000000000000000000000000000000a',
        module: 'module2',
        name: 'name2',
      },
    }

    const resultStr = Serializer.typeTagToString(testData)
    expect(resultStr).toBe(
      '0x000000000000000000000000000000000000000000000000000000000000000a::module2::name2',
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

    const expectObjectID = '0x3a7dfe7a9a5cd608810b5ebd60c7adf7316667b17ad5ae703af301b74310bcca'
    const resultObjectID = Serializer.structTagToObjectID(testData)
    expect(resultObjectID).toBe(expectObjectID)
  })

  it('test named object id with type params', () => {
    const testData: StructTag = {
      address: '0x3',
      module: 'coin_store',
      name: 'CoinStore',
      typeParams: [
        {
          Struct: {
            address: '0x3',
            module: 'gas_coin',
            name: 'RGas',
            typeParams: [],
          },
        },
      ],
    }
    const expectObjectID = '0xfdda11f9cc18bb30973779eb3610329d7e0e3c6ecce05b4d77b5a839063bff66'
    const resultObjectID = Serializer.structTagToObjectID(testData)
    expect(resultObjectID).toBe(expectObjectID)
  })

  it('test named object id with type params', () => {
    const address = new RoochAddress('0x42')
    const testData: StructTag = {
      address: '0x3',
      module: 'coin_store',
      name: 'CoinStore',
      typeParams: [
        {
          Struct: {
            address: '0x3',
            module: 'gas_coin',
            name: 'RGas',
            typeParams: [],
          },
        },
      ],
    }
    const expectObjectID = '0x562409111a2ca55814e56eb42186470c4adda4a04a4a84140690f4d68e8e1c06'
    const resultObjectID = Serializer.accountNamedObjectID(address, testData)
    expect(resultObjectID).toBe(expectObjectID)
  })
})
