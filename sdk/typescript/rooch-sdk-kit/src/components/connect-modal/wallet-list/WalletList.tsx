// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import * as styles from './WalletList.css.js'
import { WalletListItem } from './WalletListItem.js'
import { Wallet } from '../../../wellet/wallet.js'

type WalletListProps = {
  wallets: Wallet[]
  selectedWalletName?: string
  onSelect: (wallet: Wallet) => void
  walletStatus: Map<string, boolean>
  isDetecting: boolean
}

export function WalletList({
  wallets,
  selectedWalletName,
  onSelect,
  walletStatus,
  isDetecting,
}: WalletListProps) {
  return (
    <ul className={styles.container}>
      {wallets.map((wallet) => (
        <WalletListItem
          key={wallet.getName()}
          name={wallet.getName()}
          icon={wallet.getIcon()}
          isSelected={wallet.getName() === selectedWalletName}
          isInstalled={walletStatus.get(wallet.getName())}
          isDetecting={isDetecting}
          onClick={() => onSelect(wallet)}
        />
      ))}
    </ul>
  )
}
