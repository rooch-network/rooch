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
    // Disable concurrency to ensure logs are in order and avoid resource conflicts
    pool: 'forks',
    poolOptions: {
      forks: {
        singleFork: true,
      },
    },
    hookTimeout: 1000000,
    testTimeout: 1000000,
    teardownTimeout: 120000, // 120 seconds for teardown to ensure cleanup runs
    env: {
      NODE_ENV: 'test',
    },
    reporters: ['verbose'],
    // Note: vitest doesn't have forceExit config option like jest
    // The improved cleanup logic in testbox.ts should handle proper exit
    // If tests still hang, the workflow has a global timeout as fallback
  },
})
