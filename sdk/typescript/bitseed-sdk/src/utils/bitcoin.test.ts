import * as bitcoin from 'bitcoinjs-lib';
import { decodeScriptPubKey, ScriptTypeWitnessV0KeyHash, decodeUTXOs } from './bitcoin';

describe('decodeScriptPubKey', () => {
  const network = bitcoin.networks.testnet;

  it('should decode a valid P2WPKH scriptPubKey', () => {
    const scriptPubKeyHex = '00148fcd888b6817682f90cdbbd7f795316c61f6da65';
    const result = decodeScriptPubKey(scriptPubKeyHex, network);

    expect(result.asm).toContain('OP_0 8fcd888b6817682f90cdbbd7f795316c61f6da65');
    expect(result.address).toBe('tb1q3lxc3zmgza5zlyxdh0tl09f3d3sldkn9ftwu3c')
    expect(result.type).toBe(ScriptTypeWitnessV0KeyHash);
  });

  it('should decode a valid P2TR scriptPubKey', () => {
    const scriptPubKeyHex = '5120114002a1d9df42cd866b715e4477f79c848229be5a3eb83fa57f5831e4af5095';
    const result = decodeScriptPubKey(scriptPubKeyHex, network);

    expect(result.asm).toContain('OP_1 114002a1d9df42cd866b715e4477f79c848229be5a3eb83fa57f5831e4af5095');
    expect(result.address).toBe('')
    expect(result.type).toBe('nonstandard');
  });

  it('should throw an error for an invalid scriptPubKey', () => {
    const invalidScriptPubKeyHex = '002a1d9';

    expect(() => {
      decodeScriptPubKey(invalidScriptPubKeyHex, network);
    }).toThrow();
  });

  it('should return nonstandard for an unknown script type', () => {
    const nonStandardScriptPubKeyHex = '6a';

    const result = decodeScriptPubKey(nonStandardScriptPubKeyHex, network);

    expect(result.type).toBe('nonstandard');
    expect(result.address).toBe('')
  });
});

describe('decodeUTXOs', () => {
  const network = bitcoin.networks.testnet;
  const signedTxHex = '02000000000101569ff8f31348b74ab0bfbdf58830af62672e3bff61e7be9921efef6767a81a430000000000fdffffff01e803000000000000225120114002a1d9df42cd866b715e4477f79c848229be5a3eb83fa57f5831e4af5095034072d8431a1e383f04a021c601880c37d16c9f34269ae775e3045cff38647d486c38bf837f3b39577e556b9920b4ea20d8780d9a5233350d743ec2616da3c2aaed8220e1f462719cc15cccd0fac248b5d478aed358bfe4835bb02c43c38b9a5f635ad8ac0063036f7264552ba4626f706474657374647469636b68746573745469636b66616d6f756e74016a61747472696275746573a0570762697473656564510a746578742f706c61696e010014534756736247387349466476636d786b49513d3d6841c0e1f462719cc15cccd0fac248b5d478aed358bfe4835bb02c43c38b9a5f635ad8b890202e8c5bdceee485a7cca0f2c3f3e8b62866b26abe70036bd8bae6713c9d00000000';

  it('should decode UTXOs for the given address', () => {
    const result = decodeUTXOs(signedTxHex, network);

    const expectedResult = [
      {
        n: 0,
        txid: '7f914e257234675694a4b75986a1dc6a5a3329dcaf506ad70d88399cdf13bc0f',
        sats: 1000,
        scriptPubKey: {
          "address": "",
          "asm": "OP_1 114002a1d9df42cd866b715e4477f79c848229be5a3eb83fa57f5831e4af5095",
          "desc": "Script nonstandard",
          "hex": "5120114002a1d9df42cd866b715e4477f79c848229be5a3eb83fa57f5831e4af5095",
          "type": "nonstandard",
        },
      },
    ];

    expect(result).toEqual(expectedResult);
  });
});
