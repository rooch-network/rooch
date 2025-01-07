// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ButtonHTMLAttributes, ReactNode } from 'react'

import { CreateSessionArgs } from '@roochnetwork/rooch-sdk'
import { ConnectModal } from './connect-modal/ConnectModal.js'
import { useSessionStore } from '../hooks/useSessionsStore.js'
import { useCreateSessionKey, useCurrentAddress, useCurrentSession } from '../hooks/index.js'
import { Button } from './ui/Button.js'

type ConnectButtonProps = {
  onClick: () => void
  children: ReactNode
  sessionConf?: CreateSessionArgs
} & ButtonHTMLAttributes<HTMLButtonElement>

export function SessionKeyGuard({ children, sessionConf, onClick }: ConnectButtonProps) {
  const session = useCurrentSession()
  const curSession = useCurrentSession()
  const curAddress = useCurrentAddress()
  const _sessionConf = useSessionStore((state) => state.sessionConf)
  const { mutate } = useCreateSessionKey()
  const handleCreateSession = () => {
    if (curSession && !curSession.isSessionExpired()) {
      onClick()
      return
    }
    const _conf = _sessionConf || sessionConf
    if (_conf) {
      mutate({ ..._conf })
      // TODO: Continue to call
    } else {
      onClick()
    }
  }
  return (
    <>
      {curAddress ? (
        <Button
          asChild
          onClick={() => {
            handleCreateSession()
          }}
        >
          {children}
        </Button>
      ) : (
        <ConnectModal
          trigger={children!}
          onSuccess={() => {
            if (!session) {
              handleCreateSession()
            }
          }}
        />
      )}
    </>
  )
}
