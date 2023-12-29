// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// <reference types="vitest" />

import { vanillaExtractPlugin } from '@vanilla-extract/vite-plugin'
import { defineConfig } from 'vite'
// import { configDefaults } from 'vitest/config'

export default defineConfig({
  plugins: [vanillaExtractPlugin()],
  // test: {
  //   exclude: [...configDefaults.exclude, 'tests/**'],
  //   environment: 'jsdom',
  //   restoreMocks: true,
  //   globals: true,
  //   setupFiles: ['./test/setup.ts'],
  // },
})
