import { sha3_256 } from 'js-sha3';
import { InscribeSeed } from './seed';

describe('InscribeSeed', () => {
  it('should generate the correct seed', () => {
    // Mock data
    const block_hash = 'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890';
    const utxo = {
      txid: '1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
      vout: 1
    };

    // Expected seed
    const expectedSeed = sha3_256(Buffer.concat([
      Buffer.from(block_hash, 'hex'),
      Buffer.from(utxo.txid, 'hex'),
      Buffer.from([1, 0, 0, 0]) // vout is 1, little-endian format
    ]));

    // Instantiate InscribeSeed
    const inscribeSeed = new InscribeSeed(block_hash, utxo);

    // Generate seed
    const seed = inscribeSeed.seed();

    // Assert the generated seed is as expected
    expect(seed).toBe(expectedSeed);
  });
});
