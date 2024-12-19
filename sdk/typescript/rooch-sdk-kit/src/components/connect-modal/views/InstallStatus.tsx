// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { Wallet } from '../../../wellet/wallet.js'

import { Button } from '../../ui/Button.js'
import { Heading } from '../../ui/Heading.js'
import { Text } from '../../ui/Text.js'
import * as styles from './InstallStatus.css.js'

type InstallStatusProps = {
  selectedWallet: Wallet
}

export function InstallStatus({ selectedWallet }: InstallStatusProps) {
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
      <div className={styles.installStatus}>
        <Text color="danger">undetected Wallet</Text>
      </div>
      <div className={styles.installButtonContainer}>
        <Button
          type="button"
          variant="outline"
          onClick={() => window.open(selectedWallet.getInstallUrl(), '_blank')}
        >
          Install
        </Button>
      </div>
    </div>
  )
}
