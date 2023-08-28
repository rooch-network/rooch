// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

export const getInitials = (string: string) =>
  string.split(/\s/).reduce((response, word) => (response += word.slice(0, 1)), '')
