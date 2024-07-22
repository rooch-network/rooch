import { Ordit } from '@sadoprotocol/ordit-sdk'
import { BitSeed, GeneratorLoader, UniSatDataSource } from '../../../src'

const network = 'testnet'
const datasource = new UniSatDataSource({ network })
const generatorLoader = new GeneratorLoader(datasource)

export function createTestBitSeed(): BitSeed {
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
