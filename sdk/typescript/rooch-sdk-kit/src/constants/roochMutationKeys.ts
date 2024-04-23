// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { MutationKey } from '@tanstack/react-query'

function formMutationKeyFn(baseEntity: string) {
  return function mutationKeyFn(additionalKeys: MutationKey = []) {
    return [{ ...roochMutationKeys.all, baseEntity }, ...additionalKeys]
  }
}

export const roochMutationKeys = {
  all: { baseScope: 'rooch' },
  addNetwork: formMutationKeyFn('add-network'),
  switchNetwork: formMutationKeyFn('switch-network'),
  removeNetwork: formMutationKeyFn('remove-network'),
  removeSession: formMutationKeyFn('remove-session'),
  transferObject: formMutationKeyFn('transfer-object'),
}
