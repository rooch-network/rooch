// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { clsx } from 'clsx'
import type { ReactNode } from 'react'

import { Heading } from '../../ui/Heading.js'
import * as styles from './WalletListItem.css.js'

type WalletListItemProps = {
  name: string
  icon: ReactNode
  isSelected?: boolean
  isInstalled?: boolean
  isDetecting?: boolean
  onClick: () => void
}

export function WalletListItem({
  name,
  icon,
  onClick,
  isSelected = false,
  isInstalled = false,
  isDetecting = false,
}: WalletListItemProps) {
  return (
    <li className={styles.container}>
      <button
        className={clsx(styles.walletItem, { [styles.selectedWalletItem]: isSelected })}
        type="button"
        onClick={onClick}
      >
        {icon && typeof icon === 'string' ? (
          <img className={styles.walletIcon} src={icon} alt={`${name} logo`} />
        ) : (
          icon
        )}
        <Heading size="md" truncate asChild>
          <div>{name}</div>
        </Heading>
        <span
          className={clsx(styles.walletStatus, {
            [styles.installedStatus]: (isInstalled && !isDetecting) || name === 'Local',
            [styles.notInstalledStatus]: !isInstalled && !isDetecting,
            [styles.detectingStatus]: isDetecting && name !== 'Local',
          })}
        >
          {isDetecting && name !== 'Local' ? (
            <>
              <span className={styles.loadingSpinner} />
            </>
          ) : isInstalled ? (
            'Installed'
          ) : (
            'Uninstalled'
          )}
        </span>
      </button>
    </li>
  )
}
