// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Button } from '../../ui/Button.js'
import { Heading } from '../../ui/Heading.js'
import * as styles from './CreateSessionView.css.js'
import { toShortStr } from '@roochnetwork/rooch-sdk'
import { Text } from '../../ui/Text.js'
import { useSessionStore } from '../../../hooks/useSessionsStore.js'
import { unix2str, getUTCOffset } from '../../../utils/time.js'
import { useCreateSessionKey } from '../../../hooks/index.js'
import { useProgress } from '../../ProgressProvider.js'

export function CreateSessionView() {
  const _sessionConf = useSessionStore((state) => state.sessionConf)
  const { mutateAsync, isError } = useCreateSessionKey()
  const { start, finish } = useProgress()

  const createSession = async () => {
    if (!_sessionConf) {
      return
    }
    start()
    mutateAsync({ ..._sessionConf }).finally(() => finish())
  }

  if (!_sessionConf) {
    return <></>
  }

  return (
    <div className={styles.container}>
      <Heading as="h2">Info</Heading>
      <div className={styles.content}>
        <Info name="App name" value={_sessionConf.appName} />
        <Info name="App url" value={_sessionConf.appUrl} />
        <Info
          name="Expiration Interval"
          value={
            _sessionConf.maxInactiveInterval
              ? _sessionConf.maxInactiveInterval * 1000 + Date.now()
              : 0
          }
        />
        <div className={styles.scopeContent}>
          <Heading as="h3" size="sm" weight="normal">
            Scope
          </Heading>
          {_sessionConf.scopes
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
