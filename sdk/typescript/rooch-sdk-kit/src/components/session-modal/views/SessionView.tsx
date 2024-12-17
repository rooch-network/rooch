// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { useEffect, useMemo, useState } from 'react'

import { Button } from '../../ui/Button.js'
import { Heading } from '../../ui/Heading.js'
import * as styles from './SessionView.css.js'
import { Session, toShortStr } from '@roochnetwork/rooch-sdk'
import { Text } from '../../ui/Text.js'
import { SessionKeyGuard } from '../../SessionKeyGuard.js'
import { getUTCOffset, second2Countdown, unix2str } from '../../../utils/time.js'

type ConnectionStatusProps = {
  selectedSession: Session
  removeSession: (session: Session) => Promise<void>
}

export function SessionView({ selectedSession, removeSession }: ConnectionStatusProps) {
  const [timeRemaining, setTimeRemaining] = useState(-1)
  const [removing, setRemoving] = useState(false)

  useEffect(() => {
    if (selectedSession.isSessionExpired()) {
      setTimeRemaining(0)
      return
    }
    const interval = setInterval(() => {
      const now = Date.now() / 1000
      setTimeRemaining(
        selectedSession.getLastActiveTime() / 1000 + selectedSession.maxInactiveInterval - now,
      )
    }, 1000)

    return () => clearInterval(interval)
  }, [selectedSession])

  const scopes = useMemo(() => {
    return selectedSession.scopes
      .slice(0, 3)
      .sort((a, b) => b.length - a.length)
      .map((item) => {
        const _tmp = item.split('::')
        return _tmp[0].length > 3
          ? toShortStr(item, {
              start: 12,
              end: 10 + _tmp[1].length + _tmp[2].length,
            })
          : item
      })
  }, [selectedSession])

  return (
    <div className={styles.container}>
      <Heading as="h2">Info</Heading>
      <div className={styles.content}>
        <Info name="Create At:" value={selectedSession.localCreateSessionTime} />
        <Info name="Last Active Time:" value={selectedSession.getLastActiveTime()} />
        <Info
          name="Time Remaining:"
          value={
            timeRemaining === 0
              ? 'Expired'
              : timeRemaining === -1
                ? 'calculating'
                : second2Countdown(timeRemaining)
          }
        />
        <div className={styles.scopeContent}>
          <Heading as="h3" size="sm" weight="normal">
            Scope
          </Heading>
          {scopes.map((scope) => (
            <Text>${scope}</Text>
          ))}
        </div>
        <div className={styles.moreContent}>
          <Text className={styles.moreInfo}>
            <a href="https://portal.rooch.network/settings" target="_blank" rel="noreferrer">
              More Info
            </a>
          </Text>
        </div>
        <div className={styles.removeButtonContainer}>
          <SessionKeyGuard
            onClick={() => {
              setRemoving(true)
              removeSession(selectedSession).finally(() => setRemoving(false))
            }}
          >
            <Button
              className={styles.removeBtn}
              disabled={removing}
              type="button"
              variant="outline"
            >
              Remove
            </Button>
          </SessionKeyGuard>
        </div>
      </div>
    </div>
  )
}

export function Info({ name, value }: { name: string; value: number | string }) {
  return (
    <div className={styles.sessionItemContent}>
      <Text color="danger" style={{ width: '130px' }}>
        {name}
      </Text>
      <Text color="muted">
        {typeof value === 'number' ? `${unix2str(value)}${getUTCOffset()}` : value}
      </Text>
    </div>
  )
}
