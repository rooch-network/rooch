// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import type { Wallet } from '../../../wellet/wallet.js'

import { Button } from '../../ui/Button.js'
import { Heading } from '../../ui/Heading.js'
import { Text } from '../../ui/Text.js'
import * as styles from './SwitchNetworkView.css.js'
import { WalletNetworkType } from '../../../wellet/index.js'
import { useProgress } from '../../ProgressProvider.js'

type CheckWalletViewProps = {
  wallet: Wallet
  targetNetWork: WalletNetworkType
  switchNetwork: (wallet: Wallet, target: WalletNetworkType) => Promise<void>
}

export function SwitchNetworkView({ wallet, targetNetWork, switchNetwork }: CheckWalletViewProps) {
  const { start, finish, loading } = useProgress()
  const switch2Network = () => {
    start()
    switchNetwork(wallet, targetNetWork).finally(() => {
      finish()
    })
  }

  return (
    <div className={styles.container}>
      {wallet.getName() && (
        <img
          className={styles.walletIcon}
          src={wallet.getIcon()}
          alt={`${wallet.getName()} logo`}
        />
      )}
      <div className={styles.title}>
        <Heading as="h2" size="xl">
          Check {wallet.getName()}
        </Heading>
      </div>
      <div className={styles.connectionStatus}>
        <Text color="danger">
          {!loading ? `Wallet network is not ${targetNetWork}` : 'being processed...'}
        </Text>
        <Text color="muted">
          {!loading ? `Please switch wallet network to ${targetNetWork}` : ''}
        </Text>
      </div>
      <div className={styles.retryButtonContainer}>
        <Button disabled={loading} type="button" variant="outline" onClick={() => switch2Network()}>
          Switch
        </Button>
      </div>
    </div>
  )
}
