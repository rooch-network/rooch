// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as path from 'node:path'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  test: {
    minThreads: 1,
    maxThreads: 8,
    hookTimeout: 1000000,
    testTimeout: 1000000,
    // Global setup for e2e tests - starts shared Rooch server with dynamic port
    globalSetup: ['./test-e2e/globalSetup.ts'],
    // debug
    // poolOptions: {
    //   threads: {
    //     singleThread: true,
    //   }
    // },
    env: {
      NODE_ENV: 'test',
    },
  },
})
