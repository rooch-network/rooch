// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import debug from 'debug'
import { Ordit } from '@sadoprotocol/ordit-sdk'
import { BitSeed, GeneratorLoader, RoochDataSource } from '../../../dist/esm'

const log = debug('bitseed:e2e:test_bitseed_web')

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

  log('primary wallet address:', primaryWallet.selectedAddress)
  log('funding wallet address:', fundingWallet.selectedAddress)

  const bitseed = new BitSeed(primaryWallet, fundingWallet, datasource, generatorLoader)

  return bitseed
}
