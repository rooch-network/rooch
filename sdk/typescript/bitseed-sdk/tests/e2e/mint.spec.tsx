import React from 'react'
import { test, expect } from '@playwright/experimental-ct-react'
import MintStory from './mint.story'

test.use({ viewport: { width: 500, height: 500 } })

test('mint tick', async ({ mount }) => {
  const component = await mount(<MintStory />)

  const moveTickInscriptionId = '75e95eeba0b3450feda8d880efe00600816e5934160a4757fbdaa99a0e3bb436i0'

  // Input the InscriptionID
  await component.locator('input[placeholder="TickDeployID"]').fill(moveTickInscriptionId)
  await component.locator('input[placeholder="UserInput"]').fill('20240306')

  // Click the mint button
  await component.locator('button:has-text("Mint")').click()

  // Optionally, check for the presence of the inscriptionId in the output/result
  await expect(component).toContainText('Mint Result: ', {timeout: 60000 })
})
