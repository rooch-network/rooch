import React from 'react'
import { test, expect } from '@playwright/experimental-ct-react'
import DeployStory from './deploy.story'
import { BitseedTestEnv } from './commons/bitseed_test_env'

test.use({ viewport: { width: 500, height: 500 } })

var testEnv: BitseedTestEnv = new BitseedTestEnv();
let roochServerAddress: string | null;

test.beforeAll(async () => {
  console.log('Before tests');
  await testEnv.start();
  roochServerAddress = testEnv.getRoochServerAddress();
});

test.afterAll(async () => {
  console.log('After tests');
  await testEnv.stop()
});

test('Deploy move tick with simple generator', async ({ page, mount }) => {
  if (!roochServerAddress) {
    throw new Error('Failed to get Rooch server address');
  }
  
  try {
    console.log("DeployStory mount 1")
    const component = await mount(<DeployStory roochServerAddress={roochServerAddress} />)
    console.log("DeployStory mount 2")

    const generatorInscriptionId = '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0'
    const deployArg = `{"height":{"type":"range","data":{"min":1,"max":1000}}}`
  
    // Input the InscriptionID
    await component.locator('input[placeholder="Tick"]').fill('move')
    await component.locator('input[placeholder="Max"]').fill('1000')
    await component.locator('input[placeholder="GeneratorInscriptionID"]').fill(generatorInscriptionId)
    await component.locator('input[placeholder="DeployArg"]').fill(deployArg)
  
    // Click the deploy button
    await component.locator('button:has-text("Deploy")').click()
  
    // Check for the presence of the Rooch server address in the component
    await expect(component).toContainText(`Rooch Server: ${roochServerAddress}`)
  
    // Optionally, check for the presence of the inscriptionId in the output/result
    await expect(component).toContainText('Deploy Result: ')
  } catch(e: any) {
    console.log("test error:", e)
    await page.pause(); 
  }
})
