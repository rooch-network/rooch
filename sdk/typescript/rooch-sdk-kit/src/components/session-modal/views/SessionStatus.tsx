// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Button } from '../../ui/Button.js'
import { Heading } from '../../ui/Heading.js'
import * as styles from './SessionStatus.css.js'
import { Session, toShortStr } from '@roochnetwork/rooch-sdk'
import { Text } from '../../ui/Text.js'
import dayjs from 'dayjs'

type ConnectionStatusProps = {
  selectedSession: Session
}

export function SessionStatus({ selectedSession }: ConnectionStatusProps) {
  return (
    <div className={styles.container}>
      <Heading as="h2">{selectedSession.getLastActiveTime()}</Heading>
      <div className={styles.content}>
        <Info name="Create At:" value={selectedSession.localCreateSessionTime} />
        <Info name="Last Active Time:" value={selectedSession.getLastActiveTime()} />
        <Info name="Time Remaining:" value={selectedSession.maxInactiveInterval} />
        <div className={styles.ScopeContent}>
          <Heading as="h3" size="sm" weight="normal">
            Scope
          </Heading>
          {selectedSession.scopes.map((item) => (
            <Text>
              {toShortStr(item, {
                start: 6,
                end: 50,
              })}
            </Text>
          ))}
        </div>
        <div className={styles.removeButtonContainer}>
          <Button type="button" variant="outline" onClick={() => {}}>
            remove
          </Button>
        </div>
      </div>
    </div>
  )
}

export function Info({ name, value }: { name: string; value: number }) {
  return (
    <div className={styles.sessionItemContent}>
      <Text color="danger" style={{ width: '130px' }}>
        {name}
      </Text>
      <Text color="muted">{dayjs.unix(Number(value / 1000)).format('MMM DD, YYYY HH:mm:ss')}</Text>
    </div>
  )
}
