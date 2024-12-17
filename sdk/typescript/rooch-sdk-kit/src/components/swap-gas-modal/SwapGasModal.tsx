// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ReactNode } from 'react'
import { SwapGasView } from './views/SwapGasView.js'
import { ControlledModalAProps, Modal } from '../ui/Modal.js'

export type SwapGasModalProps = {
  /** The trigger button that opens the dialog. */
  trigger: NonNullable<ReactNode>
} & ControlledModalAProps

export function SwapGasModal({ trigger, open, defaultOpen, onOpenChange }: SwapGasModalProps) {
  return (
    <Modal trigger={trigger} open={open} defaultOpen={defaultOpen} onOpenChange={onOpenChange}>
      <SwapGasView />
    </Modal>
  )
}
