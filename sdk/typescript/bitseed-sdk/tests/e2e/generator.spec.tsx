import React from 'react'

import path from 'path'
import { fileURLToPath } from 'url';
import { test, expect } from '@playwright/experimental-ct-react'
import DeployGeneratorStory from './generator.story'
import { BitseedTestEnv } from './commons/bitseed_test_env'

const __dirname = path.dirname(fileURLToPath(import.meta.url));
test.use({ viewport: { width: 500, height: 500 } })

var testEnv: BitseedTestEnv = new BitseedTestEnv();
let roochServerAddress: string | null;

test.beforeAll(async () => {
  console.log('Before tests');
  await testEnv.start();
  roochServerAddress = testEnv.getRoochServerAddress();

  await testEnv.getFaucetBTC("bcrt1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2s6gng6d", 1)
  await testEnv.getFaucetBTC("bcrt1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrsnzsp4k", 1)
});

test.afterAll(async () => {
  console.log('After tests');
  await testEnv.stop()
});

test('Upload generator', async ({ mount }) => {
  if (!roochServerAddress) {
    throw new Error('Failed to get Rooch server address');
  }

  const component = await mount(<DeployGeneratorStory roochServerAddress={roochServerAddress} />)

  // Upload generator wasm file
  await component
    .locator('input[placeholder="wasmFile"]')
    .setInputFiles(path.join(__dirname, '../data/generator.wasm'))

  // Click the deploy button
  await component.locator('button:has-text("Deploy")').click()

  // Optionally, check for the presence of the inscriptionId in the output/result
  await expect(component).toContainText('Deploy Result: ')
})

test('Upload invalid generator', async ({ mount }) => {
  if (!roochServerAddress) {
    throw new Error('Failed to get Rooch server address');
  }

  const component = await mount(<DeployGeneratorStory roochServerAddress={roochServerAddress} />)

  // Upload generator wasm file
  await component
    .locator('input[placeholder="wasmFile"]')
    .setInputFiles(path.join(__dirname, '../data/invalid-generator.wasm'))

  // Click the deploy button
  await component.locator('button:has-text("Deploy")').click()

  // Optionally, check for the presence of the inscriptionId in the output/result
  await expect(component).toContainText('Deploy Result: ')
})
