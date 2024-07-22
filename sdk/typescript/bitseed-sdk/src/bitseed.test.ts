import path from 'path'
import fs from 'fs';
import * as bitcoin from 'bitcoinjs-lib'
import { Transaction as BTCTransaction } from "bitcoinjs-lib";
import { BitSeed } from './bitseed';
import {
  Ordit,
  IDatasource,
  RelayOptions,
  GetSpendablesOptions,
  UTXOLimited,
  GetInscriptionOptions,
  Inscription,
  GetTransactionOptions,
  Transaction
} from '@sadoprotocol/ordit-sdk';
import { IGeneratorLoader, GeneratorLoader } from './generator';
import { SFTRecord, InscriptionID } from './types';
import { InscribeOptions, DeployOptions } from './interfaces'
import { toB64 } from './utils';

const networkType = 'testnet'

const loadWasmBytesFromFile = (url: string) => {
  const filePath = path.resolve(url);
  const fileBuffer = fs.readFileSync(filePath);
  return new Uint8Array(fileBuffer)
}

describe('BitSeed', () => {
  const mempool = new Map<string, UTXOLimited[]>();
  const inscriptionStore = new Map<string, Inscription>();
  const txStore = new Map<string, Transaction>();

  let primaryWallet: Ordit;
  let fundingWallet: Ordit;
  let datasourceMock: jest.Mocked<IDatasource>;
  let generatorLoaderMock: IGeneratorLoader;
  let bitSeed: BitSeed;

  beforeEach(() => {
    // address: tb1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2sh3ew0h
    primaryWallet = new Ordit({
      wif: 'cNGdjKojxE7nCcYdK34d12cdYTzBdDV4VdXdbpG7SHGTRWuCxpAW',
      network: networkType,
      type: 'taproot',
    })

    // address: tb1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrs7m68qv
    fundingWallet = new Ordit({
      wif: 'cNfgnR9UB1garDrQ3WVaQ2LbG4CPxpuEepor44yyuiB8wtSa3Bta',
      network: networkType,
      type: 'taproot',
    })

    mempool.set('tb1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrs7m68qv', [
      {
        n: 1,
        txid: '9e71d06045d5d677799a70647c9e5484b232aa684b73334038a447c044dc24cd',
        sats: 45465,
        scriptPubKey: {
          asm: 'OP_1 b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07',
          desc: 'Script witness_v1_taproot',
          hex: '5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07',
          address: 'tb1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrs7m68qv',
          type: 'witness_v1_taproot'
        }
      },
      {
        n: 0,
        txid: '814ccf5c6de163a83081a0f51c42b1c436a2cc8c3303b5d25f91b6efbd50ef3b',
        sats: 10000,
        scriptPubKey: {
          asm: 'OP_1 b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07',
          desc: 'Script witness_v1_taproot',
          hex: '5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07',
          address: 'tb1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrs7m68qv',
          type: 'witness_v1_taproot'
        }
      }
    ])

    // set simple generator inscription
    let wasmBytes = loadWasmBytesFromFile(path.resolve(__dirname, '../tests/data/generator.wasm'))
    inscriptionStore.set('6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0', {
      id: '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0',
      outpoint: '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8:0',
      owner: 'tb1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2sh3ew0h',
      genesis: '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8',
      fee: -1,
      height: 2580530,
      number: 2174609,
      sat: 600,
      timestamp: 1709590640,
      mediaType: 'application/wasm',
      mediaSize: 45904,
      mediaContent: toB64(wasmBytes),
      value: 600,
      meta: {}
    });

    // set move tick inscription
    inscriptionStore.set('75e95eeba0b3450feda8d880efe00600816e5934160a4757fbdaa99a0e3bb436i0', {
      id: '75e95eeba0b3450feda8d880efe00600816e5934160a4757fbdaa99a0e3bb436i0',
      outpoint: '75e95eeba0b3450feda8d880efe00600816e5934160a4757fbdaa99a0e3bb436:0',
      owner: 'tb1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2sh3ew0h',
      genesis: '75e95eeba0b3450feda8d880efe00600816e5934160a4757fbdaa99a0e3bb436',
      fee: -1,
      height: 2580531,
      number: 2174829,
      sat: 600,
      timestamp: 1709591745,
      mediaType: '',
      mediaSize: 0,
      mediaContent: '',
      value: 600,
      meta: {
        op: 'deploy',
        tick: 'move',
        amount: 1000,
        attributes: {
          repeat: 1,
          generator: '/inscription/6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0',
          deploy_args: [`{"height":{"type":"range","data":{"min":1,"max":1000}}}`]
        }
      }
    });

    txStore.set('42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5', {
      txid: '42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5',
      hash: '1722ca7fe748b507cea2bb6fe7ac14996407584fcaf76c5c34dfb38fa2d94efc',
      version: 2,
      size: 438,
      vsize: 180,
      weight: 720,
      locktime: 0,
      vin: [],
      vout: [],
      hex: '02000000000101415767e9a8aa3dbc9a200897c2b553ab4b5b6d4480f8e5485f0ced6e4b9d82130000000000fdffffff015802000000000000225120114002a1d9df42cd866b715e4477f79c848229be5a3eb83fa57f5831e4af509503405a4f4b80c92bf91c486017464cae0fe09e7238a0b0cdcd5f6eb7a6033b64866de5ac4e97cdf7f3952b2d4cb5649a5d33749fb6a36c006d462847183a633706c0d120e1f462719cc15cccd0fac248b5d478aed358bfe4835bb02c43c38b9a5f635ad8ac0063036f72640101106170706c69636174696f6e2f6a736f6e004c66b627249a8bde99ac75d34d207a77ab6ada2bfdca27b5e9edfdd7757f9d796fcdbc79e75a6dde9bd1b7b5e3b71feb5d5c6b4f1cdb47f7f74e7c7de79ef5bf7a79f69ad9e6dae3777d762d2b7a979ab7485ab3fbac7abfe29e9bad7da96c79d7a9968cbf6ab82c680063036f726401011e6170706c69636174696f6e2f6a736f6e3b636861727365743d7574662d3800027b7d6841c1e1f462719cc15cccd0fac248b5d478aed358bfe4835bb02c43c38b9a5f635ad8b890202e8c5bdceee485a7cca0f2c3f3e8b62866b26abe70036bd8bae6713c9d00000000',
      blockhash: '00000000000000265bfdc4e465fda6f12248f13ad26bf22976931ef46589d8c6',
      confirmations: 1776,
      time: 1708478409,
      blocktime: 1708478409,
      fee: 0.0000018099999999999996,
      blockheight: 2578904
    })

    datasourceMock = {
      getBalance: jest.fn(),
      getInscription: jest.fn(),
      getInscriptionUTXO: jest.fn(),
      getInscriptions: jest.fn(),
      getTransaction: jest.fn(),
      getSpendables: jest.fn(),
      getUnspents: jest.fn(),
      relay: jest.fn()
    };

    generatorLoaderMock = new GeneratorLoader(datasourceMock)

    bitSeed = new BitSeed(
      primaryWallet,
      fundingWallet,
      datasourceMock,
      generatorLoaderMock
    );

    datasourceMock.getSpendables.mockImplementation(async function (opts: GetSpendablesOptions): Promise<UTXOLimited[]> {
      let utxos = (mempool.get(opts.address) || new Array<UTXOLimited>()).
        filter((utxo) => utxo.sats >= opts.value)

      return new Promise<UTXOLimited[]>(function (resolve) {
        resolve(utxos)
      })
    })

    datasourceMock.relay.mockImplementation(async function ({ hex }: RelayOptions): Promise<string> {
      const tx = bitcoin.Transaction.fromHex(hex)
      const txid = tx.getId()

      return new Promise<string>(function (resolve) {
        setTimeout(() => {
          resolve(txid)
        }, 10)
      })
    })
  });

  describe('inscribe method', () => {
    it('should throw an error if no address is selected in the primary wallet', async () => {
      primaryWallet.selectedAddress = undefined;

      const sftRecord: SFTRecord = {
        op: 'test',
        tick: 'testTick',
        amount: 1,
        attributes: {}
      };

      await expect(bitSeed.inscribe(sftRecord)).rejects.toThrow('not selected address');
    });


    it('should deposit reveal fee and inscribe successfully', async () => {
      function stringBody(str: string) {
        const encoder = new TextEncoder();
        return encoder.encode(str);
      }

      const sftRecord: SFTRecord = {
        op: 'test',
        tick: 'testTick',
        amount: 1,
        attributes: {},
        content: {
          content_type: 'text/plain',
          body: stringBody('Hello, World!')
        }
      };

      const inscriptionID: InscriptionID = await bitSeed.inscribe(sftRecord);

      expect(inscriptionID).toHaveProperty('txid');
      expect(inscriptionID.index).toEqual(0);
      expect(datasourceMock.relay).toHaveBeenCalledTimes(2);
    });
  });

  describe('generator method', () => {


    it('should be ok when mint invalid-generator.wasm', async () => {
      let wasmBytes = loadWasmBytesFromFile(path.resolve(__dirname, '../tests/data/invalid-generator.wasm'))
      console.log('wasm length:', wasmBytes.length)

      const inscribeOptions: InscribeOptions = {
        fee_rate: 1,
      }

      const inscriptionID = await bitSeed.generator("simple", wasmBytes, inscribeOptions)

      expect(inscriptionID).toHaveProperty('txid');
      expect(inscriptionID.index).toEqual(0);
      expect(datasourceMock.relay).toHaveBeenCalledTimes(2);
    });

    it('should be ok when mint generator.wasm', async () => {
      let wasmBytes = loadWasmBytesFromFile(path.resolve(__dirname, '../tests/data/generator.wasm'))
      console.log('wasm length:', wasmBytes.length)

      const inscribeOptions: InscribeOptions = {
        fee_rate: 1,
      }

      const inscriptionID = await bitSeed.generator("simple", wasmBytes, inscribeOptions)

      expect(inscriptionID).toHaveProperty('txid');
      expect(inscriptionID.index).toEqual(0);
      expect(datasourceMock.relay).toHaveBeenCalledTimes(2);
    });
  });

  describe('deploy method', () => {
    it('deploy move tick should be ok', async () => {
      const tick = 'move';
      const max = 1000;
      const generator = {
        txid: '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8',
        index: 0,
      }

      const deployArgs = [
        '{"height":{"type":"range","data":{"min":1,"max":1000}}}'
      ];

      const deployOptions: DeployOptions = {
        fee_rate: 1,
        repeat: 1,
        deploy_args: deployArgs,
      }

      const inscriptionID = await bitSeed.deploy(tick, max, generator, deployOptions)

      expect(inscriptionID).toHaveProperty('txid');
      expect(inscriptionID.index).toEqual(0);
      expect(datasourceMock.relay).toHaveBeenCalledTimes(2);
    });

    it('deploy move tick with multi deploy arg should be ok', async () => {
      const tick = 'move';
      const max = 1000;
      const generator = {
        txid: '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8',
        index: 0,
      }

      const deployArgs = [];

      for (var i=0; i<50; i++) {
        deployArgs.push(`{"level${i}":{"type":"range","data":{"min":1,"max":1000}}}`)
      }

      const deployOptions: DeployOptions = {
        fee_rate: 1,
        repeat: 1,
        deploy_args: deployArgs,
      }

      const inscriptionID = await bitSeed.deploy(tick, max, generator, deployOptions)

      expect(inscriptionID).toHaveProperty('txid');
      expect(inscriptionID.index).toEqual(0);
      expect(datasourceMock.relay).toHaveBeenCalledTimes(2);
    });
  });

  describe('mint method', () => {

    beforeEach(() => {
      datasourceMock.getInscription.mockImplementation(async function ({ id }: GetInscriptionOptions): Promise<Inscription> {
        let inscription = inscriptionStore.get(id)

        return new Promise<Inscription>(function (resolve, reject) {
          if (!inscription) {
            reject('inscription not exists')
            return
          }

          resolve(inscription)
        })
      })

      datasourceMock.getTransaction.mockImplementation(async function ({ txId }: GetTransactionOptions): Promise<{
        tx: Transaction;
        rawTx?: BTCTransaction;
      }> {
        let tx = txStore.get(txId)

        return new Promise<{
          tx: Transaction;
          rawTx?: BTCTransaction;
        }>(function (resolve, reject) {
          if (!tx) {
            reject('inscription not exists')
            return
          }

          resolve({
            tx,
          })
        })
      })
    })

    it('mint move tick should be ok', async () => {
      const tickInscriptionId = {
        txid: '75e95eeba0b3450feda8d880efe00600816e5934160a4757fbdaa99a0e3bb436',
        index: 0,
      }

      const inscribeOptions: InscribeOptions = {
        fee_rate: 1,
        satpoint: {
          outpoint: {
            txid: '42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5',
            vout: 0
          },
          offset: 0
        }
      }

      const inscriptionID = await bitSeed.mint(tickInscriptionId, 'xxxx', inscribeOptions)

      expect(inscriptionID).toHaveProperty('txid');
      expect(inscriptionID.index).toEqual(0);
      expect(datasourceMock.relay).toHaveBeenCalledTimes(2);
    });
  });
});

