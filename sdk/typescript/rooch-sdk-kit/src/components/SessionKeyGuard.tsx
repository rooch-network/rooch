// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ButtonHTMLAttributes, ReactNode } from 'react'

import { useCreateSessionKey, useCurrentAddress, useCurrentSession } from '../hooks/index.js'
import { ConnectModal } from './connect-modal/ConnectModal.js'
import { useSessionStore } from '../hooks/useSessionsStore.js'
import { CreateSessionArgs } from '@roochnetwork/rooch-sdk'

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
    if (curSession) {
      onClick()
      return
    }
    const _conf = _sessionConf || sessionConf
    if (_conf) {
      mutate(_conf, {
        onSuccess: () => {
          onClick()
        },
      })
    }
  }
  return (
    <>
      {curAddress ? (
        <button
          style={{
            all: 'unset',
            cursor: 'pointer',
          }}
          onClick={() => {
            handleCreateSession()
          }}
        >
          {children}
        </button>
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
