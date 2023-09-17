// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useState } from 'react'
import { useAuth } from 'src/hooks/useAuth'

// ** Rooch SDK
import { IAccount, Account, JsonRpcProvider, PrivateKeyAuth, Ed25519Keypair } from '@rooch/sdk'

export default function useSessionAccount() {
  const auth = useAuth()
  const [loading, setLoading] = useState(false)
  const [sessionAccount, setSessionAccount] = useState<IAccount | undefined>(undefined)

  const requestWalletCreateSessionKey = (scope: Array<string>, maxInactiveInterval: number): IAccount => {
    const provider = new JsonRpcProvider()

    const pk = Ed25519Keypair.generate()
    const roochAddress = pk.toRoochAddress()
    const authorizer = new PrivateKeyAuth(pk)

    return new Account(provider, roochAddress, authorizer)
  }

  const requestAuthorize = async (scope: Array<string>, maxInactiveInterval: number) => {
    setLoading(true)

    const defaultAccount = auth.defaultAccount()
    if (!defaultAccount) {
      setSessionAccount(undefined)

      return
    }

    if (defaultAccount != null) {
      if (defaultAccount.kp != null) {
        const provider = new JsonRpcProvider()

        const roochAddress = defaultAccount.address
        const authorizer = new PrivateKeyAuth(defaultAccount.kp)

        const account = new Account(provider, roochAddress, authorizer)
        const sessionAccount = await account.createSessionAccount(
          scope[0],
          60 * 20,
          maxInactiveInterval,
        )
        setSessionAccount(sessionAccount)
      } else if (defaultAccount.type === 'ETH') {
        const sessionAccount = await requestWalletCreateSessionKey(scope, maxInactiveInterval)
        setSessionAccount(sessionAccount)
      }
    }

    setLoading(false)
  }

  return { loading, sessionAccount, requestAuthorize }
}
