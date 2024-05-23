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

interface SessionGuardProps {
  children: ReactNode
}

const defaultScope = [
  '0x1::*::*',
  '0x3::*::*',
  '0x49ee3cf17a017b331ab2b8a4d40ecc9706f328562f9db63cba625a9c106cdf35::*::*',
]

export const SessionGuard = (props: SessionGuardProps) => {
  const { children } = props

  const { isConnected } = useCurrentWallet()
  const [open, setOpen] = useState(false)

  const sessionKey = useCurrentSession()
  const { mutateAsync: createSessionKey } = useCreateSessionKey()

  const s = useLocation()

  useEffect(() => {
    if (!isConnected) {
      return
    }

    setOpen(
      sessionKey === null &&
        navItems().find((item) => s.pathname.startsWith(item.path) && item.auth) !== undefined,
    )
  }, [isConnected, s, sessionKey])

  const handleAuth = async () => {
    await createSessionKey({
      appName: 'rooch-portal',
      appUrl: 'portal.rooch.network',
      scopes: defaultScope,
    })
  }

  return (
    <>
      <SessionKeyModal isOpen={open} onAuthorize={handleAuth} scopes={defaultScope} />
      {children}
    </>
  )
}
