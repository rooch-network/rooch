// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as Dialog from '@radix-ui/react-dialog'
import clsx from 'clsx'
import { useState } from 'react'
import type { ReactNode } from 'react'

import { BackIcon } from '../icons/BackIcon.js'
import { CloseIcon } from '../icons/CloseIcon.js'
import { StyleMarker } from '../styling/StyleMarker.js'
import { Heading } from '../ui/Heading.js'
import { IconButton } from '../ui/IconButton.js'
import * as styles from './SessionModal.css.js'
// import { ConnectionStatus } from './views/ConnectionStatus.js'
import { WhatIsAWallet } from './views/WhatIsAWallet.js'
import { SessionList } from './session-list/SessionList.js'
// import { useCreateSessionKey } from '../../hooks/index.js'
import { Session } from '@roochnetwork/rooch-sdk'
import { SessionStatus } from './views/SessionStatus.js'

type SessionModalView = 'what-is-a-session' | 'session-status' | 'create-status'

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

type ConnectModalProps = {
  /** The trigger button that opens the dialog. */
  trigger: NonNullable<ReactNode>
} & (ControlledModalProps | UncontrolledModalProps)

export function SessionModal({ trigger, open, defaultOpen, onOpenChange }: ConnectModalProps) {
  const [isModalOpen, setModalOpen] = useState(open ?? defaultOpen)
  const [currentView, setCurrentView] = useState<SessionModalView>()
  const [selectedSession, setSelectedSession] = useState<Session>()
  // const { mutate, isError } = useCreateSessionKey()

  const resetSelection = () => {
    setSelectedSession(undefined)
    setCurrentView(undefined)
  }

  const handleOpenChange = (open: boolean) => {
    if (!open) {
      resetSelection()
    }
    setModalOpen(open)
    onOpenChange?.(open)
  }

  // const createSession = () => {
  //   setCurrentView('create-status')
  //   // mutate(
  //   //   { wallet },
  //   //   {
  //   //     onSuccess: () => handleOpenChange(false),
  //   //   },
  //   // )
  // }

  let modalContent: ReactNode | undefined
  switch (currentView) {
    case 'what-is-a-session':
      modalContent = <WhatIsAWallet />
      break
    case 'session-status':
      modalContent = <SessionStatus selectedSession={selectedSession!} />
      break
    case 'create-status':
      modalContent = (
        // <ConnectionStatus
        //   selectedWallet={selectedSession!}
        //   hadConnectionError={isError}
        //   onRetryConnection={connectWallet}
        // />
        <></>
      )
      break
    default:
      modalContent = <WhatIsAWallet />
  }

  return (
    <Dialog.Root open={open ?? isModalOpen} onOpenChange={handleOpenChange}>
      <Dialog.Trigger asChild>{trigger}</Dialog.Trigger>
      <Dialog.Portal>
        <StyleMarker>
          <Dialog.Overlay className={styles.overlay}>
            <Dialog.Content className={styles.content} aria-describedby={undefined}>
              <div
                className={clsx(styles.sessionListContainer, {
                  [styles.sessionListContainerWithViewSelected]: !!currentView,
                })}
              >
                <div className={styles.sessionListContent}>
                  <Dialog.Title className={styles.title} asChild>
                    <Heading as="h2">Session Manager</Heading>
                  </Dialog.Title>
                  <SessionList
                    selectedSessionAuthKey={selectedSession?.getAuthKey()}
                    onSelect={(session) => {
                      setSelectedSession(session)
                      setCurrentView('session-status')
                    }}
                  />
                </div>
                <button
                  className={styles.whatIsASessionButton}
                  onClick={() => setCurrentView('what-is-a-session')}
                  type="button"
                >
                  What is a Session?
                </button>
              </div>
              <div
                className={clsx(styles.viewContainer, {
                  [styles.selectedViewContainer]: !!currentView,
                })}
              >
                <div className={styles.backButtonContainer}>
                  <IconButton type="button" aria-label="Back" onClick={() => resetSelection()}>
                    <BackIcon />
                  </IconButton>
                </div>
                {modalContent}
              </div>
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
