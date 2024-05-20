// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** React Imports
import { ReactNode } from 'react'

export type SidebarType = {
  show: boolean
  onOpen?: () => void
  children: ReactNode
  onClose?: () => void
  hideBackdrop?: boolean
  backDropClick?: () => void
  direction?: 'left' | 'right'
}
