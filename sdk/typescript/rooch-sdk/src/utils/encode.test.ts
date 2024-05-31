// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { describe, it, expect } from 'vitest'
import { TypeTag, StructTag } from '../types'
import {
  typeTagToString,
  structTagToCanonicalString,
  typeTagToCanonicalString,
  structTagToObjectID,
} from './encode'

describe('typeTagToString', () => {
  it('should handle string type tags correctly', () => {
    expect(typeTagToString('U8')).toEqual('U8')
    expect(typeTagToString('U64')).toEqual('U64')
    expect(typeTagToString('Address')).toEqual('Address')
  })

  it('should handle Vector type tags correctly', () => {
    const vectorTypeTag = { Vector: 'U8' } as TypeTag
    expect(typeTagToString(vectorTypeTag)).toEqual('Vector<U8>')

    const nestedVectorTypeTag = { Vector: { Vector: 'U8' } } as TypeTag
    expect(typeTagToString(nestedVectorTypeTag)).toEqual('Vector<Vector<U8>>')
  })

  it('should handle Struct type tags correctly', () => {
    const structTypeTag = {
      Struct: {
        address: '0x1',
        module: 'Account',
        name: 'Account',
      },
    }
    expect(typeTagToString(structTypeTag)).toEqual('0x1::Account::Account')

    const structTypeTagWithTypeParams = {
      Struct: {
        address: '0x1',
        module: 'Account',
        name: 'Account',
        type_params: ['U8', { Vector: 'U64' }],
      },
    } as TypeTag
    expect(typeTagToString(structTypeTagWithTypeParams)).toEqual(
      '0x1::Account::Account<U8, Vector<U64>>',
    )
  })
})

describe('strcutTagToString', () => {
  it('strcutTagToString with no type_params', () => {
    const structTag: StructTag = {
      address: '00000000000000000000000000000001',
      module: 'module1',
      name: 'name1',
    }

    const result = structTagToCanonicalString(structTag)
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

    const result = structTagToCanonicalString(structTag)
    expect(result).toBe(
      '0000000000000000000000000000000000000000000000000000000000000001::module1::name1<u8,vector<u64>,000000000000000000000000000000000000000000000000000000000000000a::module2::name2>',
    )
  })
})

describe('typeTagToString', () => {
  it('typeTagToString with string type', () => {
    const typeTag: TypeTag = 'U8'

    const result = typeTagToCanonicalString(typeTag)
    expect(result).toBe('u8')
  })

  it('typeTagToString with Vector type', () => {
    const typeTag: TypeTag = { Vector: 'U64' }

    const result = typeTagToCanonicalString(typeTag)
    expect(result).toBe('vector<u64>')
  })

  it('typeTagToString with Struct type', () => {
    const typeTag: TypeTag = {
      Struct: {
        address: '0000000000000000000000000000000a',
        module: 'module2',
        name: 'name2',
      },
    }

    const result = typeTagToCanonicalString(typeTag)
    expect(result).toBe(
      '000000000000000000000000000000000000000000000000000000000000000a::module2::name2',
    )
  })

  it('typeTagToString with unknown type', () => {
    const typeTag: any = { Unknown: 'U64' }

    expect(() => typeTagToCanonicalString(typeTag)).toThrowError()
  })
})

describe('structTagToObjectID', () => {
  it('test_named_object_id', () => {
    const structTag: StructTag = {
      address: '0x3',
      module: 'object',
      name: 'Timestamp',
      type_params: [],
    }

    const timestamp_object_id = structTagToObjectID(structTag)
    const object_id = '0x5921974509dbe44ab84328a625f4a6580a5f89dff3e4e2dec448cb2b1c7f5b9'
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

    const coin_store_object_id = structTagToObjectID(structTag)
    const object_id = '0x9fe449ea079f937dbc977733d6b0ae450ec44ba22ec8793076026606db1c9f49'
    expect(coin_store_object_id).toBe(object_id)
  })
})
