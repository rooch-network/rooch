// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { test, expect } from "vitest"
import sum from "../../sum"

test("sums two numbers", () => {
  expect(sum(4, 7)).toBe(11)
})
