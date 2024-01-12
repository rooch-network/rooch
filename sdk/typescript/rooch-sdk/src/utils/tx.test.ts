// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import {
  encodeFunctionCall,
  typeTagToSCS,
  structTagToSCS,
  addressToSCS,
  encodeStructTypeTags,
  encodeArg,
  strcutTagToString,
  typeTagToString,
  strcutTagToObjectID,
} from './tx'
import { toHexString } from './hex'
import { TypeTag, StructTag, AccountAddress, Arg } from '../types'
import * as rooch_types from '../generated/runtime/rooch_types/mod'
import { bytes } from '../generated/runtime/serde/mod'

describe('encodeFunctionCall', () => {
  it('should encode a function call correctly', () => {
    const functionId: string = '0x1::ModuleName::function_name'
    const tyArgs: TypeTag[] = ['Bool', 'U8']
    const args: bytes[] = [new Uint8Array([1, 2, 3]), new Uint8Array([4, 5, 6])]

    const result = encodeFunctionCall(functionId, tyArgs, args)
    expect(result).toBeInstanceOf(rooch_types.MoveActionVariantFunction)
    // Add more assertions to check the properties of the result object.
  })
})

describe('typeTagToSCS', () => {
  it('should convert a TypeTag to an SCS TypeTag', () => {
    const ty: TypeTag = 'Bool'
    const result = typeTagToSCS(ty)

    expect(result).toBeInstanceOf(rooch_types.TypeTagVariantbool)
  })

  // Add more test cases for other TypeTags.
})

describe('structTagToSCS', () => {
  it('should convert a StructTag to an SCS StructTag', () => {
    const data: StructTag = {
      address: '0x1',
      module: 'ModuleName',
      name: 'StructName',
      type_params: ['Bool', 'U8'],
    }

    const result = structTagToSCS(data)

    expect(result).toBeInstanceOf(rooch_types.StructTag)
    // Add more assertions to check the properties of the result object.
  })
})

describe('addressToSCS', () => {
  it('should convert an AccountAddress to an SCS AccountAddress', () => {
    const addr: AccountAddress = '0x1'

    const result = addressToSCS(addr)

    expect(result).toBeInstanceOf(rooch_types.AccountAddress)
    // Add more assertions to check the properties of the result object.
  })
})

describe('encodeStructTypeTags', () => {
  it('should encode an array of struct type tags correctly', () => {
    const typeArgsString: string[] = [
      '0x1::ModuleName::StructName<0x2::AnotherModule::AnotherStruct>',
    ]

    const result = encodeStructTypeTags(typeArgsString)

    expect(result).toHaveLength(1)
    expect(result[0]).toHaveProperty('Struct')
    // Add more assertions to check the properties of the result object.
  })
})

describe('encodeArg', () => {
  it('should encode Vector TypeTag', () => {
    const arg = {
      type: { Vector: 'U8' },
      value: [100],
    } as Arg

    const result = encodeArg(arg)

    expect(toHexString(result)).toBe('0x0164')
  })
})

describe('strcutTagToString', () => {
  it('strcutTagToString with no type_params', () => {
    const structTag: StructTag = {
      address: '00000000000000000000000000000001',
      module: 'module1',
      name: 'name1',
    }

    const result = strcutTagToString(structTag)
    expect(result).toBe(
      '0000000000000000000000000000000000000000000000000000000000000001::module1::name1',
    )
  })

  it('strcutTagToString with type_params', () => {
    const structTag: StructTag = {
      address: '00000000000000000000000000000001',
      module: 'module1',
      name: 'name1',
      type_params: [
        'U8',
        { Vector: 'U64' },
        {
          Struct: {
            address: '0000000000000000000000000000000a',
            module: 'module2',
            name: 'name2',
          },
        },
      ],
    }

    const result = strcutTagToString(structTag)
    expect(result).toBe(
      '0000000000000000000000000000000000000000000000000000000000000001::module1::name1<U8,Vector<U64>,000000000000000000000000000000000000000000000000000000000000000a::module2::name2>',
    )
  })
})

describe('typeTagToString', () => {
  it('typeTagToString with string type', () => {
    const typeTag: TypeTag = 'U8'

    const result = typeTagToString(typeTag)
    expect(result).toBe('U8')
  })

  it('typeTagToString with Vector type', () => {
    const typeTag: TypeTag = { Vector: 'U64' }

    const result = typeTagToString(typeTag)
    expect(result).toBe('Vector<U64>')
  })

  it('typeTagToString with Struct type', () => {
    const typeTag: TypeTag = {
      Struct: {
        address: '0000000000000000000000000000000a',
        module: 'module2',
        name: 'name2',
      },
    }

    const result = typeTagToString(typeTag)
    expect(result).toBe(
      '000000000000000000000000000000000000000000000000000000000000000a::module2::name2',
    )
  })

  it('typeTagToString with unknown type', () => {
    const typeTag: any = { Unknown: 'U64' }

    expect(() => typeTagToString(typeTag)).toThrowError()
  })
})

describe('strcutTagToObjectID', () => {
  it('test_named_object_id', () => {
    const structTag: StructTag = {
      address: '0x3',
      module: 'timestamp',
      name: 'Timestamp',
      type_params: [],
    }

    const timestamp_object_id = strcutTagToObjectID(structTag)
    const object_id = '0x711ab0301fd517b135b88f57e84f254c94758998a602596be8ae7ba56a0d14b3'
    expect(timestamp_object_id).toBe(object_id)
  })

  it('test_account_named_object_id', () => {
    const structTag: StructTag = {
      address: '0x3',
      module: 'coin_store',
      name: 'CoinStore',
      type_params: [
        {
          Struct: {
            address: '0x3',
            module: 'gas_coin',
            name: 'GasCoin',
            type_params: [],
          },
        },
      ],
    }

    const coin_store_object_id = strcutTagToObjectID(structTag)
    const object_id = '0xd073508b9582eff4e01078dc2e62489c15bbef91b6a2e568ac8fb33f0cf54daa'
    expect(coin_store_object_id).toBe(object_id)
  })
})
