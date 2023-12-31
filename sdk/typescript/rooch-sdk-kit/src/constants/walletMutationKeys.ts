// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { MutationKey } from '@tanstack/react-query'

function formMutationKeyFn(baseEntity: string) {
  return function mutationKeyFn(additionalKeys: MutationKey = []) {
    return [{ ...walletMutationKeys.all, baseEntity }, ...additionalKeys]
  }
}

export const walletMutationKeys = {
  all: { baseScope: 'wallet' },
  connectWallet: formMutationKeyFn('connect-wallet'),
  autoConnectWallet: formMutationKeyFn('auto-connect-wallet'),
  switchAccount: formMutationKeyFn('switch-account'),
  createSessionKey: formMutationKeyFn('create-session-key'),
}
