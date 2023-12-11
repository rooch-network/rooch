// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
//
import { defineConfig } from 'tsup'

export default defineConfig({
  entry: ['src/index.ts'],
  minify: true,
  target: 'es2020',
})
