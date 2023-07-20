// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { resolve } from "path"
import { defineConfig } from "vitest/config"
import dts from "vite-plugin-dts"

export default defineConfig({
  build: {
    lib: {
      entry: resolve(__dirname, "src/index.ts"),
      name: "rooch",
      fileName: "rooch",
    },
  },
  plugins: [dts()],
  test: {
    minThreads: 1,
    maxThreads: 8,
    hookTimeout: 1000000,
    testTimeout: 1000000,
    env: {
      NODE_ENV: "test",
    },
  },
})
