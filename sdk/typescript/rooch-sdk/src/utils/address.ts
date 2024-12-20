// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
export const toShortStr = (
  input: string,
  shortOpt: { start: number; end: number } = {
    start: 6,
    end: 6,
  },
) => {
  try {
    if (input.length <= shortOpt.start + shortOpt.end) {
      return input
    }
    return `${input.substring(0, shortOpt.start)}...${input.substring(
      input.length - shortOpt.end,
      input.length,
    )}`
  } catch (error) {
    return ''
  }
}
