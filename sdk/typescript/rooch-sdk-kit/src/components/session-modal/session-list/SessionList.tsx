// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as styles from './SessionList.css.js'
import { SessionListItem } from './SessionListItem.js'
import { useSessions } from '../../../hooks/index.js'
import { Session } from '@roochnetwork/rooch-sdk'

type WalletListProps = {
  selectedSessionAuthKey?: string
  onSelect: (wallet: Session) => void
}

export function SessionList({ selectedSessionAuthKey, onSelect }: WalletListProps) {
  const sessions = useSessions()

  return (
    <ul className={styles.container}>
      {sessions.map((session) => (
        <SessionListItem
          key={session.getAuthKey()}
          authKey={session.getAuthKey()}
          isSelected={session.getAuthKey() === selectedSessionAuthKey}
          onClick={() => onSelect(session)}
        />
      ))}
    </ul>
  )
}
