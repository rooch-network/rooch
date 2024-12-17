// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { ReactNode, useState } from 'react'
import * as Dialog from '@radix-ui/react-dialog'

import * as styles from './Modal.css.js'

import { StyleMarker } from '../styling/StyleMarker.js'
import { ProgressProvider } from '../ProgressProvider.js'
import { IconButton } from './IconButton.js'
import { CloseIcon } from '../icons/CloseIcon.js'

export type ControlledModalAProps = {
  /** The controlled open state of the dialog. */
  open?: boolean

  /** Event handler called when the open state of the dialog changes. */
  onOpenChange?: (open: boolean) => void

  defaultOpen?: boolean
}

export type ModalProps = {
  /** The trigger button that opens the dialog. */
  trigger: NonNullable<ReactNode>
  children: ReactNode
} & ControlledModalAProps

export function Modal({ trigger, children, open, defaultOpen, onOpenChange }: ModalProps) {
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
              <ProgressProvider>{children}</ProgressProvider>
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
