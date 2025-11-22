// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { TestBox as TestBoxBase } from '@roochnetwork/test-suite'

let globalTestBox: TestBoxBase | null = null

export async function setup() {
  console.log('🚀 Starting global Rooch server for e2e tests...')

  // Create a TestBox instance for server management only
  globalTestBox = new TestBoxBase()

  // Start Rooch server with dynamic port (0 = auto-assign)
  await globalTestBox.loadRoochEnv('local', 0)

  const roochServerAddress = globalTestBox.getRoochServerAddress()
  const fullUrl = `http://${roochServerAddress}`

  console.log(`✅ Rooch server started at: ${fullUrl}`)

  // Set environment variable for all tests to use
  process.env.VITE_FULLNODE_URL = fullUrl

  // Return teardown function
  return async () => {
    console.log('🧹 Cleaning up global Rooch server...')
    if (globalTestBox) {
      globalTestBox.cleanEnv()
      globalTestBox = null
    }
    console.log('✅ Global cleanup completed')
  }
}
