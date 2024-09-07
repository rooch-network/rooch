// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as path from 'path'
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
    poolOptions: {
      threads: {
        singleThread: true,
      }
    },
    env: {
      NODE_ENV: 'test',
    },
  },
})
