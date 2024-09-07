// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import debug from 'debug'
import * as fs from 'fs'
import { Ordit } from '@sadoprotocol/ordit-sdk'
import bitseed from '../../../dist/cjs/index.js'
import { HTTPDebugTransport } from './http_debug_transport.js'
const { BitSeed, GeneratorLoader, RoochDataSource, inscriptionIDToString, parseInscriptionID } =
  bitseed

const log = debug('bitseed:e2e:test_bitseed_node')

export function createTestBitSeed(roochServerAddress: string) {
  const network = 'regtest'
  const datasource = new RoochDataSource({
    transport: new HTTPDebugTransport({ url: `http://${roochServerAddress}` }, false),
  })

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

export async function prepareTestGenerator(bitseed, filePath: string): Promise<string> {
  let wasmBytes = await readFileAsBytes(filePath)
  log('wasm length:', wasmBytes.length)

  const deployOptions = {
    fee_rate: 1,
  }

  const inscriptionId = await bitseed.generator('simple', wasmBytes, deployOptions)
  log('prepareGenerator inscriptionId:', inscriptionId)

  return inscriptionIDToString(inscriptionId)
}

export async function deployTestTick(bitseed, generatorID, tick, max, deployArg) {
  let generator = parseInscriptionID(generatorID)
  const deployArgs = [deployArg]

  const deployOptions = {
    fee_rate: 1,
    repeat: 1,
    deploy_args: deployArgs,
  }

  const inscriptionId = await bitseed.deploy(tick, max, generator, deployOptions)

  return inscriptionIDToString(inscriptionId)
}

const readFileAsBytes = (filePath: string): Promise<Uint8Array> => {
  return new Promise((resolve, reject) => {
    fs.readFile(filePath, (err, data) => {
      if (err) {
        reject(err)
      } else {
        resolve(new Uint8Array(data))
      }
    })
  })
}
