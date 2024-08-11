import React from 'react'
import { test, expect } from '@playwright/experimental-ct-react'
import MintStory from './mint.story'
import { BitseedTestEnv } from './commons/bitseed_test_env'

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

test('mint tick', async ({ mount }) => {
  if (!roochServerAddress) {
    throw new Error('Failed to get Rooch server address');
  }

  const component = await mount(<MintStory roochServerAddress={roochServerAddress} />)

  const moveTickInscriptionId = '75e95eeba0b3450feda8d880efe00600816e5934160a4757fbdaa99a0e3bb436i0'

  // Input the InscriptionID
  await component.locator('input[placeholder="TickDeployID"]').fill(moveTickInscriptionId)
  await component.locator('input[placeholder="UserInput"]').fill('20240306')

  // Click the mint button
  await component.locator('button:has-text("Mint")').click()

  // Optionally, check for the presence of the inscriptionId in the output/result
  await expect(component).toContainText('Mint Result: ', {timeout: 60000 })
})
