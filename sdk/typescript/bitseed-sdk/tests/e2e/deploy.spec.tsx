import React from 'react'
import { test, expect } from '@playwright/experimental-ct-react'
import DeployStory from './deploy.story'

test.use({ viewport: { width: 500, height: 500 } })

test('Deploy move tick with simple generator', async ({ mount }) => {
  const component = await mount(<DeployStory />)

  const generatorInscriptionId = '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0'
  const deployArg = `{"height":{"type":"range","data":{"min":1,"max":1000}}}`

  // Input the InscriptionID
  await component.locator('input[placeholder="Tick"]').fill('move')
  await component.locator('input[placeholder="Max"]').fill('1000')
  await component.locator('input[placeholder="GeneratorInscriptionID"]').fill(generatorInscriptionId)
  await component.locator('input[placeholder="DeployArg"]').fill(deployArg)

  // Click the deploy button
  await component.locator('button:has-text("Deploy")').click()

  // Optionally, check for the presence of the inscriptionId in the output/result
  await expect(component).toContainText('Deploy Result: ')
})
