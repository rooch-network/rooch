// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ReactNode, useEffect, useState } from 'react'
import { useLocation } from 'react-router-dom'
import {
  useCreateSessionKey,
  useCurrentSession,
  useCurrentWallet,
} from '@roochnetwork/rooch-sdk-kit'
import { SessionKeyModal } from '@/components/session-key-modal.tsx'
import { navItems } from '@/navigation'
import { useNetworkVariable } from '@/networks'
import { ErrorValidateCantPayGasDeposit } from '@roochnetwork/rooch-sdk'

interface SessionGuardProps {
  children: ReactNode
}

const defaultScope = [
  '0x1::*::*',
  '0x3::*::*',
]

export const SessionGuard = (props: SessionGuardProps) => {
  const { children } = props

  const { isConnected } = useCurrentWallet()
  const [open, setOpen] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const sessionKey = useCurrentSession()
  const { mutateAsync: createSessionKey } = useCreateSessionKey()
  const mintAddress = useNetworkVariable('mintAddress')

  const location = useLocation()

  useEffect(() => {
    if (!isConnected) {
      return
    }

    setOpen(
      sessionKey === null &&
        navItems.find((item) => location.pathname.startsWith(item.path) && item.auth) !== undefined,
    )
  }, [isConnected, location, sessionKey])

  const allScope = defaultScope.concat(mintAddress.map((address) => `${address}::*::*`))

  const handleAuth = async () => {
    setError(null)
    try {
      await createSessionKey({
        appName: 'rooch-portal',
        appUrl: 'portal.rooch.network',
        scopes: allScope,
        maxInactiveInterval: 60 * 60 * 8,
      })
    } catch (e: any) {
      let msg = ''
      if ('message' in e) {
        msg = e.message
      }
      if ('code' in e) {
        switch (e.code) {
          case ErrorValidateCantPayGasDeposit:
            msg =
              'Authorization failed due to insufficient gas fees. Please ensure you have enough gas fees.'
            break
        }
      }
      setError(msg)
    }
  }

  return (
    <>
      <SessionKeyModal isOpen={open} onAuthorize={handleAuth} scopes={allScope} error={error} />
      {children}
    </>
  )
}
