// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

//import { resolve } from 'path'
import { defineConfig } from 'vitest/config'
import dts from 'vite-plugin-dts'
import { nodePolyfills } from 'vite-plugin-node-polyfills'

export default defineConfig({
  // TODO:Replace tsup packaging after repair
  //  build: {
  //    lib: {
  //      entry: resolve(__dirname, 'src/index.ts'),
  //      name: 'Rooch',
  //      fileName: 'rooch',
  //      formats: ['es', 'cjs'],
  //    },
  //  },
  plugins: [
    dts(),
    nodePolyfills({
      // To exclude specific polyfills, add them to this list.
      exclude: [
        'fs', // Excludes the polyfill for `fs` and `node:fs`.
      ],
      // Whether to polyfill specific globals.
      globals: {
        Buffer: true, // can also be 'build', 'dev', or false
        global: true,
        process: true,
      },
      // Whether to polyfill `node:` protocol imports.
      protocolImports: true,
    }),
  ],
  test: {
    minThreads: 1,
    maxThreads: 8,
    hookTimeout: 1000000,
    testTimeout: 1000000,
    env: {
      NODE_ENV: 'test',
    },
    includeSource: ['src/**/*.{js,ts}'],
  },
})
