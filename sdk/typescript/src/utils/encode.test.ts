import { describe, it, expect } from 'vitest'
import { TypeTag } from "../types"
import { typeTagToString } from './encode'

describe('typeTagToString', () => {
    it('should handle string type tags correctly', () => {
        expect(typeTagToString('U8')).toEqual('U8');
        expect(typeTagToString('U64')).toEqual('U64');
        expect(typeTagToString('Address')).toEqual('Address');
    });

    it('should handle Vector type tags correctly', () => {
        const vectorTypeTag = { Vector: 'U8' } as TypeTag;
        expect(typeTagToString(vectorTypeTag)).toEqual('Vector<U8>');

        const nestedVectorTypeTag = { Vector: { Vector: 'U8' } } as TypeTag;
        expect(typeTagToString(nestedVectorTypeTag)).toEqual('Vector<Vector<U8>>');
    });

    it('should handle Struct type tags correctly', () => {
        const structTypeTag = {
            Struct: {
                address: '0x1',
                module: 'Account',
                name: 'Account'
            }
        };
        expect(typeTagToString(structTypeTag)).toEqual('0x1::Account::Account');

        const structTypeTagWithTypeParams = {
            Struct: {
                address: '0x1',
                module: 'Account',
                name: 'Account',
                type_params: ['U8', { Vector: 'U64' }]
            }
        } as TypeTag;
        expect(typeTagToString(structTypeTagWithTypeParams)).toEqual('0x1::Account::Account<U8, Vector<U64>>');
    });
})
