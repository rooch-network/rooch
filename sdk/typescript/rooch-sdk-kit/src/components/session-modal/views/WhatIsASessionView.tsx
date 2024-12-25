// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useEffect, useState } from 'react'
import { toShortStr } from '@roochnetwork/rooch-sdk'

import { Text } from '../../ui/Text.js'
import { Button } from '../../ui/Button.js'
import { Heading } from '../../ui/Heading.js'
import * as styles from './WhatIsASessionView.css.js'
import { useProgress } from '../../ProgressProvider.js'
import { useCreateSessionKey } from '../../../hooks/index.js'
import { useSessionStore } from '../../../hooks/useSessionsStore.js'
import { getUTCOffset, unix2str } from '../../../utils/time.js'

type WhatIsASessionViewProps = {
  getTitle: (title: string) => void
}

export function WhatIsASessionView({ getTitle }: WhatIsASessionViewProps) {
  const { start, finish } = useProgress()
  const { mutateAsync, isError } = useCreateSessionKey()
  const _sessionConf = useSessionStore((state) => state.sessionConf)

  const [model, setModel] = useState<'create' | 'what'>('what')

  useEffect(() => {
    getTitle(model === 'what' ? 'What is a session' : 'Create session')
  }, [getTitle, model])

  const createSession = () => {
    if (!_sessionConf) {
      return
    }
    setModel('create')
    start()
    mutateAsync({ ..._sessionConf }).finally(() => finish())
  }

  return model === 'what' ? (
    <div className={styles.whatContent}>
      <Heading as="h2">What is a Session</Heading>
      <div className={styles.whatContent}>
        <Text weight="medium" color="muted">
          Rooch's Session Key is a temporary key that facilitates users to interact with the chain.
        </Text>
        <Text weight="medium" color="muted">
          When interacting with Rooch applications, each application generates a session key. It has
          an expiration time and will become invalid if there is no interaction for a long time.
        </Text>
        <div className={styles.whatMoreContent}>
          <Text className={styles.moreInfo}>
            <a
              href="https://rooch.network/learn/core-concepts/accounts/session-key"
              target="_blank"
              rel="noreferrer"
            >
              More Info
            </a>
          </Text>
        </div>
      </div>
      {_sessionConf && (
        <div className={styles.actionButtonContainer}>
          <Button type="button" variant="outline" onClick={createSession}>
            Create
          </Button>
        </div>
      )}
    </div>
  ) : (
    <div className={styles.createSessionContainer}>
      <Heading as="h2">Info</Heading>
      <div className={styles.createSessionContent}>
        <Info name="App name" value={_sessionConf!.appName} />
        <Info name="App url" value={_sessionConf!.appUrl} />
        <Info
          name="Expiration Interval"
          value={
            _sessionConf!.maxInactiveInterval
              ? _sessionConf!.maxInactiveInterval * 1000 + Date.now()
              : 0
          }
        />
        <div className={styles.createSessionScopeContent}>
          <Heading as="h3" size="sm" weight="normal">
            Scope
          </Heading>
          {_sessionConf!.scopes
            .slice(0, 3)
            .map((item) =>
              typeof item === 'string' ? item : `${item.address}::${item.module}::${item.function}`,
            )
            .sort((a, b) => b.length - a.length)
            .map((item) => {
              const _tmp = item.split('::')
              return (
                <Text key={item}>
                  {_tmp[0].length > 3
                    ? toShortStr(item, {
                        start: 12,
                        end: 10 + _tmp[1].length + _tmp[2].length,
                      })
                    : item}
                </Text>
              )
            })}
        </div>
        <div className={styles.createSessionStatus}>
          {isError ? (
            <Text color="danger">Create failed</Text>
          ) : (
            <Text color="muted">Confirm sign in the wallet...</Text>
          )}
        </div>
        {isError ? (
          <div className={styles.actionButtonContainer}>
            <Button type="button" variant="outline" onClick={createSession}>
              Retry Create
            </Button>
          </div>
        ) : null}
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
        {typeof value === 'number' ? `${unix2str(value)} (${getUTCOffset()})` : value}
      </Text>
    </div>
  )
}
