// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ButtonHTMLAttributes, ReactNode } from 'react'

import { useCurrentWallet } from '../hooks/index.js'
import { ConnectModal } from './connect-modal/ConnectModal.js'

type ConnectButtonProps = {
  onClick: () => void
  children: ReactNode
} & ButtonHTMLAttributes<HTMLButtonElement>

export function WalletGuard({ children, onClick }: ConnectButtonProps) {
  const { status } = useCurrentWallet()
  return (
    <>
      {status === 'connected' ? (
        <button
          style={{
            all: 'unset',
            cursor: 'pointer',
          }}
          onClick={onClick}
        >
          {children}
        </button>
      ) : (
        <ConnectModal trigger={children!} onSuccess={onClick} />
      )}
    </>
  )
}
