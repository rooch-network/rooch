import { describe, it, expect } from "vitest"
import { encodeFunctionCall, typeTagToSCS, structTagToSCS, addressToSCS, encodeStructTypeTags } from './tx';
import { TypeTag, StructTag, AccountAddress } from "../types";
import * as rooch_types from "../generated/runtime/rooch_types/mod";
import { bytes } from '../generated/runtime/serde/mod';

describe('encodeFunctionCall', () => {
  it('should encode a function call correctly', () => {
    const functionId: string = '0x1::ModuleName::function_name';
    const tyArgs: TypeTag[] = ['Bool', 'U8'];
    const args: bytes[] = [new Uint8Array([1, 2, 3]), new Uint8Array([4, 5, 6])];

    const result = encodeFunctionCall(functionId, tyArgs, args);
    expect(result).toBeInstanceOf(rooch_types.MoveActionVariantFunction);
    // Add more assertions to check the properties of the result object.
  });
});

describe('typeTagToSCS', () => {
  it('should convert a TypeTag to an SCS TypeTag', () => {
    const ty: TypeTag = 'Bool';
    const result = typeTagToSCS(ty);

    expect(result).toBeInstanceOf(rooch_types.TypeTagVariantbool);
  });

  // Add more test cases for other TypeTags.
});

describe('structTagToSCS', () => {
  it('should convert a StructTag to an SCS StructTag', () => {
    const data: StructTag = {
      address: '0x1',
      module: 'ModuleName',
      name: 'StructName',
      type_params: ['Bool', 'U8'],
    };

    const result = structTagToSCS(data);

    expect(result).toBeInstanceOf(rooch_types.StructTag);
    // Add more assertions to check the properties of the result object.
  });
});

describe('addressToSCS', () => {
  it('should convert an AccountAddress to an SCS AccountAddress', () => {
    const addr: AccountAddress = '0x1';

    const result = addressToSCS(addr);

    expect(result).toBeInstanceOf(rooch_types.AccountAddress);
    // Add more assertions to check the properties of the result object.
  });
});

describe('encodeStructTypeTags', () => {
  it('should encode an array of struct type tags correctly', () => {
    const typeArgsString: string[] = ['0x1::ModuleName::StructName<0x2::AnotherModule::AnotherStruct>'];

    const result = encodeStructTypeTags(typeArgsString);

    expect(result).toHaveLength(1);
    expect(result[0]).toHaveProperty('Struct');
    // Add more assertions to check the properties of the result object.
  });
});
