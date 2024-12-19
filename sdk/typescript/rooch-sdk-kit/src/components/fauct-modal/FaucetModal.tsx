// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as Dialog from '@radix-ui/react-dialog'
import { useState } from 'react'
import type { ReactNode } from 'react'

import { CloseIcon } from '../icons/CloseIcon.js'
import { StyleMarker } from '../styling/StyleMarker.js'
import { IconButton } from '../ui/IconButton.js'
import * as styles from './FaucetModal.css.js'
import { FaucetView } from './views/FaucetView.js'
import { ProgressProvider } from '../ProgressProvider.js'

type ControlledModalProps = {
  /** The controlled open state of the dialog. */
  open: boolean

  /** Event handler called when the open state of the dialog changes. */
  onOpenChange: (open: boolean) => void

  defaultOpen?: never
}

type UncontrolledModalProps = {
  open?: never

  onOpenChange?: never

  /** The open state of the dialog when it is initially rendered. Use when you do not need to control its open state. */
  defaultOpen?: boolean
}

type FaucetModalProps = {
  /** The trigger button that opens the dialog. */
  trigger: NonNullable<ReactNode>
  inviter?: string
  swapRGas: () => void
} & (ControlledModalProps | UncontrolledModalProps)

export function FaucetModal({
  trigger,
  inviter,
  swapRGas,
  open,
  defaultOpen,
  onOpenChange,
}: FaucetModalProps) {
  const [isModalOpen, setModalOpen] = useState(open ?? defaultOpen)

  const handleOpenChange = (open: boolean) => {
    setModalOpen(open)
    onOpenChange?.(open)
  }

  return (
    <Dialog.Root open={open ?? isModalOpen} onOpenChange={handleOpenChange}>
      <Dialog.Trigger asChild>{trigger}</Dialog.Trigger>
      <Dialog.Portal>
        <StyleMarker>
          <Dialog.Overlay className={styles.overlay}>
            <Dialog.Content className={styles.content} aria-describedby={undefined}>
              <ProgressProvider>
                <FaucetView inviter={inviter} swapRGas={swapRGas} />
              </ProgressProvider>
              <Dialog.Close className={styles.closeButtonContainer} asChild>
                <IconButton type="button" aria-label="Close">
                  <CloseIcon />
                </IconButton>
              </Dialog.Close>
            </Dialog.Content>
          </Dialog.Overlay>
        </StyleMarker>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
