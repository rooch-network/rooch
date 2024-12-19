// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as Dialog from '@radix-ui/react-dialog'
import clsx from 'clsx'
import { useEffect, useState } from 'react'
import type { ReactNode } from 'react'

import { BackIcon } from '../icons/BackIcon.js'
import { CloseIcon } from '../icons/CloseIcon.js'
import { StyleMarker } from '../styling/StyleMarker.js'
import { Heading } from '../ui/Heading.js'
import { IconButton } from '../ui/IconButton.js'
import * as styles from './SessionModal.css.js'
import { WhatIsASessionView } from './views/WhatIsASessionView.js'
import { SessionList } from './session-list/SessionList.js'
import { Session } from '@roochnetwork/rooch-sdk'
import { SessionView } from './views/SessionView.js'
import { useCurrentSession, useSessions } from '../../hooks/index.js'
import { ProgressProvider } from '../ProgressProvider.js'

type SessionModalView = 'what-is-a-session' | 'session-status'

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
  const sessions = useSessions()
  const currentSession = useCurrentSession()
  const [isModalOpen, setModalOpen] = useState(open ?? defaultOpen)
  const [currentView, setCurrentView] = useState<SessionModalView>('what-is-a-session')
  const [selectedSession, setSelectedSession] = useState<Session>()
  const [sessionTitle, setSessionTitle] = useState<string>()

  const resetSelection = () => {
    setSelectedSession(undefined)
    setCurrentView('what-is-a-session')
  }

  const handleOpenChange = (open: boolean) => {
    if (!open) {
      resetSelection()
    }
    setModalOpen(open)
    onOpenChange?.(open)
  }

  useEffect(() => {
    if (currentSession) {
      setSelectedSession(currentSession)
      setCurrentView('session-status')
    } else {
      setSelectedSession(undefined)
      setCurrentView('what-is-a-session')
    }
  }, [currentSession])

  let modalContent: ReactNode | undefined
  switch (currentView) {
    case 'session-status':
      modalContent = (
        <SessionView
          selectedSession={selectedSession!}
          removedCallback={(session) => {
            if (sessions.length > 1) {
              sessions.forEach((v, i) => {
                if (v.getAuthKey() === session.getAuthKey()) {
                  if (i - 1 < 0) {
                    return
                  }
                  setSelectedSession(sessions[i - 1])
                  setCurrentView('session-status')
                }
              })
            }
          }}
        />
      )
      break
    default:
      modalContent = (
        <WhatIsASessionView
          getTitle={(title) => {
            setSessionTitle(title)
          }}
        />
      )
  }

  useEffect(() => {
    if (sessions.length === 0) {
      setCurrentView('what-is-a-session')
    }
  }, [sessions])

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
                    selectedSessionAuthKey={selectedSession?.getAuthKey() || sessionTitle}
                    onSelect={(session) => {
                      if (!session) {
                        setSelectedSession(undefined)
                        setCurrentView('what-is-a-session')
                      } else {
                        setSelectedSession(session)
                        setCurrentView('session-status')
                      }
                    }}
                  />
                </div>
                <button
                  className={styles.whatIsASessionButton}
                  onClick={() => setCurrentView('what-is-a-session')}
                  type="button"
                >
                  What is a session?
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
                <ProgressProvider>{modalContent}</ProgressProvider>
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
