// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as Dialog from '@radix-ui/react-dialog'
import { useState } from 'react'
import type { ReactNode } from 'react'

import { CloseIcon } from '../icons/CloseIcon.js'
import { StyleMarker } from '../styling/StyleMarker.js'
import { IconButton } from '../ui/IconButton.js'
import * as styles from './LocalWalletModal.css.js'
// import { FaucetView } from './views/FaucetView.js'
import { ProgressProvider } from '../ProgressProvider.js'
import { LocalWalletManagerView } from './views/ManagerView.js'

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

type LocalWalletModalProps = {
  /** The trigger button that opens the dialog. */
  trigger: NonNullable<ReactNode>
} & (ControlledModalProps | UncontrolledModalProps)

export function LocalWalletModal({
  trigger,
  open,
  defaultOpen,
  onOpenChange,
}: LocalWalletModalProps) {
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
              <Dialog.Title />
              <ProgressProvider>
                <LocalWalletManagerView />
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
