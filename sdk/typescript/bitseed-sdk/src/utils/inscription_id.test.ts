import { InscriptionID } from '../types'
import { parseInscriptionID, inscriptionIDToString, extractInscription } from './inscription_id';

describe('parseInscriptionID', () => {
  it('should correctly parse a valid InscriptionID string', () => {
    const validID = 'c75299ecf9787076e276271384e55c08b5dbbc187917a59a76cdf340e4aa0734i0';
    const expected: InscriptionID = {
      txid: 'c75299ecf9787076e276271384e55c08b5dbbc187917a59a76cdf340e4aa0734',
      index: 0
    };
    expect(parseInscriptionID(validID)).toEqual(expected);
  });

  it('should throw an error for an invalid InscriptionID string', () => {
    const invalidID = 'invalidInscriptionID';
    expect(() => parseInscriptionID(invalidID)).toThrow('Invalid InscriptionID format');
  });

  it('should throw an error for an InscriptionID string with no index', () => {
    const noIndexID = 'c75299ecf9787076e276271384e55c08b5dbbc187917a59a76cdf340e4aa0734';
    expect(() => parseInscriptionID(noIndexID)).toThrow('Invalid InscriptionID format');
  });

  it('should throw an error for an InscriptionID string with an invalid index', () => {
    const invalidIndexID = 'c75299ecf9787076e276271384e55c08b5dbbc187917a59a76cdf340e4aa0734iNaN';
    expect(() => parseInscriptionID(invalidIndexID)).toThrow('Invalid InscriptionID format');
  });

});

describe('inscriptionIDToString', () => {
  it('should correctly convert an InscriptionID object to a string', () => {
    const inscriptionID: InscriptionID = {
      txid: 'c75299ecf9787076e276271384e55c08b5dbbc187917a59a76cdf340e4aa0734',
      index: 0
    };
    const expectedString = 'c75299ecf9787076e276271384e55c08b5dbbc187917a59a76cdf340e4aa0734i0';
    expect(inscriptionIDToString(inscriptionID)).toBe(expectedString);
  });

});

describe('extractInscription', () => {
  it('should extract the hash from a well-formed input string', () => {
    const input = '/inscription/6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0';
    const expected = '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0';
    expect(extractInscription(input)).toBe(expected);
  });

  it('should return null if the prefix is incorrect', () => {
    const input = '/wrongprefix/6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0';
    expect(extractInscription(input)).toBeNull();
  });

  it('should return null if the hash is missing', () => {
    const input = '/inscription/';
    expect(extractInscription(input)).toBeNull();
  });

  it('should return null for an empty string', () => {
    const input = '';
    expect(extractInscription(input)).toBeNull();
  });

  it('should handle strings with additional characters after the hash', () => {
    const input = '/inscription/6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0/somethingelse';
    const expected = '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0';
    expect(extractInscription(input)).toBe(expected);
  });

  it('should return null if the hash contains invalid characters', () => {
    const input = '/inscription/6f55475ce65054G8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8';
    expect(extractInscription(input)).toBeNull();
  });

});