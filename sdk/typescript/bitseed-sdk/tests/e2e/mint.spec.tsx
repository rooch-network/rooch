// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import path from 'path'
import { fileURLToPath } from 'url'

import React from 'react'
import { test, expect } from '@playwright/experimental-ct-react'
import MintStory from './mint.story'
import rooch_test from '@roochnetwork/test-suite'
import {
  createTestBitSeed,
  prepareTestGenerator,
  deployTestTick,
} from './commons/test_bitseed_node.js'
import { sleep } from './commons/time'

const { TestBox } = rooch_test
const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

test.use({ viewport: { width: 500, height: 500 } })

var testBox: any = null
let roochServerAddress: string | null
let generatorID: string | null
let moveTickInscriptionId: string | null

test.beforeAll(async () => {
  testBox = new TestBox();
  await testBox.loadBitcoinEnv()
  await testBox.loadRoochEnv("local", 0)
  roochServerAddress = testBox.getRoochServerAddress()

  await testBox.getFaucetBTC('bcrt1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2s6gng6d', 1)
  await testBox.getFaucetBTC('bcrt1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrsnzsp4k', 1)

  await sleep(10000)

  if (roochServerAddress) {
    let bitseed = createTestBitSeed(roochServerAddress)
    generatorID = await prepareTestGenerator(
      bitseed,
      path.join(__dirname, '../data/generator.wasm'),
    )
    await sleep(10000)

    const deployArg = `{"height":{"type":"range","data":{"min":1,"max":1000}}}`
    moveTickInscriptionId = await deployTestTick(bitseed, generatorID, 'move', 1000, deployArg)
    await sleep(10000)
  }
})

test.afterAll(async () => {
  await testBox.unloadContainer()
})

test('mint tick', async ({ mount }) => {
  if (!roochServerAddress) {
    throw new Error('Failed to get Rooch server address')
  }

  if (!moveTickInscriptionId) {
    throw new Error('Failed to get moveTickInscriptionId')
  }

  const component = await mount(<MintStory roochServerAddress={roochServerAddress} />)

  // Input the InscriptionID
  await component.locator('input[placeholder="TickDeployID"]').fill(moveTickInscriptionId)
  await component.locator('input[placeholder="UserInput"]').fill('20240306')

  // Click the mint button
  await component.locator('button:has-text("Mint")').click()

  // Optionally, check for the presence of the inscriptionId in the output/result
  await expect(component).toContainText('Mint Result: ', { timeout: 60000 })
})
