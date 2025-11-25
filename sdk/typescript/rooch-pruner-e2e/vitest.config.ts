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
    maxThreads: 1, // Disable concurrency to ensure logs are in order
    hookTimeout: 1000000,
    testTimeout: 1000000,
    env: {
      NODE_ENV: 'test',
    },
    reporter: 'verbose',
  },
})
