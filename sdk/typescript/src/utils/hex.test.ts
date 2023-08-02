import { describe, it, expect } from "vitest"
import { toHexString, fromHexString, padLeft, padRight } from './hex';

describe('toHexString', () => {
    it('should convert an empty array to "0x"', () => {
      const byteArray: number[] = [];
      expect(toHexString(byteArray)).toBe('0x');
    });
  
    it('should convert a single-byte array to a hex string', () => {
      const byteArray: number[] = [255];
      expect(toHexString(byteArray)).toBe('0xff');
    });
  
    it('should convert a multi-byte array to a hex string', () => {
      const byteArray: number[] = [16, 32, 64, 128];
      expect(toHexString(byteArray)).toBe('0x10204080');
    });
  
    it('should handle non-integer values by truncating the decimal part', () => {
      const byteArray: number[] = [1.5, 2.9, 3.1];
      expect(toHexString(byteArray)).toBe('0x010203');
    });
  
    it('should handle negative values by taking the two complement', () => {
      const byteArray: number[] = [-1, -128];
      expect(toHexString(byteArray)).toBe('0xff80');
    });
});

describe('fromHexString', () => {
  it('should convert a hex string without "0x" prefix to a Uint8Array', () => {
    const hex = '10204080';
    expect(fromHexString(hex)).toEqual(new Uint8Array([16, 32, 64, 128]));
  });

  it('should convert a hex string with "0x" prefix to a Uint8Array', () => {
    const hex = '0x10204080';
    expect(fromHexString(hex)).toEqual(new Uint8Array([16, 32, 64, 128]));
  });

  it('should handle odd-length hex strings by adding a leading zero', () => {
    const hex = '12345';
    expect(fromHexString(hex)).toEqual(new Uint8Array([1, 35, 69]));
  });

  it('should pad the result with zeros when padding is specified', () => {
    const hex = '1234';
    expect(fromHexString(hex, 6)).toEqual(new Uint8Array([0, 18, 52]));
  });
});

describe('padLeft', () => {
  it('should pad a string on the left with zeros', () => {
    const str = '123';
    expect(padLeft(str, 5)).toBe('00123');
  });

  it('should pad a string on the left with a custom character', () => {
    const str = '123';
    expect(padLeft(str, 5, 'X')).toBe('XX123');
  });

  it('should not modify the string if its length is already equal to or greater than the specified length', () => {
    const str = '12345';
    expect(padLeft(str, 6)).toBe('012345');
  });
});

describe('padRight', () => {
  it('should pad a string on the right with zeros', () => {
    const str = '123';
    expect(padRight(str, 5)).toBe('12300');
  });

  it('should pad a string on the right with a custom character', () => {
    const str = '123';
    expect(padRight(str, 5, 'X')).toBe('123XX');
  });

  it('should not modify the string if its length is already equal to or greater than the specified length', () => {
    const str = '12345';
    expect(padRight(str, 6)).toBe('123450');
  });
});
