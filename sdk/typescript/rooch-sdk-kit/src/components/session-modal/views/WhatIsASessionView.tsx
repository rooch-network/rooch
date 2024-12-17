// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { useState } from 'react'
import { toShortStr } from '@roochnetwork/rooch-sdk'

import { Text } from '../../ui/Text.js'
import { Button } from '../../ui/Button.js'
import { Heading } from '../../ui/Heading.js'
import { Info } from './CreateSessionView.js'
import * as styles from './WhatIsASessionView.css.js'
import { useProgress } from '../../ProgressProvider.js'
import { useCreateSessionKey } from '../../../hooks/index.js'
import { useSessionStore } from '../../../hooks/useSessionsStore.js'

export function WhatIsASessionView() {
  const _sessionConf = useSessionStore((state) => state.sessionConf)
  const [model, setModel] = useState<'create' | 'what'>('what')
  const { start, finish } = useProgress()
  const { mutateAsync, isError } = useCreateSessionKey()
  const createSession = async () => {
    if (!_sessionConf) {
      return
    }
    setModel('create')
    start()
    mutateAsync({ ..._sessionConf }).finally(() => finish())
  }
  return model === 'what' ? (
    <div className={styles.container}>
      <Heading as="h2">What is a Session</Heading>
      <div className={styles.content}>
        <Text weight="medium" color="muted">
          Rooch's Session Key is a temporary key that facilitates users to interact with the chain.
        </Text>
        <Text weight="medium" color="muted">
          When interacting with Rooch applications, each application generates a session key. It has
          an expiration time and will become invalid if there is no interaction for a long time.
        </Text>
        <div className={styles.moreContent}>
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
        <div className={styles.createButtonContainer}>
          <Button type="button" variant="outline" onClick={createSession}>
            Create
          </Button>
        </div>
      )}
    </div>
  ) : (
    <div className={styles.container}>
      <Heading as="h2">Info</Heading>
      <div className={styles.content}>
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
        <div className={styles.scopeContent}>
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
          <div className={styles.retryButtonContainer}>
            <Button type="button" variant="outline" onClick={createSession}>
              Retry Create
            </Button>
          </div>
        ) : null}
      </div>
    </div>
  )
}
