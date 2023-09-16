// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useState } from 'react'
import { useAuth } from 'src/hooks/useAuth'

// ** Rooch SDK
import { IAccount, Account, JsonRpcProvider, PrivateKeyAuth } from '@rooch/sdk'

export default function useSessionAccount() {
  const auth = useAuth()
  const [loading, setLoading] = useState(false)
  const [account, setAccount] = useState<IAccount | undefined>(undefined)

  const requestAuthorize = async (scope: string) => {
    setLoading(true)

    const defaultAccount = auth.defaultAccount()

    if (defaultAccount != null && defaultAccount.kp != null) {
      const provider = new JsonRpcProvider()

      const roochAddress = defaultAccount.address
      const authorizer = new PrivateKeyAuth(defaultAccount.kp)

      const account = new Account(provider, roochAddress, authorizer)
      const sessionAccount = await account.createSessionAccount(scope, 60 * 20, 60 * 10)
      setAccount(sessionAccount)
    }

    setLoading(false)
  }

  return { loading, account, requestAuthorize }
}
