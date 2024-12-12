// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ButtonHTMLAttributes, ReactNode } from 'react'

import { SessionModal } from './session-modal/SessionModal.js'

type ConnectButtonProps = {
  onClick: () => void
  children: ReactNode
} & ButtonHTMLAttributes<HTMLButtonElement>

export function SessionKeyGuardButton({ children }: ConnectButtonProps) {
  return <SessionModal trigger={children!} />
}
