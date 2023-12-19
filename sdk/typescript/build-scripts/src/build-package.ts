#! /usr/bin/env tsx
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { buildPackage } from './utils/buildPackage'

buildPackage().catch((error) => {
  console.error(error)
  process.exit(1)
})
