// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { Wallet } from '../../../wellet/wallet.js'

import { Button } from '../../ui/Button.js'
import { Heading } from '../../ui/Heading.js'
import { Text } from '../../ui/Text.js'
import * as styles from './ConnectionStatus.css.js'

type ConnectionStatusProps = {
  selectedWallet: Wallet
  info?: string[]
  hadConnectionError: boolean
  onRetryConnection: (selectedWallet: Wallet) => void
}

export function ConnectionStatus({
  selectedWallet,
  info,
  hadConnectionError,
  onRetryConnection,
}: ConnectionStatusProps) {
  return (
    <div className={styles.container}>
      {selectedWallet.getName() && (
        <img
          className={styles.walletIcon}
          src={selectedWallet.getIcon()}
          alt={`${selectedWallet.getName()} logo`}
        />
      )}
      <div className={styles.title}>
        <Heading as="h2" size="xl">
          Opening {selectedWallet.getName()}
        </Heading>
      </div>
      <div className={styles.connectionStatus}>
        {hadConnectionError ? (
          <Text color="danger">Connection failed</Text>
        ) : (
          <Text color="muted">Confirm connection in the wallet...</Text>
        )}
        {info
          ? info.map((item, i) => (
              <Text color="warning" key={i}>
                {item}
              </Text>
            ))
          : null}
      </div>
      {hadConnectionError ? (
        <div className={styles.retryButtonContainer}>
          <Button type="button" variant="outline" onClick={() => onRetryConnection(selectedWallet)}>
            Retry Connection
          </Button>
        </div>
      ) : null}
    </div>
  )
}
