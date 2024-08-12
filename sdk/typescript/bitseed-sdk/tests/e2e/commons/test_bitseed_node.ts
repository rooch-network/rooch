import * as fs from 'fs';
import { Ordit } from '@sadoprotocol/ordit-sdk'
import { BitSeed, GeneratorLoader, RoochDataSource, DeployOptions, inscriptionIDToString } from '../../../src'

export function createTestBitSeed(roochServerAddress: string): BitSeed {
  const network = 'regtest'
  const datasource = new RoochDataSource({ url: `http://${roochServerAddress}` })
  const generatorLoader = new GeneratorLoader(datasource)

  // address: tb1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2sh3ew0h
  const primaryWallet = new Ordit({
    wif: 'cNGdjKojxE7nCcYdK34d12cdYTzBdDV4VdXdbpG7SHGTRWuCxpAW',
    network,
    type: 'taproot',
  })

  // address: tb1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrs7m68qv
  const fundingWallet = new Ordit({
    wif: 'cNfgnR9UB1garDrQ3WVaQ2LbG4CPxpuEepor44yyuiB8wtSa3Bta',
    network,
    type: 'taproot',
  })

  /*
  const fundingWallet = new Ordit({
    wif: 'cTW1Q2A8AVBuJ1sEBoV9gWokc6e5NYFPHxez6hhriVL2jKH6bfct',
    network,
    type: 'taproot',
  })
  */

  console.log('primary wallet address:', primaryWallet.selectedAddress)
  console.log('funding wallet address:', fundingWallet.selectedAddress)

  const bitseed = new BitSeed(primaryWallet, fundingWallet, datasource, generatorLoader)

  return bitseed
}

export async function prepareGenerator(bitseed: BitSeed, filePath: string): Promise<string> {
  let wasmBytes = await readFileAsBytes(filePath)
  console.log('wasm length:', wasmBytes.length)

  const deployOptions: DeployOptions = {
    fee_rate: 1,
  }

  const inscriptionId = await bitseed.generator("simple", wasmBytes, deployOptions)
  return inscriptionIDToString(inscriptionId)
}

const readFileAsBytes = (filePath: string): Promise<Uint8Array> => {
  return new Promise((resolve, reject) => {
    fs.readFile(filePath, (err, data) => {
      if (err) {
        reject(err);
      } else {
        resolve(new Uint8Array(data));
      }
    });
  });
};
