// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import path from 'path'
import { fileURLToPath } from 'url'

import React from 'react'
import { test, expect } from '@playwright/experimental-ct-react'

import DeployStory from './deploy.story'
import rooch_test from '@roochnetwork/test-suite'
import { createTestBitSeed, prepareTestGenerator } from './commons/test_bitseed_node.js'
import { sleep } from './commons/time'

const { TestBox } = rooch_test
const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

test.use({ viewport: { width: 500, height: 500 } })

var testBox: any = null
let roochServerAddress: string | null
let generatorID: string | null

test.beforeAll(async () => {
  testBox = new TestBox()
  await testBox.loadBitcoinEnv(null, true)
  await testBox.loadRoochEnv('local', 0)
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
  }
})

test.afterAll(async () => {
  await testBox.unloadContainer()
})

test('Deploy move tick with simple', async ({ page, mount }) => {
  if (!roochServerAddress) {
    throw new Error('Failed to get Rooch server address')
  }

  const component = await mount(<DeployStory roochServerAddress={roochServerAddress} />)

  const generatorInscriptionId = `${generatorID}`
  const deployArg = `{"height":{"type":"range","data":{"min":1,"max":1000}}}`

  // Input the InscriptionID
  await component.locator('input[placeholder="Tick"]').fill('move')
  await component.locator('input[placeholder="Max"]').fill('1000')
  await component
    .locator('input[placeholder="GeneratorInscriptionID"]')
    .fill(generatorInscriptionId)
  await component.locator('input[placeholder="DeployArg"]').fill(deployArg)

  // Click the deploy button
  await component.locator('button:has-text("Deploy")').click()

  // Check for the presence of the Rooch server address in the component
  await expect(component).toContainText(`Rooch Server: ${roochServerAddress}`)

  // Optionally, check for the presence of the inscriptionId in the output/result
  await expect(component).toContainText('Deploy Result: ', { timeout: 60000 })
})
