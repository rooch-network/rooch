// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import React from 'react'

import path from 'path'
import { fileURLToPath } from 'url'
import { test, expect } from '@playwright/experimental-ct-react'
import DeployGeneratorStory from './generator.story'
import rooch_test from '@roochnetwork/test-suite'
import { sleep } from './commons/time'

const { TestBox } = rooch_test
const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

test.use({ viewport: { width: 500, height: 500 } })

var testBox: any = null
let roochServerAddress: string | null

test.beforeAll(async () => {
  testBox = new TestBox()
  await testBox.loadBitcoinEnv(null, true)
  await testBox.loadRoochEnv('local', 0)

  roochServerAddress = testBox.getRoochServerAddress()

  await testBox.getFaucetBTC('bcrt1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2s6gng6d', 1)
  await testBox.getFaucetBTC('bcrt1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrsnzsp4k', 1)

  await sleep(10000)
})

test.afterAll(async () => {
  await testBox.unloadContainer()
})

test('Upload generator', async ({ mount }) => {
  if (!roochServerAddress) {
    throw new Error('Failed to get Rooch server address')
  }

  const component = await mount(<DeployGeneratorStory roochServerAddress={roochServerAddress} />)

  // Upload generator wasm file
  await component
    .locator('input[placeholder="wasmFile"]')
    .setInputFiles(path.join(__dirname, '../data/generator.wasm'))

  // Click the deploy button
  await component.locator('button:has-text("Deploy")').click()

  // Optionally, check for the presence of the inscriptionId in the output/result
  await expect(component).toContainText('Deploy Result: ', { timeout: 60000 })
})

test('Upload invalid generator', async ({ mount }) => {
  if (!roochServerAddress) {
    throw new Error('Failed to get Rooch server address')
  }

  const component = await mount(<DeployGeneratorStory roochServerAddress={roochServerAddress} />)

  // Upload generator wasm file
  await component
    .locator('input[placeholder="wasmFile"]')
    .setInputFiles(path.join(__dirname, '../data/invalid-generator.wasm'))

  // Click the deploy button
  await component.locator('button:has-text("Deploy")').click()

  // Optionally, check for the presence of the inscriptionId in the output/result
  await expect(component).toContainText('Error:', { timeout: 60000 })
})
