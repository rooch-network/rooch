// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { clsx } from 'clsx'

import { Heading } from '../../ui/Heading.js'
import * as styles from './SessionListItem.css.js'
import { toShortStr } from '@roochnetwork/rooch-sdk'

type SessionListItemProps = {
  authKey: string
  isSelected?: boolean
  onClick: () => void
}

export function SessionListItem({ authKey, onClick, isSelected = false }: SessionListItemProps) {
  return (
    <li className={styles.container}>
      <button
        className={clsx(styles.walletItem, { [styles.selectedWalletItem]: isSelected })}
        type="button"
        onClick={onClick}
      >
        <Heading size="md" truncate asChild>
          <div>
            {toShortStr(authKey, {
              start: 12,
              end: 6,
            })}
          </div>
        </Heading>
      </button>
    </li>
  )
}
