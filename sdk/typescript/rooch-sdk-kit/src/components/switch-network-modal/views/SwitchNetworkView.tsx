// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Wallet } from '../../../wellet/wallet.js'

import { Button } from '../../ui/Button.js'
import { Heading } from '../../ui/Heading.js'
import { Text } from '../../ui/Text.js'
import * as styles from './SwitchNetworkView.css.js'
import { WalletNetworkType } from '../../../wellet/index.js'
import { useProgress } from '../../ProgressProvider.js'
import { useState } from 'react'
import { checkWalletNetwork } from '../../util/wallet.js'
import { useCurrentNetwork } from '../../../hooks/index.js'

type CheckWalletViewProps = {
  wallet: Wallet
  targetNetWork: WalletNetworkType
  switchNetwork: (wallet: Wallet, target: WalletNetworkType) => Promise<void>
  onSuccess: () => void
}

export function SwitchNetworkView({
  wallet,
  onSuccess,
  targetNetWork,
  switchNetwork,
}: CheckWalletViewProps) {
  const { start, finish, loading } = useProgress()
  const [error, setError] = useState(false)
  const [support, setSupport] = useState(true)
  const roochNetwork = useCurrentNetwork()
  const switch2Network = () => {
    start()
    switchNetwork(wallet, targetNetWork)
      .catch((e: Error) => {
        if ('message' in e && e.message.includes('not support')) {
          setSupport(false)
        }
        setError(true)
      })
      .finally(() => {
        finish()
      })
  }

  const refresh = () => {
    start()
    checkWalletNetwork(wallet, roochNetwork)
      .then((result) => {
        if (!result) {
          onSuccess()
        }
      })
      .finally(() => finish())
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
          {!loading
            ? `${error ? 'Switch failed' : `Wallet network is not ${targetNetWork}`}`
            : 'being processed...'}
        </Text>
        <Text color="muted">
          {!loading
            ? `Please ${error ? 'manually' : ''} switch the wallet network to ${targetNetWork}`
            : ''}
        </Text>
      </div>
      <div className={styles.retryButtonContainer}>
        <Button
          disabled={loading}
          type="button"
          variant="outline"
          onClick={() => (support ? switch2Network() : refresh())}
        >
          {support ? (error ? 'Retry Switch' : 'Switch') : 'Refresh'}
        </Button>
      </div>
    </div>
  )
}
