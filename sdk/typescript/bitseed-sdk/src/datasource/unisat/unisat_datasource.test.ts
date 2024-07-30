import { UniSatDataSource } from './unisat_datasource'; // 假设你的类名为 YourClass
import { Wallet } from '../../wallet'

describe('UniSatDataSource', () => {
  let instance: UniSatDataSource;
  let wallet: Wallet;

  beforeEach(() => {
    wallet = new Wallet({
      wif: 'cNGdjKojxE7nCcYdK34d12cdYTzBdDV4VdXdbpG7SHGTRWuCxpAW',
      network: "testnet",
      type: 'taproot',
    })

    instance = new UniSatDataSource({network: 'testnet'});
  });

  describe('getBalance', () => {
    it('should return the correct balance for a given address', async () => {
      if (!wallet.selectedAddress) {
        throw new Error('no selected address')
      }
  
      const balance = await instance.getBalance({ address: wallet.selectedAddress });
      expect(balance).toBeDefined()
    });

    it('should return the 0 balance for a new address', async () => {
      const wallet = new Wallet({
        bip39: 'right second until palace kid wear tennis phone bike broccoli oval saddle',
        network: "testnet",
        type: 'taproot',
      })

      const account = wallet.generateAddress('taproot', 2, 0)
      if (!account.address) {
        throw new Error('no selected address')
      }

      const balance = await instance.getBalance({ address: account.address });
      expect(balance).toBe(0.0);
    });
  });

  describe('getInscriptionUTXO', () => {
    it('should return the correct UTXO for getInscriptionUTXO', async () => {
      const utxo = await instance.getInscriptionUTXO({ id: '42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5i1' });

      expect(utxo).toBeDefined()
      expect(utxo.n).toBe(0)
      expect(utxo.txid).toBe('42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5')
      expect(utxo.sats).toBe(600)
      expect(utxo.safeToSpend).toBeFalsy()
      expect(utxo.confirmation).toBe(-1)
    });
  });

  describe('getInscription', () => {
    jest.setTimeout(20000)
    
    it('should return the correct Inscription for getInscriptionUTXO', async () => {
      const inscription = await instance.getInscription({ id: '42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5i0', decodeMetadata: true });

      expect(inscription).toBeDefined()
      expect(inscription.id).toBe('42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5i0')
      expect(inscription.outpoint).toBe('42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5:0')
      expect(inscription.owner).toBe('tb1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2sh3ew0h')
      expect(inscription.genesis).toBe('42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5')
      expect(inscription.fee).toBe(-1)
      expect(inscription.height).toBe(2578904)
      expect(inscription.number).toBe(1317777)
      expect(inscription.sat).toBe(600)
      expect(inscription.timestamp).toBe(1708478409)
      expect(inscription.mediaType).toBe('application/json')
      expect(inscription.mediaSize).toBe(102)
      expect(inscription.mediaContent).toBe('tickmovemax1000generator/content/dd1f515b828eedabd6b0be147cf611ca08c20f39058feee9b96efaa2eba43d9di0repeat0has/user/inputfalsedeploy/args')
      expect(inscription.value).toBe(600)
      expect(inscription.meta).toStrictEqual({})
    });

    it('should return the correct Inscription for getInscription for wasm', async () => {
      const inscription = await instance.getInscription({ id: '33485f62248720b7faf7f24b1236a77e444fea0f0b67dee17461fdf8029cb7b8i0', decodeMetadata: true });

      expect(inscription).toBeDefined()
      expect(inscription.id).toBe('33485f62248720b7faf7f24b1236a77e444fea0f0b67dee17461fdf8029cb7b8i0')
      expect(inscription.outpoint).toBe('33485f62248720b7faf7f24b1236a77e444fea0f0b67dee17461fdf8029cb7b8:0')
      expect(inscription.owner).toBe('tb1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2sh3ew0h')
      expect(inscription.genesis).toBe('33485f62248720b7faf7f24b1236a77e444fea0f0b67dee17461fdf8029cb7b8')
      expect(inscription.fee).toBe(-1)
      expect(inscription.height).toBe(2580221)
      expect(inscription.number).toBe(2058307)
      expect(inscription.sat).toBe(1000)
      expect(inscription.timestamp).toBe(1709392826)
      expect(inscription.mediaType).toBe('application/wasm')
      expect(inscription.mediaSize).toBe(101)
      expect(inscription.mediaContent).toBe('AGFzbQEAAAABCAJgAX8AYAAAAg8BB2NvbnNvbGUDbG9nAAADAgEBBQMBAAEHFwIGbWVtb3J5AgAKaGVsbG9Xb3JsZAABCggBBgBBABAACwsUAQBBAAsOSGVsbG8sIFdvcmxkIQA=')
      expect(inscription.value).toBe(1000)
      expect(inscription.meta).toStrictEqual({
        op: 'mint',
        tick: 'generator',
        amount: 1,
        attributes: undefined
      })
    });

    it('should return the correct Inscription for getInscription for generator wasm', async () => {
      const inscription = await instance.getInscription({ id: '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0', decodeMetadata: true });
      console.log('generator inscription:', inscription)
      expect(inscription).toBeDefined()
    });

    it('should return the correct Inscription for getInscription for move tick', async () => {
      const inscription = await instance.getInscription({ id: '75e95eeba0b3450feda8d880efe00600816e5934160a4757fbdaa99a0e3bb436i0', decodeMetadata: true });
      console.log('move tick inscription:', inscription)
      expect(inscription).toBeDefined()
    });
  });
  
  describe('getInscriptions', () => {
    jest.setTimeout(20000)
    
    it('should return the correct Inscriptions for getInscriptions by owner', async () => {
      const inscriptions = await instance.getInscriptions({ owner: 'tb1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2sh3ew0h' });
      expect(inscriptions).toBeDefined()
      expect(inscriptions.length).toBeGreaterThan(0);
    });
  });

  describe('getTransaction', () => {
    jest.setTimeout(20000)
    
    it('should return the correct TX for getTransaction', async () => {
      const resp = await instance.getTransaction({ txId: '42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5' });
      console.log("resp:", resp)
      expect(resp).toBeDefined()
    });

    it('should return the raw TX for getTransaction', async () => {
      const resp = await instance.getTransaction({ 
        txId: '42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5',
        ordinals: true,
        hex: true,
        witness: true,
        decodeMetadata: true,
      });
      console.log("raw resp:", resp)
      expect(resp).toBeDefined()
    });
  });

  describe('getSpendables', () => {
    jest.setTimeout(20000)
    
    it('should return the UTXSs for getSpendables', async () => {
      const utxos = await instance.getSpendables({ address: 'tb1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrs7m68qv', value: 100 });
      console.log('spendable utxos:', utxos)
      expect(utxos).toBeDefined()
    });
  });

  describe('getUnspents', () => {
    jest.setTimeout(20000)
    
    it('should return the UTXSs for getUnspents', async () => {
      const resp = await instance.getUnspents({ address: 'tb1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrs7m68qv' });
      expect(resp).toBeDefined()
    });
  });

})
